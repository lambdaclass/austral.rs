use super::{DocString, Extra, Ident, Pragma, Slot, TypeParam};
use crate::lexer::Token;
use chumsky::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct UnionDecl {
    doc_string: Option<DocString>,
    pragmas: Vec<Pragma>,

    name: Ident,
    type_params: Vec<TypeParam>,
    cases: Vec<Case>,
}

impl UnionDecl {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self, Extra<'a>> {
        group((
            DocString::parser().or_not(),
            Pragma::parser().repeated().collect::<Vec<_>>(),
            just(Token::Union).ignore_then(Ident::parser()),
            TypeParam::parser()
                .separated_by(just(Token::Comma))
                .allow_trailing()
                .collect::<Vec<_>>()
                .delimited_by(just(Token::LBracket), just(Token::RBracket))
                .or_not()
                .map(Option::unwrap_or_default),
            just(Token::Is)
                .ignore_then(Case::parser().repeated().collect::<Vec<_>>())
                .then_ignore(just(Token::End))
                .then_ignore(just(Token::Semi)),
        ))
        .map(|(doc_string, pragmas, name, type_params, cases)| Self {
            doc_string,
            pragmas,
            name,
            type_params,
            cases,
        })
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Case {
    doc_string: Option<DocString>,

    name: Ident,
    fields: Vec<Slot>,
}

impl Case {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self, Extra<'a>> {
        group((
            DocString::parser().or_not(),
            just(Token::Case).ignore_then(Ident::parser()),
            just(Token::Is)
                .ignore_then(Slot::parser().repeated().collect::<Vec<_>>())
                .or_not()
                .map(Option::unwrap_or_default),
        ))
        .then_ignore(just(Token::Semi))
        .map(|(doc_string, name, fields)| Self {
            doc_string,
            name,
            fields,
        })
    }
}
