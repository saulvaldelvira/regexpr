use core::error::Error;
use core::fmt::Display;

use alloc::borrow::Cow;

#[derive(Debug)]
pub struct RegexError(Cow<'static,str>);

impl From<&'static str> for RegexError {
    fn from(value: &'static str) -> Self {
        RegexError(value.into())
    }
}

impl From<String> for RegexError {
    fn from(value: String) -> Self {
        RegexError(value.into())
    }
}

impl From<Cow<'static,str>> for RegexError {
    fn from(value: Cow<'static,str>) -> Self {
        RegexError(value)
    }
}

impl Display for RegexError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for RegexError { }
