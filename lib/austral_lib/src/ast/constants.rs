use super::{DocString, Expression, Ident, Pragma, TypeSpec};
use crate::lexer::Token;
use chumsky::prelude::*;
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ConstantDecl {
    pub doc_string: Option<DocString>,
    pub pragmas: Vec<Pragma>,

    pub name: Ident,
    pub r#type: TypeSpec,
}

impl ConstantDecl {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        group((
            DocString::parser().or_not(),
            Pragma::parser().repeated().collect::<Vec<_>>(),
            just(Token::Constant).ignore_then(Ident::parser()),
            just(Token::Colon).ignore_then(TypeSpec::parser()),
        ))
        .then_ignore(just(Token::Semi))
        .map(|(doc_string, pragmas, name, r#type)| Self {
            doc_string,
            pragmas,
            name,
            r#type,
        })
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ConstantDef {
    decl: ConstantDecl,
    pub value: Expression,
}

impl ConstantDef {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        group((
            DocString::parser().or_not(),
            Pragma::parser().repeated().collect::<Vec<_>>(),
            just(Token::Constant).ignore_then(Ident::parser()),
            just(Token::Colon).ignore_then(TypeSpec::parser()),
            just(Token::Assign).ignore_then(Expression::parser()),
        ))
        .then_ignore(just(Token::Semi))
        .map(|(doc_string, pragmas, name, r#type, value)| Self {
            decl: ConstantDecl {
                doc_string,
                pragmas,
                name,
                r#type,
            },
            value,
        })
    }
}

impl Deref for ConstantDef {
    type Target = ConstantDecl;

    fn deref(&self) -> &Self::Target {
        &self.decl
    }
}

impl DerefMut for ConstantDef {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.decl
    }
}
