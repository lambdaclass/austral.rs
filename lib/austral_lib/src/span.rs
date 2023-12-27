pub struct Position {
    pub line: usize,
    pub column: usize,
}

pub struct Span {
    pub filename: String,
    pub startp: Position,
    pub endp: Position,
}
