use std::borrow::Cow;
use logos::{Lexer, Logos};

#[derive(Clone, Debug, Logos, PartialEq)]
pub enum Token<'a> {
    #[token(r"(")]
    LParen,
    #[token(r")")]
    RParen,
    #[token(r"[")]
    LBracket,
    #[token(r"]")]
    RBracket,
    #[token(r"{")]
    LBrace,
    #[token(r"}")]
    RBrace,

    #[token(r"+")]
    Add,
    #[token(r"-")]
    Sub,
    #[token(r"*")]
    Mul,
    #[token(r"/")]
    Div,

    #[token(r"=")]
    Eq,
    #[token(r"/=")]
    NotEq,
    #[token(r"<")]
    Lt,
    #[token(r"<=")]
    LtEq,
    #[token(r">")]
    Gt,
    #[token(r">=")]
    GtEq,

    #[token(r"and")]
    And,
    #[token(r"or")]
    Or,
    #[token(r"not")]
    Not,

    #[token(r"&")]
    BorrowRead,
    #[token(r"&!")]
    BorrowWrite,
    #[token(r"Span")]
    SpanRead,
    #[token(r"Span!")]
    SpanWrite,
    #[token(r"&~")]
    ReBorrow,
    #[token(r"&(")]
    RefTransform,

    #[token(r"module")]
    Module,
    #[token(r"is")]
    Is,
    #[token(r"body")]
    Body,
    #[token(r"import")]
    Import,
    #[token(r"as")]
    As,
    #[token(r"end")]
    End,
    #[token(r"constant")]
    Constant,
    #[token(r"type")]
    Type,
    #[token(r"function")]
    Function,
    #[token(r"generic")]
    Generic,
    #[token(r"record")]
    Record,
    #[token(r"union")]
    Union,
    #[token(r"case")]
    Case,
    #[token(r"of")]
    Of,
    #[token(r"when")]
    When,
    #[token(r"typeclass")]
    TypeClass,
    #[token(r"instance")]
    Instance,
    #[token(r"method")]
    Method,
    #[token(r"if")]
    If,
    #[token(r"then")]
    Then,
    #[token(r"else")]
    Else,
    #[token(r"let")]
    Let,
    #[token(r"var")]
    Var,
    #[token(r"while")]
    While,
    #[token(r"for")]
    For,
    #[token(r"do")]
    Do,
    #[token(r"from")]
    From,
    #[token(r"to")]
    To,
    #[token(r"borrow")]
    Borrow,
    #[token(r"return")]
    Return,
    #[token(r"skip")]
    Skip,
    #[token(r"Free", |_| Universe::Free)]
    #[token(r"Linear", |_| Universe::Linear)]
    #[token(r"Type", |_| Universe::Type)]
    #[token(r"Region", |_| Universe::Region)]
    Universe(Universe),
    #[token(r"pragma")]
    Pragma,
    #[token(r"sizeof")]
    SizeOf,

    #[token(r";")]
    Semi,
    #[token(r",")]
    Comma,
    #[token(r".")]
    Period,
    #[token(r":")]
    Colon,
    #[token(r"->")]
    HypenRight,
    #[token(r"=>")]
    ArrowRight,
    #[token(r":=")]
    Assign,
    #[token(r"!")]
    Deref,

    #[regex(r#"""#, read_string)]
    String(Cow<'a, str>),
    #[regex(r#"""""#, read_triple_string)]
    TripleString(Cow<'a, str>),

    #[token(r"nil")]
    Nil,
    #[token(r"true")]
    True,
    #[token(r"false")]
    False,

    #[regex(r"'([^']|\\')'", |lex| lex.slice().chars().nth(1).unwrap())]
    Char(char),
    #[regex(r"[0-9]+", |lex| lex.slice().parse::<u64>().unwrap())]
    Decimal(u64),
    // TODO: Hexadecimal constant.
    // TODO: Binary constant.
    // TODO: Octal constant.
    #[regex(r"[+-]?[0-9]+\.[0-9]*(?:[eE][+-]?[0-9]+)?", |lex| lex.slice().parse::<f64>().unwrap())]
    Float(f64),

    #[regex(r"[A-Za-z_][A-Za-z0-9_]*", |lex| lex.slice())]
    Ident(&'a str),

    #[token(r"@embed")]
    Embed,
}

/// An universe.
#[derive(Clone, Copy, Debug, Eq, Hash, Logos, PartialEq)]
pub enum Universe {
    Free,
    Linear,
    Region,
    Type,
}

fn read_string<'a>(lex: &mut Lexer<'a, Token<'a>>) -> Option<Cow<'a, str>> {
    let mut iter = lex.remainder().chars().peekable();
    let mut count = 0;

    // Parse contents.
    let raw_contents;
    loop {
        match iter.next()? {
            '"' => {
                raw_contents = std::str::from_utf8(&lex.remainder().as_bytes()[..count]).unwrap();
                lex.bump(count + 1);
                break;
            }
            '\\' if iter.next_if_eq(&'"').is_some() => count += 2,
            '\n' => return None,
            ch => count += ch.len_utf8(),
        }
    }

    // Unescape contents.
    Some(unescape_string(raw_contents))
}

fn read_triple_string<'a>(lex: &mut Lexer<'a, Token<'a>>) -> Option<Cow<'a, str>> {
    let mut iter = lex.remainder().chars().peekable();
    let mut count = 0;

    // Parse contents.
    let raw_contents;
    loop {
        match iter.next()? {
            '"' if iter
                .next_if_eq(&'"')
                .is_some_and(|_| iter.next_if_eq(&'"').is_some()) =>
            {
                raw_contents = std::str::from_utf8(&lex.remainder().as_bytes()[..count]).unwrap();
                lex.bump(count + 3);
                break;
            }
            '\\' if iter.next_if_eq(&'"').is_some() => count += 2,
            ch => count += ch.len_utf8(),
        }
    }

    // Unescape contents.
    Some(unescape_string(raw_contents))
}

fn unescape_string(value: &str) -> Cow<str> {
    let mut target: Option<String> = None;
    let mut offset = 0;

    let make_target = |offset| {
        move || {
            std::str::from_utf8(&value.as_bytes()[..offset])
                .unwrap()
                .to_string()
        }
    };

    let mut is_escaped = false;
    for ch in value.chars() {
        if is_escaped {
            match ch {
                '\\' => target.get_or_insert_with(make_target(offset)).push(ch),
                '"' => target.get_or_insert_with(make_target(offset)).push('"'),
                _ => {
                    offset += '\\'.len_utf8();
                    if let Some(target) = &mut target {
                        target.extend(['\\', ch]);
                    }
                }
            }

            is_escaped = false;
            offset += ch.len_utf8();
        } else if ch == '\\' {
            is_escaped = true;
        } else {
            if let Some(target) = &mut target {
                target.push(ch);
            }

            offset += ch.len_utf8();
        }
    }

    target.map(Cow::Owned).unwrap_or(Cow::Borrowed(value))
}
