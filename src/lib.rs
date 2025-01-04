//! # Regular Expressions
//! This crate provides a [Regex] struct that compiles and
//! tests regular expressions.
//!
//! ## Example
//! ```rust
//! use regexpr::Regex;
//!
//! let regex = Regex::compile(r#"^a(.)c\1.*$"#).unwrap();
//! assert!(regex.test("abcb"));
//! assert!(regex.test("abcbde"));
//! assert!(!regex.test("bcdsd"));
//! assert!(!regex.test("abcd"));
//! ```
//!
//! # Rules
//!
//! | Rule  | Meaning |
//! |---------|---------|
//!  |  .   |  Matches any character |
//!  |  * | Matches the previous rule zero or more times |
//!  |  + | Matches the previous rule one or more times |
//!  |  ? | Makes the previous rule optional |
//!  | {n,m} | Matches the previous rule a minimun of n times and a maximun of m times[^min_max] |
//!  | \[a-z] | Matches any character from a to z[^ranged] |
//!  | \[agf] | Matches any of the characters inside |
//!  | \[^...] | Same as the rules above but negated |
//!  | A \| B | Maches A or B |
//!  | (ABC) | Groups rules A B and C [^group] |
//!  | \\c | Escapes the character c[^esc] |
//!  | __\\n__  _OR_ __\\k\<n\>__ | Match the n'th capture group[^capture] |
//!
//! [^min_max]: If min or max are not present, it means there's no limit on that size. \
//! Examples:\
//!     {,12} matches a rule up to 12 \
//!     {3,} matches a rule at least 3 times. \
//!     {,} is the same as *
//!
//! [^ranged]: The ranges can be mixed. \
//! Examples: \
//!     \[a-z123]: Matches any character in the ranges a-z , 1, 2 or 3 \
//!     \[^0-9ab]: Matches a character that IS NOT a number or a or b
//!
//! [^esc]: Example: "\\." Matches a literal dot character.
//!
//! [^group]: This captured groups can be later referenced
//!
//! [^capture]: n must be an integer in the range \[1,L\] where L is the number
//!             of capture groups in the expression
//!
//!
//!
//! ## Greedy vs. Lazy
//! "Lazy" versions of * and + exist. \
//! *? and +? work just as * and +, but they stop as soon as possible.
//!
//! ### Example
//!
//! ```text
//!     Regex: .*b
//!     Input: aaaaaabaaaaab
//!     Matches: One match "aaaaaabaaaaab"
//!
//!     Regex: .*?b
//!     Input: aaaaaabaaaaab
//!     Matches: Two matches "aaaaaab" and "aaaaab"
//! ```

#![deny(
    clippy::unwrap_used,
    clippy::panic,
    clippy::expect_used,
    unused_must_use
)]
#![warn(clippy::pedantic)]

#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
extern crate alloc;

use alloc::boxed::Box;
use alloc::borrow::Cow;

use core::fmt::Display;

mod case;
use case::MatchCase;

mod compiler;
use compiler::RegexCompiler;

mod matcher;

#[doc(inline)]
pub use matcher::{RegexMatch,RegexMatcher};

/// Main Regex struct
///
/// Holds a regular expression
#[derive(Debug)]
pub struct Regex {
    matches: Box<[MatchCase]>,
    n_captures: usize,
}

impl Display for Regex {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut first = true;
        for c in &self.matches {
            if !first {
                write!(f, " => ")?;
            }
            first = false;
            write!(f, "{c:#?}")?;
        }
        Ok(())
    }
}

impl Regex {
    /// Compile the given string into a [Regex]
    ///
    /// Returns error if the regex is invalid and fails to compile
    ///
    /// # Errors
    /// If the regex fails to compile, the error variant contains
    /// a message explaining the issue
    ///
    pub fn compile(src: &str) -> Result<Self,Cow<'static,str>> {
        RegexCompiler::new(src).process()
    }
    /// Returns an [Iterator] over all the [`matches`] of the [Regex] in the given string
    ///
    /// [`matches`]: RegexMatch
    #[must_use]
    pub fn find_matches<'a>(&'a self, src: &'a str) -> RegexMatcher<'a>  {
        RegexMatcher::new(src, &self.matches, self.n_captures)
    }
    /// Returns true if the regex matches the given string
    ///
    /// This is the same as calling ``find_matches``
    /// and then checking if the iterator contains at least one element
    #[must_use]
    pub fn test(&self, src: &str) -> bool {
        self.find_matches(src).next().is_some()
    }
}

impl TryFrom<&str> for Regex {
    type Error = Cow<'static,str>;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Regex::compile(value)
    }
}

/// This trait is used to add an extension method
/// ``matches_regex`` to any str-like object
pub trait RegexTestable {
    /// Returns true if it matches the given [Regex]
    fn matches_regex(&self, regex: impl AsRef<str>) -> bool;
}

impl<T: AsRef<str>> RegexTestable for T {
    fn matches_regex(&self, regex: impl AsRef<str>) -> bool {
        Regex::compile(regex.as_ref())
              .map(|regex| regex.test(self.as_ref()))
              .unwrap_or(false)
    }
}

#[cfg(test)]
mod test;

#[cfg(feature = "bindings")]
pub mod ffi;
