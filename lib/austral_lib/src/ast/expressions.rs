use super::{
    literal_bool, literal_char, literal_f64, literal_nil, literal_str, literal_u64, Ident, TypeSpec,
};
use crate::lexer::Token;
use chumsky::prelude::*;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::HashMap};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum Expression {
    Atomic(AtomicExpr),
    Compound(CompoundExpr),
}

impl Expression {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        choice((
            AtomicExpr::parser().map(Self::Atomic),
            CompoundExpr::parser().map(Self::Compound),
        ))
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum AtomicExpr {
    ConstNil,
    ConstBool(bool),
    ConstChar(char),
    ConstInt(u64),
    ConstFloat(f64),
    ConstStr(String),

    Path(PathExpr),
    RefPath(PathExpr),
    Variable(Ident),
    FnCall(FnCallExpr),
    Paren(Box<Expression>),
    Intrinsic(IntrinExpr),

    SizeOf(TypeSpec),
    BorrowRead(Ident),
    BorrowWrite(Ident),
    ReBorrow(Ident),
    Deref(Box<Self>),
}

impl AtomicExpr {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        recursive(|parser| {
            choice((
                literal_nil().to(Self::ConstNil),
                literal_bool().map(Self::ConstBool),
                literal_char().map(Self::ConstChar),
                literal_u64().map(Self::ConstInt),
                literal_f64().map(Self::ConstFloat),
                literal_str().map(Cow::into_owned).map(Self::ConstStr),
                PathExpr::parser().map(Self::Path),
                PathExpr::parser()
                    .delimited_by(just(Token::RefTransform), just(Token::RParen))
                    .map(Self::RefPath),
                Ident::parser().map(Self::Variable),
                FnCallExpr::parser().map(Self::FnCall),
                Expression::parser()
                    .boxed()
                    .delimited_by(just(Token::LParen), just(Token::RParen))
                    .map(Box::new)
                    .map(Self::Paren),
                IntrinExpr::parser().map(Self::Intrinsic),
                just(Token::SizeOf)
                    .ignore_then(
                        TypeSpec::parser().delimited_by(just(Token::LParen), just(Token::RParen)),
                    )
                    .map(Self::SizeOf),
                just(Token::BorrowRead)
                    .ignore_then(Ident::parser())
                    .map(Self::BorrowRead),
                just(Token::BorrowWrite)
                    .ignore_then(Ident::parser())
                    .map(Self::BorrowWrite),
                just(Token::ReBorrow)
                    .ignore_then(Ident::parser())
                    .map(Self::ReBorrow),
                just(Token::Deref)
                    .ignore_then(parser)
                    .map(Box::new)
                    .map(Self::Deref),
            ))
        })
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum CompoundExpr {
    Cmp(CmpExpr),
    Logic(LogicExpr),
    Arith(ArithExpr),
    Select(SelectExpr),
    Cast(CastExpr),
}

impl CompoundExpr {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        choice((
            CmpExpr::parser().map(Self::Cmp),
            LogicExpr::parser().map(Self::Logic),
            ArithExpr::parser().map(Self::Arith),
            SelectExpr::parser().map(Self::Select),
            CastExpr::parser().map(Self::Cast),
        ))
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum CmpExpr {
    Eq(AtomicExpr, AtomicExpr),
    NotEq(AtomicExpr, AtomicExpr),
    Lt(AtomicExpr, AtomicExpr),
    LtEq(AtomicExpr, AtomicExpr),
    Gt(AtomicExpr, AtomicExpr),
    GtEq(AtomicExpr, AtomicExpr),
}

impl CmpExpr {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        group((
            AtomicExpr::parser(),
            choice((
                just(Token::Eq),
                just(Token::NotEq),
                just(Token::Lt),
                just(Token::LtEq),
                just(Token::Gt),
                just(Token::GtEq),
            )),
            AtomicExpr::parser(),
        ))
        .map(|(lhs, op, rhs)| match op {
            Token::Eq => Self::Eq(lhs, rhs),
            Token::NotEq => Self::NotEq(lhs, rhs),
            Token::Lt => Self::Lt(lhs, rhs),
            Token::LtEq => Self::LtEq(lhs, rhs),
            Token::Gt => Self::Gt(lhs, rhs),
            Token::GtEq => Self::GtEq(lhs, rhs),
            _ => unreachable!(),
        })
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum LogicExpr {
    And(AtomicExpr, AtomicExpr),
    Or(AtomicExpr, AtomicExpr),
    Not(AtomicExpr),
}

impl LogicExpr {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        choice((
            group((
                AtomicExpr::parser(),
                choice((just(Token::And), just(Token::Or))),
                AtomicExpr::parser(),
            ))
            .map(|(lhs, op, rhs)| match op {
                Token::And => Self::And(lhs, rhs),
                Token::Or => Self::Or(lhs, rhs),
                _ => unreachable!(),
            }),
            just(Token::Not)
                .ignore_then(AtomicExpr::parser())
                .map(Self::Not),
        ))
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum ArithExpr {
    Add(AtomicExpr, AtomicExpr),
    Sub(AtomicExpr, AtomicExpr),
    Mul(AtomicExpr, AtomicExpr),
    Div(AtomicExpr, AtomicExpr),
    Neg(AtomicExpr),
}

impl ArithExpr {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        choice((
            group((
                AtomicExpr::parser(),
                choice((
                    just(Token::Add),
                    just(Token::Sub),
                    just(Token::Mul),
                    just(Token::Div),
                )),
                AtomicExpr::parser(),
            ))
            .map(|(lhs, op, rhs)| match op {
                Token::Add => Self::Add(lhs, rhs),
                Token::Sub => Self::Sub(lhs, rhs),
                Token::Mul => Self::Mul(lhs, rhs),
                Token::Div => Self::Div(lhs, rhs),
                _ => unreachable!(),
            }),
            just(Token::Sub)
                .ignore_then(AtomicExpr::parser())
                .map(Self::Neg),
        ))
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct SelectExpr {
    pub condition: Box<Expression>,
    pub value_true: Box<Expression>,
    pub value_false: Box<Expression>,
}

impl SelectExpr {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        group((
            just(Token::If)
                .ignore_then(Expression::parser())
                .map(Box::new),
            just(Token::Then)
                .ignore_then(Expression::parser())
                .map(Box::new),
            just(Token::Else)
                .ignore_then(Expression::parser())
                .map(Box::new),
        ))
        .boxed()
        .map(|(condition, value_true, value_false)| Self {
            condition,
            value_true,
            value_false,
        })
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct CastExpr {
    pub value: AtomicExpr,
    pub r#type: TypeSpec,
}

impl CastExpr {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        AtomicExpr::parser()
            .then_ignore(just(Token::Colon))
            .then(TypeSpec::parser())
            .map(|(value, r#type)| Self { value, r#type })
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct PathExpr {
    pub first: Ident,
    pub extra: Vec<PathSegment>,
}

impl PathExpr {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        Ident::parser()
            .then(PathSegment::parser().repeated().collect::<Vec<_>>())
            .map(|(first, extra)| Self { first, extra })
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum PathSegment {
    SlotAccess(Ident),
    PtrSlotAccess(Ident),
    ArrayIndex(Box<Expression>),
}

impl PathSegment {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        choice((
            just(Token::Period)
                .ignore_then(Ident::parser())
                .map(Self::SlotAccess),
            just(Token::HypenRight)
                .ignore_then(Ident::parser())
                .map(Self::PtrSlotAccess),
            Expression::parser()
                .delimited_by(just(Token::LParen), just(Token::RParen))
                .map(Box::new)
                .map(Self::ArrayIndex),
        ))
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FnCallExpr {
    pub target: Ident,
    pub args: FnCallArgs,
}

impl FnCallExpr {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        Ident::parser()
            .then(FnCallArgs::parser().delimited_by(just(Token::LParen), just(Token::RParen)))
            .map(|(target, args)| Self { target, args })
    }
}

#[derive(Clone, Debug, Deserialize, Default, PartialEq, Serialize)]
pub enum FnCallArgs {
    #[default]
    Empty,
    Positional(Vec<Expression>),
    Named(HashMap<Ident, Expression>),
}

impl FnCallArgs {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        choice((
            Ident::parser()
                .then_ignore(just(Token::ArrowRight))
                .then(Expression::parser())
                .separated_by(just(Token::Comma))
                .allow_trailing()
                .collect::<HashMap<_, _>>()
                .map(Self::Named),
            Expression::parser()
                .separated_by(just(Token::Comma))
                .allow_trailing()
                .collect::<Vec<_>>()
                .map(Self::Positional),
        ))
        .or_not()
        .map(Option::unwrap_or_default)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum IntrinExpr {
    Embed {
        r#type: TypeSpec,
        exp: String,
        args: Vec<Expression>,
    },
}

impl IntrinExpr {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        just(Token::Embed)
            .ignore_then(group((
                TypeSpec::parser(),
                just(Token::Comma).ignore_then(literal_str().map(Cow::into_owned)),
                just(Token::Comma)
                    .ignore_then(
                        Expression::parser()
                            .separated_by(just(Token::Comma))
                            .allow_trailing()
                            .collect::<Vec<_>>(),
                    )
                    .or_not()
                    .map(Option::unwrap_or_default),
            )))
            .map(|(r#type, exp, args)| Self::Embed { r#type, exp, args })
    }
}
