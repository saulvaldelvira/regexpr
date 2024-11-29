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

use std::borrow::Cow;
use std::fmt::Display;
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
    OneOrMore(Box<MatchCase>),
    Star(Box<MatchCase>),
    Between(char,char),
    CharMatch(Box<[MatchCase]>),
    RangeLoop{
        case: Box<MatchCase>,
        min: Option<usize>,
        max: Option<usize>,
    },
    Not(Box<MatchCase>),
}

impl MatchCase {
    fn star_loop(&self, nc: &mut CharIndices<'_>) -> bool {
        loop {
            let mut it = nc.clone();
            if self.matches(&mut it) {
                *nc = it;
            } else {
                break
            }
        }
        true
    }
    fn matches(&self, nc: &mut CharIndices<'_>) -> bool {
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
                    rule.matches(nc)
                })
            },
            MatchCase::Or(l) => {
                l.iter().any(|rule| {
                    let mut newit = nc.clone();
                    let ret = rule.matches(&mut newit);
                    if ret { *nc = newit }
                    ret
                })
            },
            MatchCase::Opt(c) => {
                let mut newit = nc.clone();
                if c.matches(&mut newit) {
                    *nc = newit;
                }
                true
            },
            MatchCase::AnyOne => nc.next().is_some(),
            MatchCase::OneOrMore(match_case) => {
                if !match_case.matches(nc) {
                    return false;
                }
                match_case.star_loop(nc)
            },
            MatchCase::Star(match_case) => {
                match_case.star_loop(nc)
            },
            MatchCase::Start => nc.offset() == 0,
            MatchCase::End => nc.next().is_none(),
            MatchCase::Between(start,end) => {
                let c = next!();
                c >= *start && c <= *end
            },
            MatchCase::Not(match_case) => {
                match nc.clone().next() {
                    Some(_) => !match_case.matches(nc),
                    None => false,
                }
            },
            MatchCase::CharMatch(cases) => {
                let ret = cases.iter().any(|case| {
                    case.matches(&mut nc.clone())
                });
                nc.next();
                ret
            },
            MatchCase::RangeLoop { case, min, max } => {
                let mut n = 0;

                if let Some(min) = min {
                    for _ in 0..*min {
                        if !case.matches(nc) {
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
                    if case.matches(&mut it) {
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
        for c in self.matches.iter() {
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
                        Some((acc, mut opt)) => {
                            let m = if acc.len() > 1 {
                                MatchCase::List(acc.into_boxed_slice())
                            } else {
                                acc.into_iter().next().unwrap()
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
                    let last = self.accc.last_mut().unwrap().0.pop()
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
                    let last = self.accc.last_mut().unwrap().0.pop()
                               .ok_or_else(|| format!("Expected pattern before '{c}'"))?;
                    let last = Box::new(last);
                    match c {
                        '?' => MatchCase::Opt(last),
                        '+' => MatchCase::OneOrMore(last),
                        '*' => MatchCase::Star(last),
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
        self.accc.last_mut().unwrap().0.push(case);
    }
}

/// Represents a match of a string on a [Regex]
#[derive(Debug)]
pub struct RegexMatch<'a> {
    start: CharIndices<'a>,
    len: usize,
}

impl RegexMatch<'_> {
    /// Gets the span of the string where it matched the [Regex]
    pub fn get_span(&self) -> (usize,usize) {
        let o = self.start.offset();
        (o, o + self.len)
    }
    /// Gets the slice of the string that matched the [Regex]
    ///
    /// This is the same as calling [get_span](Self::get_span)
    /// and then using it to slice the source string
    pub fn get_slice(&self) -> &str {
        &self.start.as_str()[..self.len]
    }
}

/// Iterator over all the matches of a string in a [Regex]
#[derive(Debug)]
pub struct RegexMatcher<'a> {
    matches: &'a [MatchCase],
    start: CharIndices<'a>,
}

impl<'a> Iterator for RegexMatcher<'a> {
    type Item = RegexMatch<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start.as_str().is_empty() && self.matches.is_empty() {
            return None;
        }

        let mut chars = self.start.clone();

        for m in self.matches {
            if !m.matches(&mut chars) {
                return match self.start.next() {
                    Some(_) => self.next(),
                    None => None,
                };
            }
        }

        let start = self.start.offset();
        let end = chars.offset();

        let len = end - start;
        let len = self.start.as_str()[..len].chars().count();

        let start = self.start.clone();
        self.start = chars;

        if self.matches.is_empty() {
            self.start.next();
        }

        Some( RegexMatch { start, len } )
    }
}

impl Regex {
    /// Compile the given string into a [Regex]
    ///
    /// Returns error if the regex is invalid and fails to compile
    pub fn compile(src: &str) -> Result<Self,Cow<'static,str>> {
        RegexCompiler::new(src).process()
    }
    /// Returns an [Iterator] over all the [`matches`] of the [Regex] in the given string
    ///
    /// [`matches`]: RegexMatch
    pub fn find_matches<'a>(&'a self, src: &'a str) -> impl Iterator<Item = RegexMatch<'a>>  {
        RegexMatcher {
            matches: &self.matches,
            start: src.char_indices(),
        }
    }
    /// Returns true if the regex matches the given string
    ///
    /// This is the same as calling [find_matches](Self::find_matches)
    /// and then checking if the iterator contains at least one element
    pub fn test(&self, src: &str) -> bool {
        self.find_matches(src).next().is_some()
    }
}

/// This trait is used to add an extension method
/// [matches_regex](Self::matches_regex) to any str-like object
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
