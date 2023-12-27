use super::{DocString, Ident, Pragma, Universe};
use crate::lexer::Token;
use chumsky::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TypeDecl {
    doc_string: Option<DocString>,
    pragmas: Vec<Pragma>,

    name: Ident,
    universe: Universe,
}

impl TypeDecl {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        group((
            DocString::parser().or_not(),
            Pragma::parser().repeated().collect::<Vec<_>>(),
            just(Token::Type).ignore_then(Ident::parser()),
            just(Token::Colon).ignore_then(Universe::parser()),
        ))
        .then_ignore(just(Token::Semi))
        .map(|(doc_string, pragmas, name, universe)| Self {
            doc_string,
            pragmas,
            name,
            universe,
        })
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum TypeSpec {
    Simple {
        name: Ident,
    },
    Generic {
        name: Ident,
        type_params: Vec<TypeSpec>,
    },
    BorrowRead {
        lhs: Box<TypeSpec>,
        rhs: Box<TypeSpec>,
    },
    BorrowWrite {
        lhs: Box<TypeSpec>,
        rhs: Box<TypeSpec>,
    },
    SpanRead {
        lhs: Box<TypeSpec>,
        rhs: Box<TypeSpec>,
    },
    SpanWrite {
        lhs: Box<TypeSpec>,
        rhs: Box<TypeSpec>,
    },
}

impl TypeSpec {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        recursive(|parser| {
            choice((
                Ident::parser()
                    .then(
                        parser
                            .clone()
                            .separated_by(just(Token::Comma))
                            .allow_trailing()
                            .collect::<Vec<_>>()
                            .delimited_by(just(Token::LBracket), just(Token::RBracket)),
                    )
                    .map(|(name, type_params)| Self::Generic { name, type_params }),
                Ident::parser().map(|name| Self::Simple { name }),
                just(Token::BorrowRead)
                    .ignore_then(
                        parser
                            .clone()
                            .map(Box::new)
                            .separated_by(just(Token::Comma))
                            .collect_exactly::<[_; 2]>()
                            .delimited_by(just(Token::LBracket), just(Token::RBracket)),
                    )
                    .map(|[lhs, rhs]| Self::BorrowRead { lhs, rhs }),
                just(Token::BorrowWrite)
                    .ignore_then(
                        parser
                            .clone()
                            .map(Box::new)
                            .separated_by(just(Token::Comma))
                            .collect_exactly::<[_; 2]>()
                            .delimited_by(just(Token::LBracket), just(Token::RBracket)),
                    )
                    .map(|[lhs, rhs]| Self::BorrowWrite { lhs, rhs }),
                just(Token::SpanRead)
                    .ignore_then(
                        parser
                            .clone()
                            .map(Box::new)
                            .separated_by(just(Token::Comma))
                            .collect_exactly::<[_; 2]>()
                            .delimited_by(just(Token::LBracket), just(Token::RBracket)),
                    )
                    .map(|[lhs, rhs]| Self::SpanRead { lhs, rhs }),
                just(Token::SpanWrite)
                    .ignore_then(
                        parser
                            .clone()
                            .map(Box::new)
                            .separated_by(just(Token::Comma))
                            .collect_exactly::<[_; 2]>()
                            .delimited_by(just(Token::LBracket), just(Token::RBracket)),
                    )
                    .map(|[lhs, rhs]| Self::SpanWrite { lhs, rhs }),
            ))
        })
    }
}
