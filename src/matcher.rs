use crate::{MatchCase, RegexConf};
use alloc::borrow::Cow;
use core::fmt::Display;
use core::iter::FusedIterator;
use core::str::CharIndices;

#[cfg(doc)]
use crate::Regex;

/// Represents a match of a string on a [Regex]
///
/// This struct is produced when iterating over a [`RegexMatcher`]
#[derive(Debug)]
pub struct RegexMatch<'a> {
    start: usize,
    slice: &'a str,
    captures: Option<Vec<&'a str>>,
}

impl<'a> RegexMatch<'a> {
    /// Gets the span of the string where it matched the [Regex]
    #[must_use]
    pub fn span(&self) -> (usize, usize) {
        let o = self.start;
        (o, o + self.slice.len())
    }
    /// Gets the slice of the string that matched the [Regex]
    ///
    /// This is the same as calling ``get_span``
    /// and then using it to slice the source string
    #[must_use]
    pub fn slice(&self) -> &str {
        self.slice
    }

    /// Gets the capture groups of this match
    ///
    /// The groups are returned in order, which means that
    /// capture group 1 will be at index 0, and so on.
    #[must_use]
    pub fn get_captures(&self) -> &[&'a str] {
        self.captures.as_deref().unwrap_or(&[])
    }
}

impl Display for RegexMatch<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let (s, e) = self.span();
        write!(f, "[{s},{e}]: \"{}\"", self.slice())
    }
}

/// Iterator over all the matches of a string in a [Regex]
#[derive(Debug, Clone)]
pub struct RegexMatcher<'a> {
    first: bool,
    ctx: RegexCtx<'a, 'a>,
    cases: LookAhead<'a, 'a>,
}

impl<'a> RegexMatcher<'a> {
    #[must_use]
    pub fn new(src: &'a str, matches: &'a [MatchCase], conf: RegexConf) -> Self {
        RegexMatcher {
            first: true,
            cases: LookAhead::new(LookAheadKind::List(matches), None),
            ctx: RegexCtx {
                captures: Cow::Borrowed(&[]),
                open_captures: Cow::Borrowed(&[][..]),
                conf,
                nc: src.char_indices(),
            },
        }
    }
}

impl<'a> Iterator for RegexMatcher<'a> {
    type Item = RegexMatch<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.ctx.nc.as_str().is_empty() && !self.first {
                return None;
            }
            let LookAheadKind::List(l) = self.cases.kind else { unreachable!() };
            if !self.first && l.first().is_some_and(|m| matches!(m, MatchCase::Start)) { return None }
            self.first = false;

            let mut chars = self.ctx.shallow_clone();
            if !self.cases.match_all(&mut chars) {
                match self.ctx.nc.next() {
                    Some(_) => continue,
                    None => return None,
                };
            }

            let start = self.ctx.nc.offset();
            let end = chars.nc.offset();

            let len = end - start;
            let slice = &self.ctx.nc.as_str()[..len];

            let RegexCtx { captures, nc, .. } = chars;
            let mut caps = None;
            if !captures.is_empty() {
                let mut v = Vec::with_capacity(captures.len());
                v.extend(
                    captures
                        .iter()
                        .map(|(c, l)| l.map(|l| &c.as_str()[..l]).unwrap_or("")),
                );
                caps = Some(v);
            }
            self.ctx.nc = nc;

            if self.cases.is_empty() {
                self.ctx.nc.next();
            }

            return Some(RegexMatch {
                start,
                slice,
                captures: caps,
            });
        }
    }
}

impl FusedIterator for RegexMatcher<'_> {}

#[derive(Clone, Debug)]
pub enum LookAheadKind<'a> {
    Repeat { m: &'a MatchCase, num: usize },
    List(&'a [MatchCase]),
}

#[derive(Debug, Clone)]
pub(crate) struct LookAhead<'l, 'a> {
    kind: LookAheadKind<'a>,
    then: Option<&'l LookAhead<'l, 'a>>,
}

impl<'l, 'a> LookAhead<'l, 'a> {
    pub fn new(l: LookAheadKind<'a>, then: Option<&'l LookAhead<'l, 'a>>) -> Self {
        LookAhead { kind: l, then }
    }

    pub fn match_all(&self, ctx: &mut RegexCtx<'_, 'a>) -> bool {
        let mut r = true;
        match self.kind {
            LookAheadKind::Repeat { m, mut num } if num > 0 => loop {
                num -= 1;
                r = m.matches(
                    ctx,
                    &LookAhead {
                        kind: LookAheadKind::Repeat { m, num },
                        then: self.then,
                    },
                );
                if !r || num == 0 {
                    break;
                }
            },
            LookAheadKind::List(match_cases) if !match_cases.is_empty() => {
                for i in 0..match_cases.len() {
                    let rem = match_cases.get(i + 1..).unwrap_or(&[]);
                    r = match_cases[i].matches(
                        ctx,
                        &LookAhead {
                            kind: LookAheadKind::List(rem),
                            then: self.then,
                        },
                    );
                    if !r {
                        break;
                    }
                }
                ctx.end_capture(&ctx.char_iter());
            }
            _ => {}
        }
        r && self.then.as_ref().is_none_or(|t| t.match_all(ctx))
    }

    const fn is_empty(&self) -> bool {
        self.then.is_none()
            && match self.kind {
                LookAheadKind::Repeat { num, .. } => num == 0,
                LookAheadKind::List(match_cases) => match_cases.is_empty(),
            }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct RegexCtx<'ctx, 'a> {
    captures: Cow<'ctx, [(CharIndices<'a>, Option<usize>)]>,
    open_captures: Cow<'ctx, [usize]>,
    conf: RegexConf,
    nc: CharIndices<'a>,
}

macro_rules! next {
    ($conf:expr, $chrs:expr) => {
        $chrs.next().map(|(_, c)| {
            if $conf.case_sensitive {
                c
            } else {
                c.to_lowercase().next().unwrap_or(c)
            }
        })
    };
}

impl<'a> RegexCtx<'_, 'a> {
    #[inline]
    pub fn next_char(&mut self) -> Option<char> {
        next!(self.conf, &mut self.nc)
    }
    #[inline]
    pub fn char_offset(&mut self) -> usize {
        self.nc.offset()
    }
    #[inline]
    pub fn char_iter(&self) -> CharIndices<'a> {
        self.nc.clone()
    }

    #[inline]
    pub fn peek_char(&mut self) -> Option<char> {
        next!(self.conf, self.nc.clone())
    }
    #[inline]
    pub fn conf(&self) -> RegexConf {
        self.conf
    }
    pub fn get_capture(&self, id: usize) -> &'a str {
        let id = id.wrapping_sub(1);
        let Some((nc, len)) = self.captures.get(id) else {
            return "";
        };
        &nc.as_str()[..len.unwrap_or_else(|| self.nc.offset() - nc.offset() - 1)]
    }
    pub fn start_capture(&mut self, id: usize, s: CharIndices<'a>) {
        let caps = self.captures.to_mut();
        if caps.len() <= (id - 1) {
            caps.resize_with(id, || (s.clone(), None));
        }
        caps[id - 1] = (s, None);
        self.open_captures.to_mut().push(id);
    }
    pub fn end_capture(&mut self, s: &CharIndices<'a>) {
        if self.open_captures.is_empty() {
            return;
        }
        let Some(id) = self.open_captures.to_mut().pop() else {
            unreachable!()
        };
        let c = &mut self.captures.to_mut()[id - 1];
        c.1 = Some(s.offset() - c.0.offset());
    }

    pub fn borrow_shallow<R>(&mut self, f: impl FnOnce(&mut RegexCtx<'_, 'a>) -> (R, bool)) -> R {
        let mut ctx = RegexCtx {
            captures: Cow::Borrowed(&self.captures),
            open_captures: Cow::Borrowed(&self.open_captures),
            nc: self.nc.clone(),
            conf: self.conf,
        };
        let (r, should_overwrite) = f(&mut ctx);
        let RegexCtx {
            captures,
            open_captures,
            nc,
            ..
        } = ctx;
        if should_overwrite {
            let mut cap = None;
            let mut ocap = None;
            if let Cow::Owned(o) = captures {
                cap = Some(o);
            }
            if let Cow::Owned(o) = open_captures {
                ocap = Some(o);
            }
            if let Some(o) = cap {
                self.captures = Cow::Owned(o);
            }
            if let Some(o) = ocap {
                self.open_captures = Cow::Owned(o);
            }
            self.nc = nc;
        }
        r
    }
    #[inline]
    pub fn shallow_clone<'slf>(&'slf self) -> RegexCtx<'slf, 'a> {
        RegexCtx {
            captures: Cow::Borrowed(&self.captures),
            nc: self.nc.clone(),
            open_captures: Cow::Borrowed(&self.open_captures),
            conf: self.conf,
        }
    }
}
