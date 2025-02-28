use alloc::boxed::Box;
use alloc::vec::Vec;
use core::str::Chars;

use crate::Regex;
use crate::Result;
use crate::case::MatchCase;

type OrList = Vec<MatchCase>;
type RegexCompilerScope = (Vec<MatchCase>, Option<OrList>, Option<usize>);

pub struct RegexCompiler<'a> {
    chars: Chars<'a>,
    open: usize,
    accc: Vec<RegexCompilerScope>,
    n_captures: usize,
}

impl<'a> RegexCompiler<'a> {
    pub fn new(src: &'a str) -> Self {
        let mut compiler = RegexCompiler {
            chars: src.chars(),
            open: 0,
            accc: Vec::new(),
            n_captures: 0,
        };
        compiler.enter_scope(false);
        compiler
    }
    fn enter_scope(&mut self, capt: bool) {
        self.open += 1;
        let cid = if capt {
            self.n_captures += 1;
            Some(self.n_captures)
        } else {
            None
        };
        self.accc.push((Vec::new(), None, cid));
    }
    fn close_scope(&mut self) -> MatchCase {
        self.open -= 1;

        match self.accc.pop() {
            Some((acc, orlist, cid)) => {
                let list = MatchCase::List(acc.into_boxed_slice());
                let mut case = if let Some(mut orl) = orlist {
                    orl.push(list);
                    MatchCase::Or(orl.into_boxed_slice())
                } else {
                    list
                };
                if let Some(id) = cid {
                    case = MatchCase::Group {
                        case: Box::new(case),
                        capture_id: id,
                    };
                }
                case
            }
            None => unreachable!(),
        }
    }
    fn last_acc(&mut self) -> &mut RegexCompilerScope {
        self.accc.last_mut().unwrap_or_else(|| unreachable!())
    }
    fn next(&mut self, c: char) -> Result<char> {
        self.chars
            .next()
            .ok_or_else(|| format!("Expected character after {c}").into())
    }
    fn multiplier(&mut self, c: char) -> Result<MatchCase> {
        let last = self
            .last_acc()
            .0
            .pop()
            .ok_or_else(|| format!("Expected pattern before '{c}'"))?;
        let last = Box::new(last);

        let lazy = self.chars.clone().next().is_some_and(|c| c == '?');
        if lazy {
            self.chars.next();
        }

        let case = match c {
            '?' => MatchCase::Opt(last),
            '+' => MatchCase::OneOrMore { case: last, lazy },
            '*' => MatchCase::Star { case: last, lazy },
            _ => unreachable!(),
        };
        Ok(case)
    }
    fn repeat(&mut self, c: char) -> Result<MatchCase> {
        let last = self
            .last_acc()
            .0
            .pop()
            .ok_or_else(|| format!("Expected pattern before '{c}'"))?;

        /* a{100,1000} */

        let i = self.chars.as_str().find('}').ok_or("Missing closing '}'")?;
        let slice = &self.chars.as_str()[..i];
        let mut split = slice.split(',');
        let min = split
            .next()
            .ok_or("Range must be split by ','. Ex: {12,15}")?;
        let max = split
            .next()
            .ok_or("Range must be split by ','. Ex: {12,15}")?;

        let min = if min.is_empty() {
            None
        } else {
            Some(min.parse().ok().ok_or("Error parsing number")?)
        };
        let max = if max.is_empty() {
            None
        } else {
            Some(max.parse().ok().ok_or("Error parsing number")?)
        };

        for _ in 0..=i {
            self.chars.next();
        }

        Ok(MatchCase::RangeLoop {
            case: Box::new(last),
            min,
            max,
        })
    }
    fn range(&mut self, c: char) -> Result<MatchCase> {
        let mut curr = self.next(c)?;
        let negated = curr == '^';
        if negated {
            curr = self.next(c)?;
        }

        let mut list = Vec::new();

        while curr != ']' {
            if curr == '\\' {
                curr = self.next(c)?;
            }
            let c = curr;
            curr = self.next(c)?;

            if curr == '-' {
                let end = self.next(c)?;
                if end == ']' {
                    return Err("Expectend end of range [.. - ..]".into());
                }
                list.push(MatchCase::Between(c, end));
                curr = self.next(c)?;
            } else {
                list.push(MatchCase::Char(c));
            }
        }

        let match_case = list.into_boxed_slice();
        let case = if negated {
            MatchCase::Not(Box::new(MatchCase::CharMatch(match_case)))
        } else {
            MatchCase::CharMatch(match_case)
        };
        Ok(case)
    }
    fn or(&mut self) {
        match self.accc.pop() {
            Some((mut acc, mut opt, cid)) => {
                let m = if acc.len() > 1 {
                    MatchCase::List(acc.into_boxed_slice())
                } else {
                    acc.remove(0)
                };
                opt.get_or_insert_with(Vec::new).push(m);
                self.accc.push((Vec::new(), opt, cid));
            }
            None => unreachable!(),
        };
    }
    fn escape(&mut self, c: char) -> Result<MatchCase> {
        let mut is_cap = self.chars.clone().next().is_some_and(char::is_numeric);

        let mut arrrows = false;
        if !is_cap && self.chars.as_str().strip_prefix("k<").is_some() {
            self.chars.next();
            self.chars.next();
            is_cap = true;
            arrrows = true;
        }

        let case = if is_cap {
            let mut captn = 0;
            while let Some(n) = self.chars.clone().next() {
                if !n.is_numeric() {
                    if arrrows && self.next(c)? != '>' {
                        return Err("Expected closing '>'".into());
                    }
                    break;
                }

                captn = captn * 10 + (n as u8 - b'0') as usize;

                self.chars.next();
            }
            if self.n_captures < captn {
                return Err("Trying to recall uncaptured".into());
            }
            MatchCase::Capture(captn)
        } else {
            MatchCase::Char(self.next(c)?)
        };
        Ok(case)
    }
    pub fn process(&mut self) -> Result<Regex> {
        while let Some(c) = self.chars.next() {
            let newcase = match c {
                '.' => MatchCase::AnyOne,
                '\\' => self.escape(c)?,
                '(' => {
                    self.enter_scope(true);
                    continue;
                }
                ')' => self.close_scope(),
                '|' => {
                    self.or();
                    continue;
                }
                '[' => self.range(c)?,
                '{' => self.repeat(c)?,
                '?' | '*' | '+' => self.multiplier(c)?,
                '^' => MatchCase::Start,
                '$' => MatchCase::End,
                c => MatchCase::Char(c),
            };
            self.append(newcase);
        }

        let matches = match self.close_scope() {
            MatchCase::List(cases) => cases,
            MatchCase::Or(l) => Box::from([MatchCase::Or(l)]),
            _ => unreachable!(),
        };

        Ok(Regex {
            matches,
            n_captures: self.n_captures,
        })
    }
    fn append(&mut self, case: MatchCase) {
        if self.accc.is_empty() {
            self.accc.push((Vec::new(), None, None));
        }
        self.last_acc().0.push(case);
    }
}
