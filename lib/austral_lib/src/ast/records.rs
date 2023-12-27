use super::{Ident, TypeParam, TypeSpec, Universe};
use crate::lexer::Token;
use chumsky::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct RecordDecl {
    // TODO: doc_string: Option<DocString>,
    // TODO: pragmas: Vec<Pragma>,
    //
    name: Ident,
    type_params: Vec<TypeParam>,
    universe: Universe,
    slots: Vec<Slot>,
}

impl RecordDecl {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        group((
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
        .map(|(name, type_params, universe, slots)| Self {
            name,
            type_params,
            universe,
            slots,
        })
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Slot {
    // TODO: doc_string: Option<DocString>,
    //
    name: Ident,
    r#type: TypeSpec,
}

impl Slot {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        group((
            Ident::parser(),
            just(Token::Colon)
                .ignore_then(TypeSpec::parser())
                .then_ignore(just(Token::Semi)),
        ))
        .map(|(name, r#type)| Self { name, r#type })
    }
}
