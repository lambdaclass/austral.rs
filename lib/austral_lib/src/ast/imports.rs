use super::common::Ident;
use crate::lexer::Token;
use chumsky::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ImportStmt {
    module: Vec<Ident>,
    symbols: Vec<ImportedSymbol>,
}

impl ImportStmt {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        just(Token::Import)
            .ignore_then(
                Ident::parser()
                    .separated_by(just(Token::Period))
                    .at_least(1)
                    .collect::<Vec<_>>(),
            )
            .then(
                ImportedSymbol::parser()
                    .separated_by(just(Token::Comma))
                    .allow_trailing()
                    .collect::<Vec<_>>()
                    .delimited_by(just(Token::LParen), just(Token::RParen)),
            )
            .then_ignore(just(Token::Semi))
            .map(|(module, symbols)| Self { module, symbols })
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ImportedSymbol {
    import_name: Ident,
    rename_into: Option<Ident>,
}

impl ImportedSymbol {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        Ident::parser()
            .then(just(Token::As).ignore_then(Ident::parser()).or_not())
            .map(|(import_name, rename_into)| Self {
                import_name,
                rename_into,
            })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn parse_import_stmt() {
        let tokens = [
            Token::Import,
            Token::Ident("A"),
            Token::Period,
            Token::Ident("B"),
            Token::Period,
            Token::Ident("C"),
            Token::LParen,
            Token::RParen,
            Token::Semi,
        ];

        assert_eq!(
            ImportStmt::parser().parse(&tokens).into_result(),
            Ok(ImportStmt {
                module: vec![Ident::new("A"), Ident::new("B"), Ident::new("C")],
                symbols: Vec::new(),
            })
        );
    }

    #[test]
    fn parse_imported_symbol() {
        assert_eq!(
            ImportedSymbol::parser()
                .parse(&[Token::Ident("a")])
                .into_result(),
            Ok(ImportedSymbol {
                import_name: Ident::new("a"),
                rename_into: None,
            }),
        );
        assert_eq!(
            ImportedSymbol::parser()
                .parse(&[Token::Ident("a"), Token::As, Token::Ident("b")])
                .into_result(),
            Ok(ImportedSymbol {
                import_name: Ident::new("a"),
                rename_into: Some(Ident::new("b")),
            }),
        );
    }
}
