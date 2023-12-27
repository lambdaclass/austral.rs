use super::{Ident, TypeSpec};
use crate::lexer::Token;
use chumsky::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ConstantDecl {
    // TODO: doc_string: Option<DocString>,
    // TODO: pragmas: Vec<Pragma>,
    //
    name: Ident,
    r#type: TypeSpec,
}

impl ConstantDecl {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        just(Token::Constant)
            .ignore_then(Ident::parser())
            .then_ignore(just(Token::Colon))
            .then(TypeSpec::parser())
            .then_ignore(just(Token::Semi))
            .map(|(name, r#type)| Self { name, r#type })
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ConstantDef {
    // TODO: Populate.
}

impl ConstantDef {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        todo()
    }
}
