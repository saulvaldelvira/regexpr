use alloc::boxed::Box;

use crate::matcher::RegexCtx;

#[derive(Clone, Debug)]
pub enum MatchCase {
    Start,
    End,
    Char(char),
    List(Box<[MatchCase]>),
    Group {
        case: Box<MatchCase>,
        capture_id: usize,
    },
    Or(Box<[MatchCase]>),
    AnyOne,
    Opt(Box<MatchCase>),
    OneOrMore {
        case: Box<MatchCase>,
        lazy: bool,
    },
    Star {
        case: Box<MatchCase>,
        lazy: bool,
    },
    Capture(usize),
    Between(char, char),
    CharMatch(Box<[MatchCase]>),
    RangeLoop {
        case: Box<MatchCase>,
        min: Option<usize>,
        max: Option<usize>,
    },
    Not(Box<MatchCase>),
}

impl MatchCase {
    fn lazy_star_loop<'a>(&'a self, ctx: &mut RegexCtx<'a>) -> bool {
        loop {
            if ctx.has_following() && ctx.clone().following_match() {
                return true;
            }
            let mut it = ctx.clone();
            if self.matches(&mut it) {
                *ctx = it;
                ctx.update_open_captures();
            } else {
                return true;
            }
        }
    }
    fn greedy_star_loop<'a>(&'a self, ctx: &mut RegexCtx<'a>) -> bool {
        let mut last_next_match = None;

        loop {
            if ctx.clone().following_match() {
                last_next_match = Some(ctx.clone());
            }

            let mut it = ctx.clone();
            if self.matches(&mut it) {
                *ctx = it;
                ctx.update_open_captures();
            } else {
                if let Some(it) = last_next_match {
                    *ctx = it;
                }
                return true;
            }
        }
    }
    fn star_loop<'a>(&'a self, ctx: &mut RegexCtx<'a>, lazy: bool) -> bool {
        ctx.update_open_captures();
        if lazy {
            self.lazy_star_loop(ctx)
        } else {
            self.greedy_star_loop(ctx)
        }
    }
    #[allow(clippy::too_many_lines)]
    pub(crate) fn matches<'a>(&'a self, ctx: &mut RegexCtx<'a>) -> bool {
        macro_rules! next {
            () => {{
                let Some(ch) = ctx.next_char() else {
                    return false;
                };
                ch
            }};
        }

        match self {
            MatchCase::Char(expected) => next!() == *expected,
            MatchCase::Group { case, capture_id } => {
                ctx.push_capture(*capture_id);
                let ret = case.matches(ctx);
                ctx.update_open_captures();
                ctx.pop_capture();
                ret
            }
            MatchCase::List(cases) => cases.iter().all(|rule| rule.matches(ctx)),
            MatchCase::Or(l) => l.iter().any(|rule| {
                let mut newit = ctx.clone();
                let ret = rule.matches(&mut newit);
                if ret {
                    *ctx = newit;
                }
                ret
            }),
            MatchCase::Opt(c) => {
                let mut newit = ctx.clone();
                if c.matches(&mut newit) && newit.clone().following_match() {
                    *ctx = newit;
                }
                true
            }
            MatchCase::AnyOne => ctx.next_char().is_some(),
            MatchCase::OneOrMore { case, lazy } => {
                if !case.matches(ctx) {
                    return false;
                }

                case.star_loop(ctx, *lazy)
            }
            MatchCase::Star { case, lazy } => case.star_loop(ctx, *lazy),
            MatchCase::Start => ctx.char_offset() == 0,
            MatchCase::End => ctx.next_char().is_none(),
            MatchCase::Between(start, end) => {
                let c = next!();
                let (start, end) = if ctx.conf().case_sensitive {
                    (*start, *end)
                } else {
                    (
                        start.to_lowercase().next().unwrap_or(*start),
                        end.to_lowercase().next().unwrap_or(*end),
                    )
                };
                c >= start && c <= end
            }
            MatchCase::Not(match_case) => match ctx.peek_char() {
                Some(_) => !match_case.matches(ctx),
                None => false,
            },
            MatchCase::CharMatch(cases) => {
                let ret = cases.iter().any(|case| case.matches(&mut ctx.clone()));
                ctx.next_char();
                ret
            }
            MatchCase::RangeLoop { case, min, max } => {
                let mut n = 0;

                if let Some(min) = min {
                    for _ in 0..*min {
                        if !case.matches(ctx) {
                            return false;
                        }
                        n += 1;
                    }
                }

                loop {
                    if max.is_some_and(|max| n > max) {
                        return false;
                    }

                    let mut it = ctx.clone();
                    if case.matches(&mut it) {
                        *ctx = it;
                    } else {
                        break;
                    }

                    n += 1;
                }

                true
            }
            MatchCase::Capture(n) => {
                let case_sensitive = ctx.conf().case_sensitive;
                ctx.get_capture(*n)
                    .chars()
                    .map(|c| {
                        if case_sensitive {
                            c
                        } else {
                            c.to_lowercase().next().unwrap_or(c)
                        }
                    })
                    .all(|c| next!() == c)
            }
        }
    }
}
