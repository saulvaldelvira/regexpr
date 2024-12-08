//! # Regular Expressions
//! This crate provides a [Regex] struct that compiles and
//! tests regular expressions.
//!
//! ## Example
//! ```rust
//! use regexpr::Regex;
//!
//! let regex = Regex::compile("abc.*").unwrap();
//! assert!(regex.test("abc"));
//! assert!(regex.test("abcde"));
//! assert!(!regex.test("bcdsd"));
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
//!  | (ABC) | Groups rules A B and C |
//!  | \\c | Escapes the character c[^esc] |
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
//!

#![deny(
    clippy::unwrap_used,
    clippy::panic,
    clippy::expect_used,
    unused_must_use
)]
#![warn(clippy::pedantic)]

use std::borrow::Cow;
use std::fmt::Display;
use std::iter::FusedIterator;
use std::str::{CharIndices, Chars};

#[derive(Clone,Debug)]
enum MatchCase {
    Start,
    End,
    Char(char),
    List(Box<[MatchCase]>),
    Or(Box<[MatchCase]>),
    AnyOne,
    Opt(Box<MatchCase>),
    OneOrMore {
        case: Box<MatchCase>,
        lazy: bool,
    },
    Star{
        case: Box<MatchCase>,
        lazy: bool
    },
    Between(char,char),
    CharMatch(Box<[MatchCase]>),
    RangeLoop{
        case: Box<MatchCase>,
        min: Option<usize>,
        max: Option<usize>,
    },
    Not(Box<MatchCase>),
}

fn all_match(nc: &mut CharIndices<'_>, matches: &[MatchCase]) -> bool {
    for i in 0..matches.len() {
        let following = matches.get(i+1..).unwrap_or(&[]);
        if !matches[i].matches(nc, following) {
            return false
        }
    }
    true
}

impl MatchCase {
    fn star_loop(&self, nc: &mut CharIndices<'_>, lazy: bool, following: &[MatchCase]) -> bool {
        macro_rules! iter {
            () => {
                {
                    let mut it = nc.clone();
                    if self.matches(&mut it, following) {
                        *nc = it;
                    } else {
                        return true
                    }
                }
            };
        }
        if lazy {
            loop {
                if !following.is_empty() && all_match(&mut nc.clone(), following) {
                    return true
                }
                iter!();
            }
        } else {
            let mut last_next_match = None;

            loop {
                if all_match(&mut nc.clone(), following) {
                    last_next_match = Some(nc.clone());
                }

                let mut it = nc.clone();
                if self.matches(&mut it, following) {
                    *nc = it;
                } else {
                    if let Some(it) = last_next_match {
                        *nc = it;
                    }
                    return true
                }

            }
        }
    }
    fn matches(&self, nc: &mut CharIndices<'_>, following: &[MatchCase]) -> bool {
       macro_rules! next {
           () => {
               {
                   let Some((_, ch)) = nc.next() else { return false };
                   ch
               }
           };
       }

        match self {
            MatchCase::Char(expected) => {
                next!() == *expected
            },
            MatchCase::List(l) => {
                l.iter().all(|rule| {
                    rule.matches(nc, following)
                })
            },
            MatchCase::Or(l) => {
                l.iter().any(|rule| {
                    let mut newit = nc.clone();
                    let ret = rule.matches(&mut newit, following);
                    if ret { *nc = newit }
                    ret
                })
            },
            MatchCase::Opt(c) => {
                let mut newit = nc.clone();
                if c.matches(&mut newit, following) {
                    *nc = newit;
                }
                true
            },
            MatchCase::AnyOne => nc.next().is_some(),
            MatchCase::OneOrMore { case, lazy } => {
                if !case.matches(nc, following) {
                    return false;
                }
                case.star_loop(nc, *lazy, following)
            },
            MatchCase::Star { case, lazy } => {
                case.star_loop(nc, *lazy, following)
            },
            MatchCase::Start => nc.offset() == 0,
            MatchCase::End => nc.next().is_none(),
            MatchCase::Between(start,end) => {
                let c = next!();
                c >= *start && c <= *end
            },
            MatchCase::Not(match_case) => {
                match nc.clone().next() {
                    Some(_) => !match_case.matches(nc, following),
                    None => false,
                }
            },
            MatchCase::CharMatch(cases) => {
                let ret = cases.iter().any(|case| {
                    case.matches(&mut nc.clone(), following)
                });
                nc.next();
                ret
            },
            MatchCase::RangeLoop { case, min, max } => {
                let mut n = 0;

                if let Some(min) = min {
                    for _ in 0..*min {
                        if !case.matches(nc, following) {
                            return false;
                        }
                        n += 1;
                    }
                }

                loop {
                    if max.is_some_and(|max| n > max) {
                        return false;
                    }

                    let mut it = nc.clone();
                    if case.matches(&mut it, following) {
                        *nc = it;
                    } else {
                        break
                    }

                    n += 1;
                }

                true
            },
        }
    }
}

/// Main Regex struct
///
/// Holds a regular expression
pub struct Regex {
    matches: Box<[MatchCase]>,
}

impl Display for Regex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

type OrList = Vec<MatchCase>;
type RegexCompilerScope = (Vec<MatchCase>, Option<OrList>);

struct RegexCompiler<'a> {
    chars: Chars<'a>,
    open: usize,
    accc: Vec<RegexCompilerScope>,
}

impl<'a> RegexCompiler<'a> {
    fn new(src: &'a str) -> Self {
        let mut compiler = RegexCompiler {
            chars: src.chars(),
            open: 0,
            accc: Vec::new(),
        };
        compiler.enter_scope();
        compiler
    }
    fn enter_scope(&mut self) {
        self.open += 1;
        self.accc.push((Vec::new(), None));
    }
    fn close_scope(&mut self) -> MatchCase {
        self.open -= 1;

        match self.accc.pop() {
            Some((acc, orlist)) => {
                match orlist {
                    Some(mut orl) => {
                        orl.push(MatchCase::List(acc.into_boxed_slice()));
                        MatchCase::Or(orl.into_boxed_slice())
                    },
                    None => {
                        MatchCase::List(acc.into_boxed_slice())

                    }
                }
            },
            None => unreachable!(),
        }
    }
    fn last_acc(&mut self) -> &mut RegexCompilerScope {
        self.accc.last_mut().unwrap_or_else(|| {
            unreachable!()
        })
    }
    #[allow(clippy::too_many_lines)]
    fn process(&mut self) -> Result<Regex,Cow<'static,str>> {
        while let Some(c) = self.chars.next() {
            macro_rules! next {
                () => {
                    {
                        let c = self.chars.next().ok_or_else(|| format!("Expected character after {c}"))?;
                        c
                    }
                };
            }
            let newcase = match c {
                '.' => MatchCase::AnyOne,
                '\\' => {
                    MatchCase::Char(next!())
                },
                '(' => {
                    self.enter_scope();
                    continue;
                },
                ')' => {
                    self.close_scope()
                }
                '|' => {
                    match self.accc.pop() {
                        Some((mut acc, mut opt)) => {
                            let m = if acc.len() > 1 {
                                MatchCase::List(acc.into_boxed_slice())
                            } else {
                                acc.remove(0)
                            };
                            opt.get_or_insert_with(Vec::new).push(m);
                            self.accc.push((Vec::new(), opt));
                        },
                        None => unreachable!(),
                    };
                    continue;
                },
                '[' => {
                    let mut curr = next!();
                    let negated = curr == '^';
                    if negated {
                        curr = next!();
                    }

                    let mut list = Vec::new();

                    while curr != ']' {
                        if curr == '\\' {
                            curr = next!();
                        }
                        let c = curr;
                        curr = next!();

                        if curr == '-' {
                            let end = next!();
                            if end == ']' {
                                return Err("Expectend end of range [.. - ..]".into());
                            }
                            list.push(MatchCase::Between(c,end));
                            curr = next!();
                        } else {
                            list.push(MatchCase::Char(c));
                        }
                    }

                    let match_case = list.into_boxed_slice();
                    if negated {
                        MatchCase::Not(Box::new(MatchCase::CharMatch(match_case)))
                    } else {
                        MatchCase::CharMatch(match_case)
                    }
                },
                '{' => {
                    let last = self.last_acc().0.pop()
                               .ok_or_else(|| format!("Expected pattern before '{c}'"))?;

                    /* a{100,1000} */

                    let i = self.chars.as_str().find('}').ok_or("Missing closing '}'")?;
                    let slice = &self.chars.as_str()[..i];
                    let mut split = slice.split(',');
                    let min = split.next().ok_or("Range must be split by ','. Ex: {12,15}")?;
                    let max = split.next().ok_or("Range must be split by ','. Ex: {12,15}")?;

                    let min = if min.is_empty() { None } else { Some(min.parse().ok().ok_or("Error parsing number")?) };
                    let max = if max.is_empty() { None } else { Some(max.parse().ok().ok_or("Error parsing number")?) };

                    for _ in 0..=i {
                        self.chars.next();
                    }

                    MatchCase::RangeLoop { case: Box::new(last), min, max }
                },
                '?' | '*' | '+' => {
                    let last = self.last_acc().0.pop()
                               .ok_or_else(|| format!("Expected pattern before '{c}'"))?;
                    let last = Box::new(last);

                    let lazy = self.chars.clone().next().is_some_and(|c| c == '?');
                    if lazy { self.chars.next(); }

                    match c {
                        '?' => MatchCase::Opt(last),
                        '+' => MatchCase::OneOrMore { case: last, lazy },
                        '*' => MatchCase::Star { case: last, lazy },
                        _ => unreachable!()
                    }
                },
                '^' => MatchCase::Start,
                '$' => MatchCase::End,
                c => MatchCase::Char(c),
            };
            self.append(newcase);
        };
        let matches = match self.close_scope() {
            MatchCase::List(l) =>  l,
            MatchCase::Or(l) => Box::from([MatchCase::Or(l)]),
            _ => unreachable!()
        };

        Ok(Regex { matches })
    }
    fn append(&mut self, case: MatchCase) {
        if self.accc.is_empty() {
            self.accc.push((Vec::new(), None));
        }
        self.last_acc().0.push(case);
    }
}

/// Represents a match of a string on a [Regex]
#[derive(Debug)]
pub struct RegexMatch<'a> {
    start: usize,
    slice: &'a str,
    len: usize,
}

impl RegexMatch<'_> {
    /// Gets the span of the string where it matched the [Regex]
    #[must_use]
    pub fn get_span(&self) -> (usize,usize) {
        let o = self.start;
        (o, o + self.len)
    }
    /// Gets the slice of the string that matched the [Regex]
    ///
    /// This is the same as calling ``get_span``
    /// and then using it to slice the source string
    #[must_use]
    pub fn get_slice(&self) -> &str { self.slice }
}

impl Display for RegexMatch<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (s,e) = self.get_span();
        write!(f, "[{s},{e}]: \"{}\"", self.get_slice())
    }
}

/// Iterator over all the matches of a string in a [Regex]
#[derive(Debug,Clone)]
pub struct RegexMatcher<'a> {
    matches: &'a [MatchCase],
    start: CharIndices<'a>,
    first: bool,
}

impl<'a> Iterator for RegexMatcher<'a> {
    type Item = RegexMatch<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start.as_str().is_empty() && !self.first {
            return None;
        }
        self.first = false;

        let mut chars = self.start.clone();
        if !all_match(&mut chars, self.matches) {
            return match self.start.next() {
                Some(_) => self.next(),
                None => None,
            };
        }

        let start = self.start.offset();
        let end = chars.offset();

        let len = end - start;
        let slice = &self.start.as_str()[..len];

        self.start = chars;

        if self.matches.is_empty() {
            self.start.next();
        }

        Some( RegexMatch { start, slice, len } )
    }
}

impl FusedIterator for RegexMatcher<'_> { }

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
        RegexMatcher {
            matches: &self.matches,
            start: src.char_indices(),
            first: true,
        }
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
