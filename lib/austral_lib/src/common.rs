pub enum Mutability {
    Immutable,
    Mutable,
}

pub enum BorrowingMode {
    ReadBorrow,
    WriteBorrow,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Identifier(String);

impl Identifier {
    pub fn new(s: &str) -> Self {
        Identifier(s.to_string())
    }
}

pub struct QIdent {
    pub source: ModuleName,
    pub original: Identifier,
    pub local: Identifier,
}

#[derive(Clone, Debug)]
pub struct ModuleName(String);

#[derive(Clone, Debug)]
pub struct DeclId(i32);

pub enum ComparisonOperator {
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
}

pub enum ArithmeticOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
}
