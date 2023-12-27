use super::FnCallArgs;
use crate::lexer::Token;
use chumsky::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct DocString {
    contents: String,
}

impl DocString {
    pub fn new(contents: impl Into<String>) -> Self {
        Self {
            contents: contents.into(),
        }
    }

    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        any().try_map(|token, _| match token {
            Token::TripleString(contents) => Ok(Self {
                contents: contents.to_string(),
            }),
            _ => Err(Default::default()),
        })
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Ident {
    pub name: String,
}

impl Ident {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        any().try_map(|token, _| match token {
            Token::Ident(ident) => Ok(Self {
                name: ident.to_string(),
            }),
            _ => Err(Default::default()),
        })
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Universe(pub crate::lexer::Universe);

impl Universe {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        any().try_map(|token, _| match token {
            Token::Universe(x) => Ok(Self(x)),
            _ => Err(Default::default()),
        })
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct TypeParam {
    pub name: Ident,
    pub universe: Universe,
    pub params: Vec<Ident>,
}

impl TypeParam {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        group((
            Ident::parser(),
            just(Token::Colon).ignore_then(Universe::parser()),
            Ident::parser()
                .separated_by(just(Token::Comma))
                .allow_trailing()
                .collect::<Vec<_>>()
                .delimited_by(just(Token::LParen), just(Token::RParen))
                .or_not()
                .map(Option::unwrap_or_default),
        ))
        .map(|(name, universe, params)| Self {
            name,
            universe,
            params,
        })
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Pragma {
    pub name: Ident,
    pub args: FnCallArgs,
}

impl Pragma {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        just(Token::Pragma)
            .ignore_then(Ident::parser())
            .then(
                FnCallArgs::parser()
                    .delimited_by(just(Token::LParen), just(Token::RParen))
                    .or_not()
                    .map(Option::unwrap_or_default),
            )
            .then_ignore(just(Token::Semi))
            .map(|(name, args)| Self { name, args })
    }
}
