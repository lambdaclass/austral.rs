use std::{num::NonZeroUsize, ops::Range, path::PathBuf};

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SpanContext {
    pub path: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SpanLocation {
    pub line: NonZeroUsize,
    pub column: NonZeroUsize,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Span {
    pub context: SpanContext,
    pub range: Range<SpanLocation>,
}

impl chumsky::span::Span for Span {
    type Context = SpanContext;
    type Offset = SpanLocation;

    fn new(context: Self::Context, range: Range<Self::Offset>) -> Self {
        Self { context, range }
    }

    fn context(&self) -> Self::Context {
        self.context.clone()
    }

    fn start(&self) -> Self::Offset {
        self.range.start
    }

    fn end(&self) -> Self::Offset {
        self.range.end
    }
}
