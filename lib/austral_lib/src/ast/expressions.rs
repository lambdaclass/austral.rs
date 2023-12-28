use super::{
    literal_bool, literal_char, literal_f64, literal_nil, literal_str, literal_u64, Ident, TypeSpec,
};
use crate::lexer::Token;
use chumsky::{prelude::*, recursive::Indirect};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, cell::OnceCell, collections::HashMap, rc::Rc};

struct ParserCache<'a, 'b> {
    pub expression:
        OnceCell<Recursive<Indirect<'a, 'b, &'a [Token<'a>], Expression, extra::Default>>>,
    pub atomic_expr:
        OnceCell<Recursive<Indirect<'a, 'b, &'a [Token<'a>], AtomicExpr, extra::Default>>>,
    pub compound_expr:
        OnceCell<Recursive<Indirect<'a, 'b, &'a [Token<'a>], CompoundExpr, extra::Default>>>,
    pub cmp_expr: OnceCell<Recursive<Indirect<'a, 'b, &'a [Token<'a>], CmpExpr, extra::Default>>>,
    pub logic_expr:
        OnceCell<Recursive<Indirect<'a, 'b, &'a [Token<'a>], LogicExpr, extra::Default>>>,
    pub arith_expr:
        OnceCell<Recursive<Indirect<'a, 'b, &'a [Token<'a>], ArithExpr, extra::Default>>>,
    pub select_expr:
        OnceCell<Recursive<Indirect<'a, 'b, &'a [Token<'a>], SelectExpr, extra::Default>>>,
    pub cast_expr: OnceCell<Recursive<Indirect<'a, 'b, &'a [Token<'a>], CastExpr, extra::Default>>>,
    pub path_expr: OnceCell<Recursive<Indirect<'a, 'b, &'a [Token<'a>], PathExpr, extra::Default>>>,
    pub path_segment:
        OnceCell<Recursive<Indirect<'a, 'b, &'a [Token<'a>], PathSegment, extra::Default>>>,
    pub fn_call_expr:
        OnceCell<Recursive<Indirect<'a, 'b, &'a [Token<'a>], FnCallExpr, extra::Default>>>,
    pub fn_call_args:
        OnceCell<Recursive<Indirect<'a, 'b, &'a [Token<'a>], FnCallArgs, extra::Default>>>,
    pub intrin_expr:
        OnceCell<Recursive<Indirect<'a, 'b, &'a [Token<'a>], IntrinExpr, extra::Default>>>,
}

impl<'a, 'b> Default for ParserCache<'a, 'b> {
    fn default() -> Self {
        Self {
            expression: Default::default(),
            atomic_expr: Default::default(),
            compound_expr: Default::default(),
            cmp_expr: Default::default(),
            logic_expr: Default::default(),
            arith_expr: Default::default(),
            select_expr: Default::default(),
            cast_expr: Default::default(),
            path_expr: Default::default(),
            path_segment: Default::default(),
            fn_call_expr: Default::default(),
            fn_call_args: Default::default(),
            intrin_expr: Default::default(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum Expression {
    Atomic(AtomicExpr),
    Compound(CompoundExpr),
}

impl Expression {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        let cache = Rc::new(ParserCache::default());
        Self::recursive_parser(cache)
    }

    fn recursive_parser<'a>(
        cache: Rc<ParserCache<'a, 'a>>,
    ) -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        match cache.expression.get() {
            Some(parser) => parser.clone(),
            None => {
                let mut parser = Recursive::declare();
                let _ = cache.expression.set(parser.clone());

                parser.define(choice((
                    AtomicExpr::recursive_parser(cache.clone()).map(Self::Atomic),
                    CompoundExpr::recursive_parser(cache.clone()).map(Self::Compound),
                )));
                parser
            }
        }
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
        let cache = Rc::new(ParserCache::default());
        Self::recursive_parser(cache)
    }

    fn recursive_parser<'a>(
        cache: Rc<ParserCache<'a, 'a>>,
    ) -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        match cache.atomic_expr.get() {
            Some(parser) => parser.clone(),
            None => {
                let mut parser = Recursive::declare();
                let _ = cache.atomic_expr.set(parser.clone());

                parser.define(choice((
                    literal_nil().to(Self::ConstNil),
                    literal_bool().map(Self::ConstBool),
                    literal_char().map(Self::ConstChar),
                    literal_u64().map(Self::ConstInt),
                    literal_f64().map(Self::ConstFloat),
                    literal_str().map(Cow::into_owned).map(Self::ConstStr),
                    PathExpr::recursive_parser(cache.clone()).map(Self::Path),
                    PathExpr::recursive_parser(cache.clone())
                        .delimited_by(just(Token::RefTransform), just(Token::RParen))
                        .map(Self::RefPath),
                    Ident::parser().map(Self::Variable),
                    FnCallExpr::recursive_parser(cache.clone()).map(Self::FnCall),
                    Expression::recursive_parser(cache.clone())
                        .boxed()
                        .delimited_by(just(Token::LParen), just(Token::RParen))
                        .map(Box::new)
                        .map(Self::Paren),
                    IntrinExpr::recursive_parser(cache.clone()).map(Self::Intrinsic),
                    just(Token::SizeOf)
                        .ignore_then(
                            TypeSpec::parser()
                                .delimited_by(just(Token::LParen), just(Token::RParen)),
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
                        .ignore_then(parser.clone())
                        .map(Box::new)
                        .map(Self::Deref),
                )));
                parser
            }
        }
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
        let cache = Rc::new(ParserCache::default());
        Self::recursive_parser(cache)
    }

    fn recursive_parser<'a>(
        cache: Rc<ParserCache<'a, 'a>>,
    ) -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        match cache.compound_expr.get() {
            Some(parser) => parser.clone(),
            None => {
                let mut parser = Recursive::declare();
                let _ = cache.compound_expr.set(parser.clone());

                parser.define(choice((
                    CmpExpr::recursive_parser(cache.clone()).map(Self::Cmp),
                    LogicExpr::recursive_parser(cache.clone()).map(Self::Logic),
                    ArithExpr::recursive_parser(cache.clone()).map(Self::Arith),
                    SelectExpr::recursive_parser(cache.clone()).map(Self::Select),
                    CastExpr::recursive_parser(cache.clone()).map(Self::Cast),
                )));
                parser
            }
        }
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
        todo()
    }

    fn recursive_parser<'a>(
        cache: Rc<ParserCache<'a, 'a>>,
    ) -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        match cache.cmp_expr.get() {
            Some(parser) => parser.clone(),
            None => {
                let mut parser = Recursive::declare();
                let _ = cache.cmp_expr.set(parser.clone());

                parser.define(
                    group((
                        AtomicExpr::recursive_parser(cache.clone()),
                        choice((
                            just(Token::Eq),
                            just(Token::NotEq),
                            just(Token::Lt),
                            just(Token::LtEq),
                            just(Token::Gt),
                            just(Token::GtEq),
                        )),
                        AtomicExpr::recursive_parser(cache.clone()),
                    ))
                    .map(|(lhs, op, rhs)| match op {
                        Token::Eq => Self::Eq(lhs, rhs),
                        Token::NotEq => Self::NotEq(lhs, rhs),
                        Token::Lt => Self::Lt(lhs, rhs),
                        Token::LtEq => Self::LtEq(lhs, rhs),
                        Token::Gt => Self::Gt(lhs, rhs),
                        Token::GtEq => Self::GtEq(lhs, rhs),
                        _ => unreachable!(),
                    }),
                );
                parser
            }
        }
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
        let cache = Rc::new(ParserCache::default());
        Self::recursive_parser(cache)
    }

    fn recursive_parser<'a>(
        cache: Rc<ParserCache<'a, 'a>>,
    ) -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        match cache.logic_expr.get() {
            Some(parser) => parser.clone(),
            None => {
                let mut parser = Recursive::declare();
                let _ = cache.logic_expr.set(parser.clone());

                parser.define(choice((
                    group((
                        AtomicExpr::recursive_parser(cache.clone()),
                        choice((just(Token::And), just(Token::Or))),
                        AtomicExpr::recursive_parser(cache.clone()),
                    ))
                    .map(|(lhs, op, rhs)| match op {
                        Token::And => Self::And(lhs, rhs),
                        Token::Or => Self::Or(lhs, rhs),
                        _ => unreachable!(),
                    }),
                    just(Token::Not)
                        .ignore_then(AtomicExpr::recursive_parser(cache.clone()))
                        .map(Self::Not),
                )));
                parser
            }
        }
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
        let cache = Rc::new(ParserCache::default());
        Self::recursive_parser(cache)
    }

    fn recursive_parser<'a>(
        cache: Rc<ParserCache<'a, 'a>>,
    ) -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        match cache.arith_expr.get() {
            Some(parser) => parser.clone(),
            None => {
                let mut parser = Recursive::declare();
                let _ = cache.arith_expr.set(parser.clone());

                parser.define(choice((
                    group((
                        AtomicExpr::recursive_parser(cache.clone()),
                        choice((
                            just(Token::Add),
                            just(Token::Sub),
                            just(Token::Mul),
                            just(Token::Div),
                        )),
                        AtomicExpr::recursive_parser(cache.clone()),
                    ))
                    .map(|(lhs, op, rhs)| match op {
                        Token::Add => Self::Add(lhs, rhs),
                        Token::Sub => Self::Sub(lhs, rhs),
                        Token::Mul => Self::Mul(lhs, rhs),
                        Token::Div => Self::Div(lhs, rhs),
                        _ => unreachable!(),
                    }),
                    just(Token::Sub)
                        .ignore_then(AtomicExpr::recursive_parser(cache.clone()))
                        .map(Self::Neg),
                )));
                parser
            }
        }
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
        let cache = Rc::new(ParserCache::default());
        Self::recursive_parser(cache)
    }

    fn recursive_parser<'a>(
        cache: Rc<ParserCache<'a, 'a>>,
    ) -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        match cache.select_expr.get() {
            Some(parser) => parser.clone(),
            None => {
                let mut parser = Recursive::declare();
                let _ = cache.select_expr.set(parser.clone());

                parser.define(
                    group((
                        just(Token::If)
                            .ignore_then(Expression::recursive_parser(cache.clone()))
                            .map(Box::new),
                        just(Token::Then)
                            .ignore_then(Expression::recursive_parser(cache.clone()))
                            .map(Box::new),
                        just(Token::Else)
                            .ignore_then(Expression::recursive_parser(cache.clone()))
                            .map(Box::new),
                    ))
                    .boxed()
                    .map(|(condition, value_true, value_false)| Self {
                        condition,
                        value_true,
                        value_false,
                    }),
                );
                parser
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct CastExpr {
    pub value: AtomicExpr,
    pub r#type: TypeSpec,
}

impl CastExpr {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        let cache = Rc::new(ParserCache::default());
        Self::recursive_parser(cache)
    }

    fn recursive_parser<'a>(
        cache: Rc<ParserCache<'a, 'a>>,
    ) -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        match cache.cast_expr.get() {
            Some(parser) => parser.clone(),
            None => {
                let mut parser = Recursive::declare();
                let _ = cache.cast_expr.set(parser.clone());

                parser.define(
                    AtomicExpr::recursive_parser(cache.clone())
                        .then_ignore(just(Token::Colon))
                        .then(TypeSpec::parser())
                        .map(|(value, r#type)| Self { value, r#type }),
                );
                parser
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct PathExpr {
    pub first: Ident,
    pub extra: Vec<PathSegment>,
}

impl PathExpr {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        let cache = Rc::new(ParserCache::default());
        Self::recursive_parser(cache)
    }

    fn recursive_parser<'a>(
        cache: Rc<ParserCache<'a, 'a>>,
    ) -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        match cache.path_expr.get() {
            Some(parser) => parser.clone(),
            None => {
                let mut parser = Recursive::declare();
                let _ = cache.path_expr.set(parser.clone());

                parser.define(
                    Ident::parser()
                        .then(
                            PathSegment::recursive_parser(cache.clone())
                                .repeated()
                                .collect::<Vec<_>>(),
                        )
                        .map(|(first, extra)| Self { first, extra }),
                );
                parser
            }
        }
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
        let cache = Rc::new(ParserCache::default());
        Self::recursive_parser(cache)
    }

    fn recursive_parser<'a>(
        cache: Rc<ParserCache<'a, 'a>>,
    ) -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        match cache.path_segment.get() {
            Some(parser) => parser.clone(),
            None => {
                let mut parser = Recursive::declare();
                let _ = cache.path_segment.set(parser.clone());

                parser.define(choice((
                    just(Token::Period)
                        .ignore_then(Ident::parser())
                        .map(Self::SlotAccess),
                    just(Token::HypenRight)
                        .ignore_then(Ident::parser())
                        .map(Self::PtrSlotAccess),
                    Expression::recursive_parser(cache.clone())
                        .delimited_by(just(Token::LParen), just(Token::RParen))
                        .map(Box::new)
                        .map(Self::ArrayIndex),
                )));
                parser
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FnCallExpr {
    pub target: Ident,
    pub args: FnCallArgs,
}

impl FnCallExpr {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        let cache = Rc::new(ParserCache::default());
        Self::recursive_parser(cache)
    }

    fn recursive_parser<'a>(
        cache: Rc<ParserCache<'a, 'a>>,
    ) -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        match cache.fn_call_expr.get() {
            Some(parser) => parser.clone(),
            None => {
                let mut parser = Recursive::declare();
                let _ = cache.fn_call_expr.set(parser.clone());

                parser.define(
                    Ident::parser()
                        .then(
                            FnCallArgs::recursive_parser(cache.clone())
                                .delimited_by(just(Token::LParen), just(Token::RParen)),
                        )
                        .map(|(target, args)| Self { target, args }),
                );
                parser
            }
        }
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
        let cache = Rc::new(ParserCache::default());
        Self::recursive_parser(cache)
    }

    fn recursive_parser<'a>(
        cache: Rc<ParserCache<'a, 'a>>,
    ) -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        match cache.fn_call_args.get() {
            Some(parser) => parser.clone(),
            None => {
                let mut parser = Recursive::declare();
                let _ = cache.fn_call_args.set(parser.clone());

                parser.define(
                    choice((
                        Ident::parser()
                            .then_ignore(just(Token::ArrowRight))
                            .then(Expression::recursive_parser(cache.clone()))
                            .separated_by(just(Token::Comma))
                            .allow_trailing()
                            .collect::<HashMap<_, _>>()
                            .map(Self::Named),
                        Expression::recursive_parser(cache.clone())
                            .separated_by(just(Token::Comma))
                            .allow_trailing()
                            .collect::<Vec<_>>()
                            .map(Self::Positional),
                    ))
                    .or_not()
                    .map(Option::unwrap_or_default),
                );
                parser
            }
        }
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
        let cache = Rc::new(ParserCache::default());
        Self::recursive_parser(cache)
    }

    fn recursive_parser<'a>(
        cache: Rc<ParserCache<'a, 'a>>,
    ) -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        match cache.intrin_expr.get() {
            Some(parser) => parser.clone(),
            None => {
                let mut parser = Recursive::declare();
                let _ = cache.intrin_expr.set(parser.clone());

                parser.define(
                    just(Token::Embed)
                        .ignore_then(group((
                            TypeSpec::parser(),
                            just(Token::Comma).ignore_then(literal_str().map(Cow::into_owned)),
                            just(Token::Comma)
                                .ignore_then(
                                    Expression::recursive_parser(cache.clone())
                                        .separated_by(just(Token::Comma))
                                        .allow_trailing()
                                        .collect::<Vec<_>>(),
                                )
                                .or_not()
                                .map(Option::unwrap_or_default),
                        )))
                        .map(|(r#type, exp, args)| Self::Embed { r#type, exp, args }),
                );
                parser
            }
        }
    }
}
