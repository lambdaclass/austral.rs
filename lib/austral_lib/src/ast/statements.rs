use super::{Expression, Extra, Ident, PathExpr, TypeSpec};
use crate::lexer::Token;
use chumsky::{prelude::*, recursive::Indirect};
use serde::{Deserialize, Serialize};
use std::{cell::OnceCell, ops::Range, rc::Rc};

#[derive(Default)]
struct ParserCache<'a, 'b> {
    statement: OnceCell<Recursive<Indirect<'a, 'b, &'a [Token<'a>], Statement, Extra<'a>>>>,
    borrow_stmt: OnceCell<Recursive<Indirect<'a, 'b, &'a [Token<'a>], BorrowStmt, Extra<'a>>>>,
    case_stmt: OnceCell<Recursive<Indirect<'a, 'b, &'a [Token<'a>], CaseStmt, Extra<'a>>>>,
    case_when: OnceCell<Recursive<Indirect<'a, 'b, &'a [Token<'a>], CaseWhen, Extra<'a>>>>,
    for_stmt: OnceCell<Recursive<Indirect<'a, 'b, &'a [Token<'a>], ForStmt, Extra<'a>>>>,
    if_stmt: OnceCell<Recursive<Indirect<'a, 'b, &'a [Token<'a>], IfStmt, Extra<'a>>>>,
    while_stmt: OnceCell<Recursive<Indirect<'a, 'b, &'a [Token<'a>], WhileStmt, Extra<'a>>>>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum Statement {
    Assign(AssignStmt),
    Borrow(BorrowStmt),
    Case(CaseStmt),
    Discard(Expression),
    For(ForStmt),
    If(IfStmt),
    Let(LetStmt),
    Return(Expression),
    While(WhileStmt),
}

impl Statement {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self, Extra<'a>> {
        let cache = Rc::new(ParserCache::default());
        Self::recursive_parser(cache)
    }

    fn recursive_parser<'a>(
        cache: Rc<ParserCache<'a, 'a>>,
    ) -> impl Clone + Parser<'a, &'a [Token<'a>], Self, Extra<'a>> {
        match cache.statement.get() {
            Some(parser) => parser.clone(),
            None => {
                let mut parser = Recursive::declare();
                let _ = cache.statement.set(parser.clone());

                parser.define(choice((
                    AssignStmt::parser().map(Self::Assign),
                    BorrowStmt::recursive_parser(cache.clone()).map(Self::Borrow),
                    CaseStmt::recursive_parser(cache.clone()).map(Self::Case),
                    Expression::parser()
                        .then_ignore(just(Token::Semi))
                        .map(Self::Discard),
                    ForStmt::recursive_parser(cache.clone()).map(Self::For),
                    IfStmt::recursive_parser(cache.clone()).map(Self::If),
                    LetStmt::parser().map(Self::Let),
                    just(Token::Return)
                        .ignore_then(Expression::parser())
                        .then_ignore(just(Token::Semi))
                        .map(Self::Return),
                    WhileStmt::recursive_parser(cache.clone()).map(Self::While),
                )));
                parser
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AssignStmt {
    target: PathExpr,
    value: Expression,
}

impl AssignStmt {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self, Extra<'a>> {
        PathExpr::parser()
            .then_ignore(just(Token::Assign))
            .then(Expression::parser())
            .then_ignore(just(Token::Semi))
            .map(|(target, value)| Self { target, value })
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct BorrowStmt {
    name: Ident,
    mut_mode: BorrowMutMode,
    r#type: TypeSpec,
    reg: Ident,
    mode: BorrowMode,
    orig: Ident,
    body: Vec<Statement>,
}

impl BorrowStmt {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self, Extra<'a>> {
        let cache = Rc::new(ParserCache::default());
        Self::recursive_parser(cache)
    }

    fn recursive_parser<'a>(
        cache: Rc<ParserCache<'a, 'a>>,
    ) -> impl Clone + Parser<'a, &'a [Token<'a>], Self, Extra<'a>> {
        match cache.borrow_stmt.get() {
            Some(parser) => parser.clone(),
            None => {
                let mut parser = Recursive::declare();
                let _ = cache.borrow_stmt.set(parser.clone());

                parser.define(
                    group((
                        just(Token::Borrow).ignore_then(Ident::parser()),
                        just(Token::Colon).ignore_then(BorrowMutMode::parser()),
                        TypeSpec::parser()
                            .then_ignore(just(Token::Comma))
                            .then(Ident::parser())
                            .delimited_by(just(Token::LBracket), just(Token::RBracket)),
                        just(Token::Assign).ignore_then(BorrowMode::parser()),
                        Ident::parser(),
                        Statement::recursive_parser(cache.clone())
                            .repeated()
                            .collect::<Vec<_>>()
                            .delimited_by(just(Token::Is), just(Token::End)),
                    ))
                    .map(
                        |(name, mut_mode, (r#type, reg), mode, orig, body)| Self {
                            name,
                            mut_mode,
                            r#type,
                            reg,
                            mode,
                            orig,
                            body,
                        },
                    ),
                );
                parser
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct CaseStmt {
    value: Expression,
    variants: Vec<CaseWhen>,
}

impl CaseStmt {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self, Extra<'a>> {
        let cache = Rc::new(ParserCache::default());
        Self::recursive_parser(cache)
    }

    fn recursive_parser<'a>(
        cache: Rc<ParserCache<'a, 'a>>,
    ) -> impl Clone + Parser<'a, &'a [Token<'a>], Self, Extra<'a>> {
        match cache.case_stmt.get() {
            Some(parser) => parser.clone(),
            None => {
                let mut parser = Recursive::declare();
                let _ = cache.case_stmt.set(parser.clone());

                parser.define(
                    just(Token::Case)
                        .ignore_then(Expression::parser())
                        .then_ignore(just(Token::Of))
                        .then(
                            CaseWhen::recursive_parser(cache)
                                .repeated()
                                .collect::<Vec<_>>(),
                        )
                        .then_ignore(just(Token::End))
                        .then_ignore(just(Token::Case))
                        .then_ignore(just(Token::Semi))
                        .map(|(value, variants)| Self { value, variants }),
                );
                parser
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct CaseWhen {
    ident: Ident,
    bindings: Vec<Binding>,
    block: Vec<Statement>,
}

impl CaseWhen {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self, Extra<'a>> {
        let cache = Rc::new(ParserCache::default());
        Self::recursive_parser(cache)
    }

    fn recursive_parser<'a>(
        cache: Rc<ParserCache<'a, 'a>>,
    ) -> impl Clone + Parser<'a, &'a [Token<'a>], Self, Extra<'a>> {
        match cache.case_when.get() {
            Some(parser) => parser.clone(),
            None => {
                let mut parser = Recursive::declare();
                let _ = cache.case_when.set(parser.clone());

                parser.define(
                    group((
                        just(Token::When).ignore_then(Ident::parser()),
                        Binding::parser()
                            .separated_by(just(Token::Comma))
                            .allow_trailing()
                            .collect::<Vec<_>>(),
                        just(Token::Do).ignore_then(
                            Statement::recursive_parser(cache)
                                .repeated()
                                .collect::<Vec<_>>()
                                .delimited_by(just(Token::Is), just(Token::End)),
                        ),
                    ))
                    .map(|(ident, bindings, block)| Self {
                        ident,
                        bindings,
                        block,
                    }),
                );
                parser
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ForStmt {
    name: Ident,
    range: Range<Expression>,
    contents: Vec<Statement>,
}

impl ForStmt {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self, Extra<'a>> {
        let cache = Rc::new(ParserCache::default());
        Self::recursive_parser(cache)
    }

    fn recursive_parser<'a>(
        cache: Rc<ParserCache<'a, 'a>>,
    ) -> impl Clone + Parser<'a, &'a [Token<'a>], Self, Extra<'a>> {
        match cache.for_stmt.get() {
            Some(parser) => parser.clone(),
            None => {
                let mut parser = Recursive::declare();
                let _ = cache.for_stmt.set(parser.clone());

                parser.define(
                    group((
                        just(Token::For).ignore_then(Ident::parser()),
                        just(Token::From).ignore_then(Expression::parser()),
                        just(Token::To).ignore_then(Expression::parser()),
                        just(Token::Do).ignore_then(
                            Statement::recursive_parser(cache)
                                .repeated()
                                .collect::<Vec<_>>()
                                .delimited_by(just(Token::Is), just(Token::End)),
                        ),
                    ))
                    .then_ignore(just(Token::End))
                    .then_ignore(just(Token::For))
                    .then_ignore(just(Token::Semi))
                    .map(|(name, start, end, contents)| Self {
                        name,
                        range: Range { start, end },
                        contents,
                    }),
                );
                parser
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct IfStmt {
    value: Expression,
    contents: Vec<Statement>,
    r#else: Option<Vec<Statement>>,
}

impl IfStmt {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self, Extra<'a>> {
        let cache = Rc::new(ParserCache::default());
        Self::recursive_parser(cache)
    }

    fn recursive_parser<'a>(
        cache: Rc<ParserCache<'a, 'a>>,
    ) -> impl Clone + Parser<'a, &'a [Token<'a>], Self, Extra<'a>> {
        match cache.if_stmt.get() {
            Some(parser) => parser.clone(),
            None => {
                let mut parser = Recursive::declare();
                let _ = cache.if_stmt.set(parser.clone());

                parser.define(
                    group((
                        just(Token::If).ignore_then(Expression::parser()),
                        just(Token::Then).ignore_then(
                            Statement::recursive_parser(cache.clone())
                                .repeated()
                                .collect::<Vec<_>>()
                                .delimited_by(just(Token::Is), just(Token::End)),
                        ),
                        just(Token::Else)
                            .ignore_then(
                                Statement::recursive_parser(cache.clone())
                                    .repeated()
                                    .collect::<Vec<_>>()
                                    .delimited_by(just(Token::Is), just(Token::End)),
                            )
                            .or_not(),
                    ))
                    .then_ignore(just(Token::End))
                    .then_ignore(just(Token::If))
                    .then_ignore(just(Token::Semi))
                    .map(|(value, contents, r#else)| Self {
                        value,
                        contents,
                        r#else,
                    }),
                );
                parser
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct LetStmt {
    is_mutable: bool,
    target: LetStmtTarget,
    value: Expression,
}

impl LetStmt {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self, Extra<'a>> {
        group((
            choice((just(Token::Let).to(false), just(Token::Var).to(true))),
            LetStmtTarget::parser(),
            just(Token::Assign).ignore_then(Expression::parser()),
        ))
        .then_ignore(just(Token::Semi))
        .map(|(is_mutable, target, value)| Self {
            is_mutable,
            target,
            value,
        })
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum LetStmtTarget {
    Simple { name: Ident, r#type: TypeSpec },
    Destructure(Vec<Binding>),
}

impl LetStmtTarget {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self, Extra<'a>> {
        choice((
            Ident::parser()
                .then_ignore(just(Token::Colon))
                .then(TypeSpec::parser())
                .map(|(name, r#type)| LetStmtTarget::Simple { name, r#type }),
            Binding::parser()
                .separated_by(just(Token::Comma))
                .allow_trailing()
                .collect::<Vec<_>>()
                .delimited_by(just(Token::LBrace), just(Token::RBrace))
                .map(Self::Destructure),
        ))
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct WhileStmt {
    value: Expression,
    contents: Vec<Statement>,
}

impl WhileStmt {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self, Extra<'a>> {
        let cache = Rc::new(ParserCache::default());
        Self::recursive_parser(cache)
    }

    fn recursive_parser<'a>(
        cache: Rc<ParserCache<'a, 'a>>,
    ) -> impl Clone + Parser<'a, &'a [Token<'a>], Self, Extra<'a>> {
        match cache.while_stmt.get() {
            Some(parser) => parser.clone(),
            None => {
                let mut parser = Recursive::declare();
                let _ = cache.while_stmt.set(parser.clone());

                parser.define(
                    just(Token::While)
                        .ignore_then(Expression::parser())
                        .then_ignore(just(Token::Do))
                        .then(
                            Statement::recursive_parser(cache.clone())
                                .repeated()
                                .collect::<Vec<_>>()
                                .delimited_by(just(Token::Is), just(Token::End)),
                        )
                        .then_ignore(just(Token::End))
                        .then_ignore(just(Token::While))
                        .then_ignore(just(Token::Semi))
                        .map(|(value, contents)| Self { value, contents }),
                );
                parser
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum BorrowMutMode {
    Read,
    Write,
}

impl BorrowMutMode {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self, Extra<'a>> {
        choice((
            just(Token::BorrowRead).to(Self::Read),
            just(Token::BorrowWrite).to(Self::Write),
        ))
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum BorrowMode {
    Read,
    Write,
    ReBorrow,
}

impl BorrowMode {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self, Extra<'a>> {
        choice((
            just(Token::BorrowRead).to(Self::Read),
            just(Token::BorrowWrite).to(Self::Write),
            just(Token::ReBorrow).to(Self::ReBorrow),
        ))
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Binding {
    name: Ident,
    rename: Option<Ident>,
    r#type: TypeSpec,
}

impl Binding {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self, Extra<'a>> {
        group((
            Ident::parser(),
            just(Token::As).ignore_then(Ident::parser()).or_not(),
            just(Token::Colon).ignore_then(TypeSpec::parser()),
        ))
        .map(|(name, rename, r#type)| Self {
            name,
            rename,
            r#type,
        })
    }
}

#[cfg(test)]
mod statement_parser_tests {
    use super::*;
    use crate::{ast::AtomicExpr, lexer::Token};
    use std::vec;

    #[test]
    fn test_assign_stmt() {
        let input = vec![
            Token::Ident("foo"),
            Token::Assign,
            Token::Ident("bar"),
            Token::Semi,
        ];

        let expected = Statement::Assign(AssignStmt {
            target: PathExpr {
                first: Ident::new("foo"),
                extra: vec![],
            },
            value: Expression::Atomic(AtomicExpr::Path(PathExpr {
                first: Ident::new("bar"),
                extra: vec![],
            })),
        });

        let actual = Statement::parser().parse(&input).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_return_stmt() {
        let input = vec![Token::Return, Token::Ident("foo"), Token::Semi];

        let expected = Statement::Return(Expression::Atomic(AtomicExpr::Path(PathExpr {
            first: Ident::new("foo"),
            extra: vec![],
        })));

        let actual = Statement::parser().parse(&input).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_discard_stmt() {
        let input = vec![Token::Ident("foo"), Token::Semi];

        let expected = Statement::Discard(Expression::Atomic(AtomicExpr::Path(PathExpr {
            first: Ident::new("foo"),
            extra: vec![],
        })));

        let actual = Statement::parser().parse(&input).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_let_stmt() {
        let input = vec![
            Token::Let,
            Token::Ident("foo"),
            Token::Colon,
            Token::Ident("bar"),
            Token::Assign,
            Token::Ident("baz"),
            Token::Semi,
        ];

        let expected = Statement::Let(LetStmt {
            is_mutable: false,
            target: LetStmtTarget::Simple {
                name: Ident::new("foo"),
                r#type: TypeSpec::Simple {
                    name: Ident::new("bar"),
                },
            },
            value: Expression::Atomic(AtomicExpr::Path(PathExpr {
                first: Ident::new("baz"),
                extra: vec![],
            })),
        });

        let actual = Statement::parser().parse(&input).unwrap();

        assert_eq!(actual, expected);
    }
}
