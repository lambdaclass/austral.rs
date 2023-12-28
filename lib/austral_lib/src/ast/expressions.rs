use super::{
    literal_bool, literal_char, literal_f64, literal_nil, literal_str, literal_u64, Extra, Ident,
    TypeSpec,
};
use crate::lexer::Token;
use chumsky::{prelude::*, recursive::Indirect};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, cell::OnceCell, collections::HashMap, rc::Rc};

#[derive(Default)]
struct ParserCache<'a, 'b> {
    pub expression: OnceCell<Recursive<Indirect<'a, 'b, &'a [Token<'a>], Expression, Extra<'a>>>>,
    pub atomic_expr: OnceCell<Recursive<Indirect<'a, 'b, &'a [Token<'a>], AtomicExpr, Extra<'a>>>>,
    pub compound_expr:
        OnceCell<Recursive<Indirect<'a, 'b, &'a [Token<'a>], CompoundExpr, Extra<'a>>>>,
    pub cmp_expr: OnceCell<Recursive<Indirect<'a, 'b, &'a [Token<'a>], CmpExpr, Extra<'a>>>>,
    pub logic_expr: OnceCell<Recursive<Indirect<'a, 'b, &'a [Token<'a>], LogicExpr, Extra<'a>>>>,
    pub arith_expr: OnceCell<Recursive<Indirect<'a, 'b, &'a [Token<'a>], ArithExpr, Extra<'a>>>>,
    pub select_expr: OnceCell<Recursive<Indirect<'a, 'b, &'a [Token<'a>], SelectExpr, Extra<'a>>>>,
    pub cast_expr: OnceCell<Recursive<Indirect<'a, 'b, &'a [Token<'a>], CastExpr, Extra<'a>>>>,
    pub path_expr: OnceCell<Recursive<Indirect<'a, 'b, &'a [Token<'a>], PathExpr, Extra<'a>>>>,
    pub path_segment:
        OnceCell<Recursive<Indirect<'a, 'b, &'a [Token<'a>], PathSegment, Extra<'a>>>>,
    pub fn_call_expr: OnceCell<Recursive<Indirect<'a, 'b, &'a [Token<'a>], FnCallExpr, Extra<'a>>>>,
    pub fn_call_args: OnceCell<Recursive<Indirect<'a, 'b, &'a [Token<'a>], FnCallArgs, Extra<'a>>>>,
    pub intrin_expr: OnceCell<Recursive<Indirect<'a, 'b, &'a [Token<'a>], IntrinExpr, Extra<'a>>>>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum Expression {
    Atomic(AtomicExpr),
    Compound(CompoundExpr),
}

impl Expression {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self, Extra<'a>> {
        let cache = Rc::new(ParserCache::default());
        Self::recursive_parser(cache)
    }

    fn recursive_parser<'a>(
        cache: Rc<ParserCache<'a, 'a>>,
    ) -> impl Clone + Parser<'a, &'a [Token<'a>], Self, Extra<'a>> {
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

    FnCall(FnCallExpr),
    Path(PathExpr),
    RefPath(PathExpr),
    Paren(Box<Expression>),
    Intrinsic(IntrinExpr),

    SizeOf(TypeSpec),
    BorrowRead(Ident),
    BorrowWrite(Ident),
    ReBorrow(Ident),
    Deref(Box<Self>),
}

impl AtomicExpr {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self, Extra<'a>> {
        let cache = Rc::new(ParserCache::default());
        Self::recursive_parser(cache)
    }

    fn recursive_parser<'a>(
        cache: Rc<ParserCache<'a, 'a>>,
    ) -> impl Clone + Parser<'a, &'a [Token<'a>], Self, Extra<'a>> {
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
                    FnCallExpr::recursive_parser(cache.clone()).map(Self::FnCall),
                    PathExpr::recursive_parser(cache.clone()).map(Self::Path),
                    PathExpr::recursive_parser(cache.clone())
                        .delimited_by(just(Token::RefTransform), just(Token::RParen))
                        .map(Self::RefPath),
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
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self, Extra<'a>> {
        let cache = Rc::new(ParserCache::default());
        Self::recursive_parser(cache)
    }

    fn recursive_parser<'a>(
        cache: Rc<ParserCache<'a, 'a>>,
    ) -> impl Clone + Parser<'a, &'a [Token<'a>], Self, Extra<'a>> {
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
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self, Extra<'a>> {
        todo()
    }

    fn recursive_parser<'a>(
        cache: Rc<ParserCache<'a, 'a>>,
    ) -> impl Clone + Parser<'a, &'a [Token<'a>], Self, Extra<'a>> {
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
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self, Extra<'a>> {
        let cache = Rc::new(ParserCache::default());
        Self::recursive_parser(cache)
    }

    fn recursive_parser<'a>(
        cache: Rc<ParserCache<'a, 'a>>,
    ) -> impl Clone + Parser<'a, &'a [Token<'a>], Self, Extra<'a>> {
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
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self, Extra<'a>> {
        let cache = Rc::new(ParserCache::default());
        Self::recursive_parser(cache)
    }

    fn recursive_parser<'a>(
        cache: Rc<ParserCache<'a, 'a>>,
    ) -> impl Clone + Parser<'a, &'a [Token<'a>], Self, Extra<'a>> {
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
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self, Extra<'a>> {
        let cache = Rc::new(ParserCache::default());
        Self::recursive_parser(cache)
    }

    fn recursive_parser<'a>(
        cache: Rc<ParserCache<'a, 'a>>,
    ) -> impl Clone + Parser<'a, &'a [Token<'a>], Self, Extra<'a>> {
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
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self, Extra<'a>> {
        let cache = Rc::new(ParserCache::default());
        Self::recursive_parser(cache)
    }

    fn recursive_parser<'a>(
        cache: Rc<ParserCache<'a, 'a>>,
    ) -> impl Clone + Parser<'a, &'a [Token<'a>], Self, Extra<'a>> {
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
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self, Extra<'a>> {
        let cache = Rc::new(ParserCache::default());
        Self::recursive_parser(cache)
    }

    fn recursive_parser<'a>(
        cache: Rc<ParserCache<'a, 'a>>,
    ) -> impl Clone + Parser<'a, &'a [Token<'a>], Self, Extra<'a>> {
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
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self, Extra<'a>> {
        let cache = Rc::new(ParserCache::default());
        Self::recursive_parser(cache)
    }

    fn recursive_parser<'a>(
        cache: Rc<ParserCache<'a, 'a>>,
    ) -> impl Clone + Parser<'a, &'a [Token<'a>], Self, Extra<'a>> {
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
                        .delimited_by(just(Token::LBracket), just(Token::RBracket))
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
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self, Extra<'a>> {
        let cache = Rc::new(ParserCache::default());
        Self::recursive_parser(cache)
    }

    fn recursive_parser<'a>(
        cache: Rc<ParserCache<'a, 'a>>,
    ) -> impl Clone + Parser<'a, &'a [Token<'a>], Self, Extra<'a>> {
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
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self, Extra<'a>> {
        let cache = Rc::new(ParserCache::default());
        Self::recursive_parser(cache)
    }

    fn recursive_parser<'a>(
        cache: Rc<ParserCache<'a, 'a>>,
    ) -> impl Clone + Parser<'a, &'a [Token<'a>], Self, Extra<'a>> {
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
                            .at_least(1)
                            .allow_trailing()
                            .collect::<HashMap<_, _>>()
                            .map(Self::Named),
                        Expression::recursive_parser(cache.clone())
                            .separated_by(just(Token::Comma))
                            .at_least(1)
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
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self, Extra<'a>> {
        let cache = Rc::new(ParserCache::default());
        Self::recursive_parser(cache)
    }

    fn recursive_parser<'a>(
        cache: Rc<ParserCache<'a, 'a>>,
    ) -> impl Clone + Parser<'a, &'a [Token<'a>], Self, Extra<'a>> {
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

#[cfg(test)]
mod expressions_parser_tests {
    use super::*;
    use crate::lexer::Token;
    use std::{borrow::Cow, vec};

    /// Test that we can parse constant expressions like:
    ///
    /// ConstNil,
    /// ConstBool(bool),
    /// ConstChar(char),
    /// ConstInt(u64),
    /// ConstFloat(f64),
    /// ConstStr(String),
    #[test]
    fn test_const_expressions() {
        let nil = vec![Token::Nil];
        assert_eq!(
            AtomicExpr::parser().parse(&nil).unwrap(),
            AtomicExpr::ConstNil
        );

        let true_ = vec![Token::True];
        assert_eq!(
            AtomicExpr::parser().parse(&true_).unwrap(),
            AtomicExpr::ConstBool(true)
        );

        let false_ = vec![Token::False];
        assert_eq!(
            AtomicExpr::parser().parse(&false_).unwrap(),
            AtomicExpr::ConstBool(false)
        );

        let char_ = vec![Token::Char('a')];
        assert_eq!(
            AtomicExpr::parser().parse(&char_).unwrap(),
            AtomicExpr::ConstChar('a')
        );

        let int_ = vec![Token::Decimal(10)];
        assert_eq!(
            AtomicExpr::parser().parse(&int_).unwrap(),
            AtomicExpr::ConstInt(10)
        );

        let float_ = vec![Token::Float(10.0)];
        assert_eq!(
            AtomicExpr::parser().parse(&float_).unwrap(),
            AtomicExpr::ConstFloat(10.0)
        );

        let str_ = vec![Token::String(Cow::Borrowed("hello world"))];
        assert_eq!(
            AtomicExpr::parser().parse(&str_).unwrap(),
            AtomicExpr::ConstStr("hello world".to_string())
        );
    }

    #[test]
    fn test_fn_call_expression() {
        let fn_call_noargs = vec![Token::Ident("foo"), Token::LParen, Token::RParen];

        assert_eq!(
            FnCallExpr::parser().parse(&fn_call_noargs).unwrap(),
            FnCallExpr {
                target: Ident::new("foo"),
                args: FnCallArgs::Empty
            }
        );

        let fn_call = vec![
            Token::Ident("foo"),
            Token::LParen,
            Token::Ident("bar"),
            Token::RParen,
        ];
        assert_eq!(
            FnCallExpr::parser().parse(&fn_call).unwrap(),
            FnCallExpr {
                target: Ident::new("foo"),
                args: FnCallArgs::Positional(vec![Expression::Atomic(AtomicExpr::Path(
                    PathExpr {
                        first: Ident::new("bar"),
                        extra: vec![]
                    }
                ))])
            }
        );
    }

    #[test]
    fn test_path_expression() {
        let path = vec![Token::Ident("foo"), Token::Period, Token::Ident("bar")];
        assert_eq!(
            PathExpr::parser().parse(&path).unwrap(),
            PathExpr {
                first: Ident::new("foo"),
                extra: vec![PathSegment::SlotAccess(Ident::new("bar"))]
            }
        );
    }

    #[test]
    fn test_arith_expression() {
        let add_expr = vec![Token::Decimal(10), Token::Add, Token::Decimal(10)];

        assert_eq!(
            ArithExpr::parser().parse(&add_expr).unwrap(),
            ArithExpr::Add(AtomicExpr::ConstInt(10), AtomicExpr::ConstInt(10))
        );

        let sub_expr = vec![Token::Decimal(10), Token::Sub, Token::Decimal(10)];

        assert_eq!(
            ArithExpr::parser().parse(&sub_expr).unwrap(),
            ArithExpr::Sub(AtomicExpr::ConstInt(10), AtomicExpr::ConstInt(10))
        );

        let mul_expr = vec![Token::Decimal(10), Token::Mul, Token::Decimal(10)];

        assert_eq!(
            ArithExpr::parser().parse(&mul_expr).unwrap(),
            ArithExpr::Mul(AtomicExpr::ConstInt(10), AtomicExpr::ConstInt(10))
        );

        let div_expr = vec![Token::Decimal(10), Token::Div, Token::Decimal(10)];

        assert_eq!(
            ArithExpr::parser().parse(&div_expr).unwrap(),
            ArithExpr::Div(AtomicExpr::ConstInt(10), AtomicExpr::ConstInt(10))
        );
    }

    #[test]
    fn test_logical_expressions() {
        let and_expr = vec![Token::True, Token::And, Token::False];

        assert_eq!(
            LogicExpr::parser().parse(&and_expr).unwrap(),
            LogicExpr::And(AtomicExpr::ConstBool(true), AtomicExpr::ConstBool(false))
        );

        let or_expr = vec![Token::True, Token::Or, Token::False];

        assert_eq!(
            LogicExpr::parser().parse(&or_expr).unwrap(),
            LogicExpr::Or(AtomicExpr::ConstBool(true), AtomicExpr::ConstBool(false))
        );

        let not_expr = vec![Token::Not, Token::True];

        assert_eq!(
            LogicExpr::parser().parse(&not_expr).unwrap(),
            LogicExpr::Not(AtomicExpr::ConstBool(true))
        );
    }
}
