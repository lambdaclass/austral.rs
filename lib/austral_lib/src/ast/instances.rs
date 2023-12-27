use super::{Ident, MethodDecl, TypeParam, TypeSpec};
use crate::lexer::Token;
use chumsky::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct InstanceDecl {
    // TODO: doc_string: Option<DocString>,
    // TODO: pragmas: Vec<Pragma>,
    //
    type_params: Vec<TypeParam>,
    name: Ident,
    arg: TypeSpec,
    methods: Vec<MethodDecl>,
}

impl InstanceDecl {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        group((
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
        .map(|(type_params, name, arg, methods)| Self {
            type_params,
            name,
            arg,
            methods,
        })
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct InstanceDef {
    // TODO: Populate.
}

impl InstanceDef {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        todo()
    }
}
