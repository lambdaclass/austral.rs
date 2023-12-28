use super::Extra;
use crate::lexer::Token;
use chumsky::prelude::*;
use std::borrow::Cow;

pub fn literal_nil<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], (), Extra<'a>> {
    just(Token::Nil).ignored()
}

pub fn literal_bool<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], bool, Extra<'a>> {
    any().try_map(|token, span| {
        Ok(match token {
            Token::True => true,
            Token::False => false,
            _ => return Err(Rich::custom(span, "expected a boolean literal")),
        })
    })
}

pub fn literal_char<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], char, Extra<'a>> {
    any().try_map(|token, span| {
        Ok(match token {
            Token::Char(value) => value,
            _ => return Err(Rich::custom(span, "expected a char literal")),
        })
    })
}

pub fn literal_u64<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], u64, Extra<'a>> {
    any().try_map(|token, span| {
        Ok(match token {
            // TODO: Match other non-decimal integers too.
            Token::Decimal(value) => value,
            _ => return Err(Rich::custom(span, "expected an integer literal")),
        })
    })
}

pub fn literal_f64<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], f64, Extra<'a>> {
    any().try_map(|token, span| {
        Ok(match token {
            Token::Float(value) => value,
            _ => return Err(Rich::custom(span, "expected a real literal")),
        })
    })
}

pub fn literal_str<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Cow<'a, str>, Extra<'a>> {
    any().try_map(|token, span| {
        Ok(match token {
            Token::String(value) => value,
            _ => return Err(Rich::custom(span, "expected a string literal")),
        })
    })
}
