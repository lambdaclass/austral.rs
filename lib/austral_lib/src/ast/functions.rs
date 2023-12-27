use super::{DocString, Ident, Pragma, Statement, TypeParam, TypeSpec};
use crate::lexer::Token;
use chumsky::prelude::*;
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FunctionDecl {
    doc_string: Option<DocString>,
    pragmas: Vec<Pragma>,

    type_params: Vec<TypeParam>,
    name: Ident,
    params: Vec<Param>,
    ret_type: TypeSpec,
}

impl FunctionDecl {
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
        .map(
            |(doc_string, pragmas, type_params, name, params, ret_type)| Self {
                doc_string,
                pragmas,
                type_params,
                name,
                params,
                ret_type,
            },
        )
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FunctionDef {
    decl: FunctionDecl,
    body: Vec<Statement>,
}

impl FunctionDef {
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
            just(Token::Function).ignore_then(Ident::parser()),
            Param::parser()
                .separated_by(just(Token::Comma))
                .allow_trailing()
                .collect::<Vec<_>>()
                .delimited_by(just(Token::LParen), just(Token::RParen)),
            just(Token::Colon).ignore_then(TypeSpec::parser()),
            Statement::parser()
                .repeated()
                .collect::<Vec<_>>()
                .delimited_by(just(Token::Is), just(Token::End)),
        ))
        .then_ignore(just(Token::Semi))
        .map(
            |(doc_string, pragmas, type_params, name, params, ret_type, body)| Self {
                decl: FunctionDecl {
                    doc_string,
                    pragmas,
                    type_params,
                    name,
                    params,
                    ret_type,
                },
                body,
            },
        )
    }
}

impl Deref for FunctionDef {
    type Target = FunctionDecl;

    fn deref(&self) -> &Self::Target {
        &self.decl
    }
}

impl DerefMut for FunctionDef {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.decl
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct MethodDecl {
    doc_string: Option<DocString>,

    type_params: Vec<TypeParam>,
    name: Ident,
    params: Vec<Param>,
    ret_ty: TypeSpec,
}

impl MethodDecl {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        group((
            DocString::parser().or_not(),
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
        .map(|(doc_string, type_params, name, params, ret_ty)| Self {
            doc_string,
            type_params,
            name,
            params,
            ret_ty,
        })
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct MethodDef {
    decl: MethodDecl,
    body: Vec<Statement>,
}

impl MethodDef {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        group((
            DocString::parser().or_not(),
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
            just(Token::Colon).ignore_then(TypeSpec::parser()),
            Statement::parser()
                .repeated()
                .collect::<Vec<_>>()
                .delimited_by(just(Token::Is), just(Token::End)),
        ))
        .then_ignore(just(Token::Semi))
        .map(
            |(doc_string, type_params, name, params, ret_ty, body)| Self {
                decl: MethodDecl {
                    doc_string,
                    type_params,
                    name,
                    params,
                    ret_ty,
                },
                body,
            },
        )
    }
}

impl Deref for MethodDef {
    type Target = MethodDecl;

    fn deref(&self) -> &Self::Target {
        &self.decl
    }
}

impl DerefMut for MethodDef {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.decl
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
