use super::{Expression, Ident, PathExpr, TypeSpec};
use crate::lexer::Token;
use chumsky::prelude::*;
use serde::{Deserialize, Serialize};
use std::ops::Range;

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
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        choice((
            AssignStmt::parser().map(Self::Assign),
            BorrowStmt::parser().map(Self::Borrow),
            CaseStmt::parser().map(Self::Case),
            Expression::parser()
                .then_ignore(just(Token::Semi))
                .map(Self::Discard),
            ForStmt::parser().map(Self::For),
            IfStmt::parser().map(Self::If),
            LetStmt::parser().map(Self::Let),
            just(Token::Return)
                .ignore_then(Expression::parser())
                .then_ignore(just(Token::Semi))
                .map(Self::Return),
            WhileStmt::parser().map(Self::While),
        ))
        .boxed()
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AssignStmt {
    target: PathExpr,
    value: Expression,
}

impl AssignStmt {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
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
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        group((
            just(Token::Borrow).ignore_then(Ident::parser()),
            just(Token::Colon).ignore_then(BorrowMutMode::parser()),
            TypeSpec::parser()
                .then_ignore(just(Token::Comma))
                .then(Ident::parser())
                .delimited_by(just(Token::LBracket), just(Token::RBracket)),
            just(Token::Assign).ignore_then(BorrowMode::parser()),
            Ident::parser(),
            Statement::parser()
                .repeated()
                .collect::<Vec<_>>()
                .delimited_by(just(Token::Is), just(Token::End)),
        ))
        .map(|(name, mut_mode, (r#type, reg), mode, orig, body)| Self {
            name,
            mut_mode,
            r#type,
            reg,
            mode,
            orig,
            body,
        })
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct CaseStmt {
    value: Expression,
    variants: Vec<CaseWhen>,
}

impl CaseStmt {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        just(Token::Case)
            .ignore_then(Expression::parser())
            .then_ignore(just(Token::Of))
            .then(CaseWhen::parser().repeated().collect::<Vec<_>>())
            .then_ignore(just(Token::End))
            .then_ignore(just(Token::Case))
            .then_ignore(just(Token::Semi))
            .map(|(value, variants)| Self { value, variants })
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct CaseWhen {
    ident: Ident,
    bindings: Vec<Binding>,
    block: Vec<Statement>,
}

impl CaseWhen {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        group((
            just(Token::When).ignore_then(Ident::parser()),
            Binding::parser()
                .separated_by(just(Token::Comma))
                .allow_trailing()
                .collect::<Vec<_>>(),
            just(Token::Do).ignore_then(
                Statement::parser()
                    .repeated()
                    .collect::<Vec<_>>()
                    .delimited_by(just(Token::Is), just(Token::End)),
            ),
        ))
        .map(|(ident, bindings, block)| Self {
            ident,
            bindings,
            block,
        })
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ForStmt {
    name: Ident,
    range: Range<Expression>,
    contents: Vec<Statement>,
}

impl ForStmt {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        group((
            just(Token::For).ignore_then(Ident::parser()),
            just(Token::From).ignore_then(Expression::parser()),
            just(Token::To).ignore_then(Expression::parser()),
            just(Token::Do).ignore_then(
                Statement::parser()
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
        })
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct IfStmt {
    value: Expression,
    contents: Vec<Statement>,
    r#else: Option<Vec<Statement>>,
}

impl IfStmt {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        group((
            just(Token::If).ignore_then(Expression::parser()),
            just(Token::Then).ignore_then(
                Statement::parser()
                    .repeated()
                    .collect::<Vec<_>>()
                    .delimited_by(just(Token::Is), just(Token::End)),
            ),
            just(Token::Else)
                .ignore_then(
                    Statement::parser()
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
        })
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct LetStmt {
    is_mutable: bool,
    target: LetStmtTarget,
    value: Expression,
}

impl LetStmt {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
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
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
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
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        just(Token::While)
            .ignore_then(Expression::parser())
            .then_ignore(just(Token::Do))
            .then(
                Statement::parser()
                    .repeated()
                    .collect::<Vec<_>>()
                    .delimited_by(just(Token::Is), just(Token::End)),
            )
            .then_ignore(just(Token::End))
            .then_ignore(just(Token::While))
            .then_ignore(just(Token::Semi))
            .map(|(value, contents)| Self { value, contents })
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum BorrowMutMode {
    Read,
    Write,
}

impl BorrowMutMode {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
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
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
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
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
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
