use super::{Ident, TypeParam, TypeSpec};
use crate::lexer::Token;
use chumsky::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct FunctionDecl {
    // TODO: doc_string: Option<DocString>,
    // TODO: pragmas: Vec<Pragma>,
    //
    type_params: Vec<TypeParam>,
    name: Ident,
    params: Vec<Param>,
    ret_type: TypeSpec,
}

impl FunctionDecl {
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
            just(Token::Function).ignore_then(Ident::parser()),
            Param::parser()
                .separated_by(just(Token::Comma))
                .allow_trailing()
                .collect::<Vec<_>>()
                .delimited_by(just(Token::LParen), just(Token::RParen)),
            just(Token::Colon)
                .ignore_then(TypeSpec::parser())
                .then_ignore(just(Token::Semi)),
        ))
        .map(|(type_params, name, params, ret_type)| Self {
            type_params,
            name,
            params,
            ret_type,
        })
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct FunctionDef {
    // TODO: Populate.
}

impl FunctionDef {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        todo()
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct MethodDecl {
    // TODO: doc_string: Option<DocString>,
    //
    type_params: Vec<TypeParam>,
    name: Ident,
    params: Vec<Param>,
    ret_ty: TypeSpec,
}

impl MethodDecl {
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
            just(Token::Method).ignore_then(Ident::parser()),
            Param::parser()
                .separated_by(just(Token::Comma))
                .allow_trailing()
                .collect::<Vec<_>>()
                .delimited_by(just(Token::LParen), just(Token::RParen)),
            just(Token::Colon)
                .ignore_then(TypeSpec::parser())
                .then_ignore(just(Token::Semi)),
        ))
        .map(|(type_params, name, params, ret_ty)| Self {
            type_params,
            name,
            params,
            ret_ty,
        })
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct MethodDef {
    // TODO: Populate.
}

impl MethodDef {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        todo()
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Param {
    name: Ident,
    r#type: TypeSpec,
}

impl Param {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        Ident::parser()
            .then_ignore(just(Token::Colon))
            .then(TypeSpec::parser())
            .map(|(name, r#type)| Self { name, r#type })
    }
}
