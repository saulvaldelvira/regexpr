use alloc::string::String;
use core::error::Error;
use core::fmt::Display;

use alloc::borrow::Cow;

#[derive(Debug)]
pub struct RegexError(Cow<'static, str>);

impl RegexError {
    #[inline]
    #[must_use]
    pub fn inner(&self) -> &Cow<'static, str> {
        &self.0
    }
}

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

impl From<Cow<'static, str>> for RegexError {
    fn from(value: Cow<'static, str>) -> Self {
        RegexError(value)
    }
}

impl From<RegexError> for Cow<'static,str> {
    fn from(val: RegexError) -> Self {
        val.0
    }
}

impl From<RegexError> for String {
    fn from(val: RegexError) -> Self {
        val.0.into_owned()
    }
}

impl Display for RegexError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for RegexError {}
