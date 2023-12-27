use super::Ident;
use serde::{Deserialize, Serialize};
// use std::ops::Range;

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum Statement {
    Assign(AssignStmt),
    // Borrow(BorrowStmt),
    Case(CaseStmt),
    Discard(DiscardStmt),
    For(ForStmt),
    If(IfStmt),
    Let(LetStmt),
    Return(ReturnStmt),
    While(WhileStmt),
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct AssignStmt {
    // target: LValue,
    // value: Expr,
}

// #[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
// pub struct BorrowStmt {}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct CaseStmt {
    // value: Expr,
    // variants: Vec<WhenStmt>,
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
