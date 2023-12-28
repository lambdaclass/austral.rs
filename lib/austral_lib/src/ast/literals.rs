use crate::lexer::Token;
use chumsky::prelude::*;
use std::borrow::Cow;

pub fn literal_nil<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], ()> {
    just(Token::Nil).ignored()
}

pub fn literal_bool<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], bool> {
    any().try_map(|token, _| {
        Ok(match token {
            Token::True => true,
            Token::False => false,
            _ => return Err(Default::default()),
        })
    })
}

pub fn literal_char<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], char> {
    any().try_map(|token, _| {
        Ok(match token {
            Token::Char(value) => value,
            _ => return Err(Default::default()),
        })
    })
}

pub fn literal_u64<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], u64> {
    any().try_map(|token, _| {
        Ok(match token {
            // TODO: Match other non-decimal integers too.
            Token::Decimal(value) => value,
            _ => return Err(Default::default()),
        })
    })
}

pub fn literal_f64<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], f64> {
    any().try_map(|token, _| {
        Ok(match token {
            Token::Float(value) => value,
            _ => return Err(Default::default()),
        })
    })
}

pub fn literal_str<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Cow<'a, str>> {
    any().try_map(|token, _| {
        Ok(match token {
            Token::String(value) => value,
            _ => return Err(Default::default()),
        })
    })
}

#[cfg(test)]
mod literals_parser_tests {
    use super::*;
    use crate::lexer::Token;
    use std::borrow::Cow;

    #[test]
    fn test_literal_str() {
        let hello_world_str = vec![Token::String(Cow::Borrowed("hello world"))];
        assert_eq!(
            literal_str().parse(&hello_world_str).unwrap(),
            Cow::Borrowed("hello world")
        );

        let empty_str = vec![Token::String(Cow::Borrowed(""))];
        assert_eq!(literal_str().parse(&empty_str).unwrap(), Cow::Borrowed(""));
    }

    #[test]
    fn test_literal_u64() {
        assert_eq!(literal_u64().parse(&[Token::Decimal(10)]).unwrap(), 10);
        assert_eq!(literal_u64().parse(&[Token::Decimal(0)]).unwrap(), 0);
        assert_eq!(literal_u64().parse(&[Token::Decimal(1)]).unwrap(), 1);
    }

    #[test]
    fn test_literal_f64() {
        assert_eq!(literal_f64().parse(&[Token::Float(10.0)]).unwrap(), 10.0);
        assert_eq!(literal_f64().parse(&[Token::Float(0.0)]).unwrap(), 0.0);
        assert_eq!(literal_f64().parse(&[Token::Float(1.0)]).unwrap(), 1.0);
        assert_eq!(literal_f64().parse(&[Token::Float(-1.0)]).unwrap(), -1.0);
    }

    #[test]
    fn test_literal_char() {
        assert_eq!(literal_char().parse(&[Token::Char('a')]).unwrap(), 'a');
    }

    #[test]
    fn test_literal_bool() {
        assert_eq!(literal_bool().parse(&[Token::True]).unwrap(), true);
        assert_eq!(literal_bool().parse(&[Token::False]).unwrap(), false);
    }

    #[test]
    fn test_literal_nil() {
        assert_eq!(literal_nil().parse(&[Token::Nil]).unwrap(), ());
    }
}
