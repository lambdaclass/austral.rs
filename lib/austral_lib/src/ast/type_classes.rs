use super::{DocString, Extra, Ident, MethodDecl, MethodDef, Pragma, TypeParam};
use crate::lexer::Token;
use chumsky::prelude::*;
use serde::{Deserialize, Serialize};

pub type TypeClassDecl = TypeClassBase<MethodDecl>;
pub type TypeClassDef = TypeClassBase<MethodDef>;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TypeClassBase<TMethod> {
    doc_string: Option<DocString>,
    pragmas: Vec<Pragma>,

    name: Ident,
    type_param: TypeParam,
    methods: Vec<TMethod>,
}

impl TypeClassBase<MethodDecl> {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self, Extra<'a>> {
        group((
            DocString::parser().or_not(),
            Pragma::parser().repeated().collect::<Vec<_>>(),
            just(Token::TypeClass).ignore_then(Ident::parser()),
            TypeParam::parser().delimited_by(just(Token::LParen), just(Token::RParen)),
            just(Token::Is)
                .ignore_then(MethodDecl::parser().repeated().collect::<Vec<_>>())
                .then_ignore(just(Token::End))
                .then_ignore(just(Token::Semi)),
        ))
        .map(|(doc_string, pragmas, name, type_param, methods)| Self {
            doc_string,
            pragmas,
            name,
            type_param,
            methods,
        })
    }
}

impl TypeClassBase<MethodDef> {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self, Extra<'a>> {
        group((
            DocString::parser().or_not(),
            Pragma::parser().repeated().collect::<Vec<_>>(),
            just(Token::TypeClass).ignore_then(Ident::parser()),
            TypeParam::parser().delimited_by(just(Token::LParen), just(Token::RParen)),
            just(Token::Is)
                .ignore_then(MethodDef::parser().repeated().collect::<Vec<_>>())
                .then_ignore(just(Token::End))
                .then_ignore(just(Token::Semi)),
        ))
        .map(|(doc_string, pragmas, name, type_param, methods)| Self {
            doc_string,
            pragmas,
            name,
            type_param,
            methods,
        })
    }
}
