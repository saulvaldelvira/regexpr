use alloc::boxed::Box;
use alloc::vec::Vec;
use core::str::Chars;
use std::collections::HashMap;

use crate::Regex;
use crate::Result;
use crate::case::MatchCase;

type OrList = Vec<MatchCase>;
type RegexCompilerScope = (Vec<MatchCase>, Option<OrList>, Option<usize>);

pub struct RegexCompiler<'a> {
    chars: Chars<'a>,
    open: usize,
    accc: Vec<RegexCompilerScope>,
    captures_map: HashMap<String, usize>,
    n_captures: usize,
}

impl<'a> RegexCompiler<'a> {
    pub fn new(src: &'a str) -> Self {
        let mut compiler = RegexCompiler {
            chars: src.chars(),
            open: 0,
            accc: Vec::new(),
            n_captures: 0,
            captures_map: HashMap::new(),
        };
        compiler
            .enter_scope(false)
            .unwrap_or_else(|_| unreachable!());
        compiler
    }
    fn enter_scope(&mut self, capt: bool) -> Result<()> {
        self.open += 1;
        let cid = if capt {
            self.n_captures += 1;

            if self.chars.clone().next().is_some_and(|c| c == '?') {
                self.chars.next();
                if self.chars.next().is_none_or(|c| c != '<') {
                    return Err("Expected an opening '<'".into());
                }
                let Some(close) = self.chars.as_str().find('>') else {
                    return Err("Expected closing '<'".into());
                };
                let name = self.chars.as_str()[..close].to_string();
                for _ in 0..=close {
                    self.chars.next();
                }
                self.captures_map.insert(name, self.n_captures);
            }
            Some(self.n_captures)
        } else {
            None
        };
        self.accc.push((Vec::new(), None, cid));
        Ok(())
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
        let mut min = None;
        let mut max = None;
        if slice.contains(',') {
            let mut split = slice.split(',');
            let mi = split
                .next()
                .ok_or("Range must be split by ','. Ex: {12,15}")?;
            let ma = split
                .next()
                .ok_or("Range must be split by ','. Ex: {12,15}")?;

            if !mi.is_empty() {
                min = Some(mi.parse().ok().ok_or("Error parsing number")?);
            }
            if !ma.is_empty() {
                max = Some(ma.parse().ok().ok_or("Error parsing number")?);
            }
        } else {
            let n = slice.parse().ok().ok_or("Error parsing number")?;
            min = Some(n);
            max = Some(n);
        }

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
        }
    }
    fn escape(&mut self, c: char) -> Result<MatchCase> {
        let next = self.next(c)?;
        if next == 's' {
            return Ok(MatchCase::Whitespace);
        } else if next == 'S' {
            return Ok(MatchCase::NotWhitespace);
        } else if next == 'd' {
            return Ok(MatchCase::Decimal);
        } else if next == 'D' {
            return Ok(MatchCase::NotDecimal);
        } else if next == 'w' {
            return Ok(MatchCase::Word);
        } else if next == 'W' {
            return Ok(MatchCase::NotWord);
        }

        let mut is_cap = next.is_numeric();
        let mut named = false;
        if !is_cap && next == 'k' && self.chars.as_str().starts_with('<') {
            self.chars.next();
            is_cap = true;
            named = true;
        }

        let case = if is_cap {
            let mut captn = 0;
            if named {
                let Some(close) = self.chars.as_str().find('>') else {
                    return Err("Expected closing '>'".into());
                };
                let name = &self.chars.as_str()[..close];
                if let Ok(id) = name.parse::<usize>() {
                    captn = id;
                } else {
                    match self.captures_map.get(name) {
                        Some(id) => captn = *id,
                        None => return Err(format!("Unknown capture '{name}'").into()),
                    }
                }
                for _ in 0..=close {
                    self.chars.next();
                }
            } else {
                let mut next = next;
                loop {
                    captn = captn * 10 + (next as u8 - b'0') as usize;

                    match self.chars.clone().next() {
                        Some(n) if n.is_numeric() => {
                            self.chars.next();
                            next = n;
                        }
                        _ => break,
                    }
                }
            }
            if self.n_captures < captn {
                return Err("Trying to recall uncaptured".into());
            }
            MatchCase::Capture(captn)
        } else {
            MatchCase::Char(next)
        };
        Ok(case)
    }
    pub fn process(&mut self) -> Result<Regex> {
        while let Some(c) = self.chars.next() {
            let newcase = match c {
                '.' => MatchCase::AnyOne,
                '\\' => self.escape(c)?,
                '(' => {
                    self.enter_scope(true)?;
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
        })
    }
    fn append(&mut self, case: MatchCase) {
        if self.accc.is_empty() {
            self.accc.push((Vec::new(), None, None));
        }
        self.last_acc().0.push(case);
    }
}
