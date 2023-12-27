use super::{Ident, MethodDecl, TypeParam};
use crate::lexer::Token;
use chumsky::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct TypeClassDecl {
    // TODO: doc_string: Option<DocString>,
    // TODO: pragmas: Vec<Pragma>,
    //
    name: Ident,
    type_param: TypeParam,
    methods: Vec<MethodDecl>,
}

impl TypeClassDecl {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        group((
            just(Token::TypeClass).ignore_then(Ident::parser()),
            TypeParam::parser().delimited_by(just(Token::LParen), just(Token::RParen)),
            just(Token::Is)
                .ignore_then(MethodDecl::parser().repeated().collect::<Vec<_>>())
                .then_ignore(just(Token::End))
                .then_ignore(just(Token::Semi)),
        ))
        .map(|(name, type_param, methods)| Self {
            name,
            type_param,
            methods,
        })
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct TypeClassDef {
    // TODO: Populate.
}

impl TypeClassDef {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        todo()
    }
}
