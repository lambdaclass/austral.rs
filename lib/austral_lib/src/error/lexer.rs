use std::{borrow::Cow, ops::Range};
use thiserror::Error;

pub type Result<'a, T> = std::result::Result<T, Error<'a>>;

#[derive(Debug, Error)]
pub enum Error<'a> {
    #[error("Unexpected input: \"{0}\" at offset {}..{}", .1.start, .1.end)]
    UnexpectedInput(Cow<'a, str>, Range<usize>),
}
