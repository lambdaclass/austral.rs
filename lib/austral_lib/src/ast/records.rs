use super::{DocString, Ident, Pragma, TypeParam, TypeSpec, Universe};
use crate::lexer::Token;
use chumsky::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct RecordDecl {
    doc_string: Option<DocString>,
    pragmas: Vec<Pragma>,

    name: Ident,
    type_params: Vec<TypeParam>,
    universe: Universe,
    slots: Vec<Slot>,
}

impl RecordDecl {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        group((
            DocString::parser().or_not(),
            Pragma::parser().repeated().collect::<Vec<_>>(),
            just(Token::Record).ignore_then(Ident::parser()),
            TypeParam::parser()
                .separated_by(just(Token::Comma))
                .allow_trailing()
                .collect::<Vec<_>>()
                .delimited_by(just(Token::LBracket), just(Token::RBracket))
                .or_not()
                .map(Option::unwrap_or_default),
            just(Token::Colon).ignore_then(Universe::parser()),
            just(Token::Is)
                .ignore_then(
                    Slot::parser()
                        .then_ignore(just(Token::Semi))
                        .repeated()
                        .collect::<Vec<_>>(),
                )
                .then_ignore(just(Token::End).then_ignore(just(Token::Semi))),
        ))
        .map(
            |(doc_string, pragmas, name, type_params, universe, slots)| Self {
                doc_string,
                pragmas,
                name,
                type_params,
                universe,
                slots,
            },
        )
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Slot {
    doc_string: Option<DocString>,

    name: Ident,
    r#type: TypeSpec,
}

impl Slot {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        group((
            DocString::parser().or_not(),
            Ident::parser(),
            just(Token::Colon)
                .ignore_then(TypeSpec::parser())
                .then_ignore(just(Token::Semi)),
        ))
        .map(|(doc_string, name, r#type)| Self {
            doc_string,
            name,
            r#type,
        })
    }
}
