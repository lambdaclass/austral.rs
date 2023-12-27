use super::{DocString, Ident, MethodDecl, MethodDef, Pragma, TypeParam, TypeSpec};
use crate::lexer::Token;
use chumsky::prelude::*;
use serde::{Deserialize, Serialize};

pub type InstanceDecl = InstanceBase<MethodDecl>;
pub type InstanceDef = InstanceBase<MethodDef>;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct InstanceBase<TMethod> {
    doc_string: Option<DocString>,
    pragmas: Vec<Pragma>,

    type_params: Vec<TypeParam>,
    name: Ident,
    arg: TypeSpec,
    methods: Vec<TMethod>,
}

impl InstanceBase<MethodDecl> {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        group((
            DocString::parser().or_not(),
            Pragma::parser().repeated().collect::<Vec<_>>(),
            just(Token::Generic)
                .ignore_then(
                    TypeParam::parser()
                        .separated_by(just(Token::Comma))
                        .allow_trailing()
                        .collect::<Vec<_>>()
                        .delimited_by(just(Token::LBracket), just(Token::RBracket)),
                )
                .or_not()
                .map(Option::unwrap_or_default),
            just(Token::Instance).ignore_then(Ident::parser()),
            TypeSpec::parser().delimited_by(just(Token::LParen), just(Token::RParen)),
            just(Token::Is)
                .ignore_then(MethodDecl::parser().repeated().collect::<Vec<_>>())
                .then_ignore(just(Token::End))
                .then_ignore(just(Token::Semi)),
        ))
        .map(
            |(doc_string, pragmas, type_params, name, arg, methods)| Self {
                doc_string,
                pragmas,
                type_params,
                name,
                arg,
                methods,
            },
        )
    }
}

impl InstanceBase<MethodDef> {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        group((
            DocString::parser().or_not(),
            Pragma::parser().repeated().collect::<Vec<_>>(),
            just(Token::Generic)
                .ignore_then(
                    TypeParam::parser()
                        .separated_by(just(Token::Comma))
                        .allow_trailing()
                        .collect::<Vec<_>>()
                        .delimited_by(just(Token::LBracket), just(Token::RBracket)),
                )
                .or_not()
                .map(Option::unwrap_or_default),
            just(Token::Instance).ignore_then(Ident::parser()),
            TypeSpec::parser().delimited_by(just(Token::LParen), just(Token::RParen)),
            just(Token::Is)
                .ignore_then(MethodDef::parser().repeated().collect::<Vec<_>>())
                .then_ignore(just(Token::End))
                .then_ignore(just(Token::Semi)),
        ))
        .map(
            |(doc_string, pragmas, type_params, name, arg, methods)| Self {
                doc_string,
                pragmas,
                type_params,
                name,
                arg,
                methods,
            },
        )
    }
}
