use super::{DocString, Extra, Ident, Pragma, Statement, TypeParam, TypeSpec};
use crate::lexer::Token;
use chumsky::prelude::*;
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FunctionDecl {
    pub doc_string: Option<DocString>,
    pub pragmas: Vec<Pragma>,

    pub type_params: Vec<TypeParam>,
    pub name: Ident,
    pub params: Vec<Param>,
    pub ret_type: TypeSpec,
}

impl FunctionDecl {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self, Extra<'a>> {
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
    pub decl: FunctionDecl,
    pub body: Vec<Statement>,
}

impl FunctionDef {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self, Extra<'a>> {
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
    pub doc_string: Option<DocString>,

    pub type_params: Vec<TypeParam>,
    pub name: Ident,
    pub params: Vec<Param>,
    pub ret_ty: TypeSpec,
}

impl MethodDecl {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self, Extra<'a>> {
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
    pub decl: MethodDecl,
    pub body: Vec<Statement>,
}

impl MethodDef {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self, Extra<'a>> {
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
    pub name: Ident,
    pub r#type: TypeSpec,
}

impl Param {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self, Extra<'a>> {
        Ident::parser()
            .then_ignore(just(Token::Colon))
            .then(TypeSpec::parser())
            .map(|(name, r#type)| Self { name, r#type })
    }
}
