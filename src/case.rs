use alloc::boxed::Box;

use crate::matcher::{LookAhead, LookAheadKind, RegexCtx};

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
    Whitespace,
    NotWhitespace,
    Decimal,
    NotDecimal,
    Word,
    NotWord,
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
    fn lazy_star_loop<'a>(
        &'a self,
        ctx: &mut RegexCtx<'_, 'a>,
        lookahead: &LookAhead<'_, 'a>,
    ) -> bool {
        loop {
            if ctx.borrow_shallow(|ctx| (lookahead.match_all(ctx), false)) {
                return true;
            }
            let is_match = ctx.borrow_shallow(|ctx| {
                let ret = self.matches(ctx, lookahead);
                (ret, ret)
            });
            if !is_match {
                return true;
            }
        }
    }
    fn greedy_star_loop<'a>(
        &'a self,
        ctx: &mut RegexCtx<'_, 'a>,
        lookahead: &LookAhead<'_, 'a>,
    ) -> bool {
        let mut last_next_match = None;

        loop {
            if ctx.borrow_shallow(|ctx| (lookahead.match_all(ctx), false)) {
                last_next_match = Some(ctx.clone());
            }

            let is_match = ctx.borrow_shallow(|ctx| {
                let ret = self.matches(ctx, lookahead);
                (ret, ret)
            });
            if !is_match {
                if let Some(it) = last_next_match {
                    *ctx = it;
                }
                return true;
            }
        }
    }
    fn star_loop<'a>(
        &'a self,
        ctx: &mut RegexCtx<'_, 'a>,
        lazy: bool,
        lookahead: &LookAhead<'_, 'a>,
    ) -> bool {
        if lazy {
            self.lazy_star_loop(ctx, lookahead)
        } else {
            self.greedy_star_loop(ctx, lookahead)
        }
    }
    #[allow(clippy::too_many_lines)]
    pub(crate) fn matches<'a>(
        &'a self,
        ctx: &mut RegexCtx<'_, 'a>,
        lookahead: &LookAhead<'_, 'a>,
    ) -> bool {
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
            MatchCase::Whitespace => next!().is_whitespace(),
            MatchCase::NotWhitespace => !next!().is_whitespace(),
            MatchCase::Decimal => next!().is_digit(10),
            MatchCase::Word => {
                let c = next!();
                c.is_alphanumeric() || c == '_'
            }
            MatchCase::NotWord => {
                let c = next!();
                !c.is_alphanumeric() && c != '_'
            }
            MatchCase::NotDecimal => !next!().is_digit(10),
            MatchCase::Group { case, capture_id } => {
                let curr = ctx.char_iter();
                ctx.start_capture(*capture_id, curr);
                let ret = case.matches(ctx, lookahead);
                ctx.end_capture(&ctx.char_iter());
                ret
            }
            MatchCase::List(cases) => {
                for i in 0..cases.len() {
                    let rem = cases.get(i + 1..).unwrap_or(&[]);
                    let look = LookAhead::new(LookAheadKind::List(rem), Some(lookahead));
                    let is_match = cases[i].matches(ctx, &look);
                    if !is_match {
                        return false;
                    }
                }
                true
            }
            MatchCase::Or(l) => l.iter().any(|rule| {
                ctx.borrow_shallow(|newit| {
                    let ret = rule.matches(newit, lookahead);
                    (ret, ret)
                })
            }),
            MatchCase::Opt(c) => {
                ctx.borrow_shallow(|newit| {
                    if c.matches(newit, lookahead)
                        && lookahead.match_all(&mut newit.shallow_clone())
                    {
                        ((), true)
                    } else {
                        ((), false)
                    }
                });
                true
            }
            MatchCase::AnyOne => ctx.next_char().is_some(),
            MatchCase::OneOrMore { case, lazy } => {
                if !case.matches(ctx, lookahead) {
                    return false;
                }

                case.star_loop(ctx, *lazy, lookahead)
            }
            MatchCase::Star { case, lazy } => case.star_loop(ctx, *lazy, lookahead),
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
                Some(_) => !match_case.matches(ctx, lookahead),
                None => false,
            },
            MatchCase::CharMatch(cases) => {
                let ret = cases
                    .iter()
                    .any(|case| case.matches(&mut ctx.shallow_clone(), lookahead));
                ctx.next_char();
                ret
            }
            MatchCase::RangeLoop { case, min, max } => {
                let mut n = 0;

                if let Some(min) = min {
                    for i in 0..*min {
                        let look = LookAhead::new(
                            LookAheadKind::Repeat {
                                m: case,
                                num: *min - i - 1,
                            },
                            Some(lookahead),
                        );
                        if !case.matches(ctx, &look) {
                            return false;
                        }
                        n += 1;
                    }
                }

                loop {
                    if max.is_some_and(|max| n >= max) {
                        break;
                    }

                    if !ctx.borrow_shallow(|it| {
                        let ret = case.matches(it, lookahead);
                        (ret, ret)
                    }) {
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
