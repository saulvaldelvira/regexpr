use std::borrow::Cow;
use std::fmt::Display;
use std::iter::FusedIterator;
use std::str::CharIndices;
use crate::MatchCase;


#[cfg(doc)]
use crate::Regex;

/// Represents a match of a string on a [Regex]
///
/// This struct is produced when iterating over a [`RegexMatcher`]
#[derive(Debug)]
pub struct RegexMatch<'a> {
    start: usize,
    slice: &'a str,
}

impl RegexMatch<'_> {
    /// Gets the span of the string where it matched the [Regex]
    #[must_use]
    pub fn span(&self) -> (usize,usize) {
        let o = self.start;
        (o, o + self.slice.len())
    }
    /// Gets the slice of the string that matched the [Regex]
    ///
    /// This is the same as calling ``get_span``
    /// and then using it to slice the source string
    #[must_use]
    pub fn slice(&self) -> &str { self.slice }
}

impl Display for RegexMatch<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (s,e) = self.span();
        write!(f, "[{s},{e}]: \"{}\"", self.slice())
    }
}

/// Iterator over all the matches of a string in a [Regex]
#[derive(Debug,Clone)]
pub struct RegexMatcher<'a> {
    first: bool,
    ctx: RegexCtx<'a>,
    cases: &'a [MatchCase],

}

impl<'a> RegexMatcher<'a> {
    #[must_use]
    pub fn new(src: &'a str, matches: &'a [MatchCase], n_captures: usize) -> Self {
        let captures = vec![""; n_captures].into_boxed_slice();
        RegexMatcher {
            first: true,
            cases: matches,
            ctx: RegexCtx {
                captures: Cow::Owned(captures),
                following: matches,
                nc: src.char_indices(),
                open_captures: Cow::Owned(Vec::new()),
            }
        }
    }
    /// Gets the current state of the capture groups.
    ///
    /// The groups are returned in order, which means that
    /// capture group 1 will be at index 0, and so on.
    ///
    /// # Note
    /// This list may be updated when [next](Iterator::next) is called.
    /// So a common use of this struct, if we want to check the groups
    /// at the end is the following.
    ///
    /// ```rust
    /// use regexpr::RegexMatcher;
    ///
    /// fn check(mut matcher: RegexMatcher<'_>) {
    ///     /* Borrow the matcher instead of moving it, so we
    ///     can use it later to check the capture groups */
    ///     for m in &mut matcher {
    ///         /* Do something with m */
    ///     }
    ///     for group in matcher.get_groups() {
    ///         /* Do something with group */
    ///     }
    /// }
    /// ```
    ///
    #[must_use]
    pub fn get_groups(&self) -> &[&'a str] {
        &self.ctx.captures
    }
}

impl<'a> Iterator for RegexMatcher<'a> {
    type Item = RegexMatch<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ctx.nc.as_str().is_empty() && !self.first {
            return None;
        }
        self.first = false;

        let mut chars = self.ctx.clone();
        chars.following = self.cases;
        if !chars.following_match() {
            return match self.ctx.nc.next() {
                Some(_) => self.next(),
                None => None,
            };
        }

        let start = self.ctx.nc.offset();
        let end = chars.nc.offset();

        let len = end - start;
        let slice = &self.ctx.nc.as_str()[..len];

        self.ctx = chars;

        if self.cases.is_empty() {
            self.ctx.nc.next();
        }

        Some( RegexMatch { start, slice } )
    }
}

impl FusedIterator for RegexMatcher<'_> { }

#[derive(Clone,Debug)]
pub (crate) struct RegexCtx<'a> {
    captures: Cow<'a, Box<[&'a str]>>,
    following: &'a [MatchCase],
    nc: CharIndices<'a>,
    open_captures: Cow<'a, Vec<(usize,CharIndices<'a>)>>,
}

impl<'a> RegexCtx<'a> {
    pub fn chars(&mut self) -> &mut CharIndices<'a> { &mut self.nc }
    fn next_case(&mut self) {
        self.following = self.following.get(1..).unwrap_or(&[]);
    }
    pub fn get_capture(&self, id: usize) -> &'a str {
        let id = id.wrapping_sub(1);
        self.captures.get(id).unwrap_or_else(||
            unreachable!()
        )
    }
    pub fn push_capture(&mut self, id: usize) {
        let caps = self.open_captures.to_mut();
        caps.reserve(self.captures.len());
        let e = (id, self.nc.clone());
        if caps.len() >= caps.capacity() {
            unreachable!()
        } else {
            caps.push(e);
        }
        /* UNSTABLE ALTERNATIVE */
        /* caps.push_within_capacity(e) */
        /*     .unwrap_or_else(|_| { */
        /*         unreachable!() */
        /*     }); */

    }
    pub fn pop_capture(&mut self) {
        self.open_captures.to_mut().pop();
    }
    pub fn has_following(&self) -> bool { !self.following.is_empty() }
    pub fn update_open_captures(&mut self) {
        for (id,chars) in self.open_captures.to_mut() {
            let len = self.nc.offset() - chars.offset();
            let slice = &chars.as_str()[..len];
            self.captures.to_mut()[*id - 1] = slice;
        }
    }
    pub fn following_match(&mut self) -> bool {
        let cases = self.following;
        for case in cases {
            self.next_case();
            if !case.matches(self) {
                return false
            }
        }
        true
    }
}

