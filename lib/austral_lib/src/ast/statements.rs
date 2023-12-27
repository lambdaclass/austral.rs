use super::{Expression, Ident, PathExpr, TypeSpec};
use crate::lexer::Token;
use chumsky::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum Statement {
    Assign(AssignStmt),
    Borrow(BorrowStmt),
    Case(CaseStmt),
    Discard(DiscardStmt),
    For(ForStmt),
    If(IfStmt),
    Let(LetStmt),
    Return(ReturnStmt),
    While(WhileStmt),
}

impl Statement {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        todo()
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

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct CaseWhen {
    value: Ident,
}

impl CaseWhen {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        group((
            just(Token::When).ignore_then(Ident::parser()),
        ))
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct DiscardStmt {
    // value: Expr,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ForStmt {
    name: Ident,
    // range: Range<Expr>,
    // contents: Block,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct IfStmt {
    // value: Expr,
    // contents: Block,
    // else: ???
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct LetStmt {
    // TODO: Populate.
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ReturnStmt {
    // value: Expr,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct WhileStmt {
    // value: Expr,
    // contents: Block,
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
