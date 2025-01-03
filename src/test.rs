#![allow(
    clippy::unwrap_used,
    clippy::panic,
    clippy::expect_used,
    unused_must_use,
    clippy::pedantic,
)]

use std::borrow::Cow;

use crate::{Regex, RegexConf, RegexTestable, ReplaceRegex, DEFAULT_REGEX_CONF};

fn template_with_conf(regex: &str, conf: RegexConf, must_pass: &[&str], must_fail: &[&str]) {
    let regex = Regex::compile(regex).unwrap();
    for mp in must_pass {
        if !regex.test_with_conf(mp, conf) {
            panic!("Should've passed: \"{mp}\"");
        }
    }
    for mf in must_fail {
        if regex.test_with_conf(mf, conf) {
            panic!("Should've failed: \"{mf}\"");
        }
    }
}

fn template(regex: &str, must_pass: &[&str], must_fail: &[&str]) {
    template_with_conf(regex, DEFAULT_REGEX_CONF, must_pass, must_fail);
}

#[test]
fn abc() {
    template(
        "abc",
        &[
            "abc",
            "abcc",
            "aabc",
            "abcabc",
        ],
        &[
            "ab",
            "a",
            "bc",
        ]
    );
    let regex = Regex::compile("abc").unwrap();
    assert_eq!(2, regex.find_matches("abcabc").count());
}

#[test]
fn dot() {
    template(
        "a..d",
        &[
            "abcd",
            "a..d",
        ],
        &[
            "ad",
            "abd",
            "abcc",
            "aabc",
            "....",
        ]
    );
}

#[test]
fn or() {
    template(
        "(abc|cba)",
        &[
            "abc",
            "cba",
            "babc",
            "aabc",
        ],
        &[
            "cga"
        ]
    );
}

#[test]
fn opt() {
    template(
        "head(opt-body)?tail",
        &[
            "headtail",
            "headopt-bodytail",
        ],
        &[
            "headopt-body",
            "opt-bodytail",
        ],
    );
    template(
        "a.?b",
        &[
            "ab",
            "acb",
        ],
        &[
            "accb",
            "ac",
        ],
    );
}

#[test]
fn star() {
    template(
        "a(abc)*c",
        &[
            "aabcc",
            "ac",
            "aabcabcc",
        ],
        &[
            "abbc"
        ],
    );
    template(".*", &["", "daksd"], &[]);
}

#[test]
fn plus() {
    template(
        "a+bc",
        &[
            "abc",
            "aabc",
            "aaaabc",
            "ababc",
        ],
        &[
            "bc",
            "bbc",
        ],
    );
}

#[test]
fn start_end() {
    template("abc", &["abc", "aabc", "abcc"], &[]);
    template("^abc", &["abc", "abcc"], &["aabc"]);
    template("abc$", &["abc", "aabc"], &["abcc"]);
    template("^abc$", &["abc"], &["aabc","abcc"]);
}

#[test]
fn trait_test() {
    assert!("a??bbbc".matches_regex("a..b+c"));
    assert!(!"abc".matches_regex("a\\.c"));
}

#[test]
fn nested() {
    template(
        "abc((dfg)+|(hij)+)?klm",
        &[
            "abcdfgklm",
            "abcklm",
            "abcdfgklm",
        ],
        &[
            "abcdfghijklm",
        ]
    );
}

#[test]
fn fail() {
    for c in ["?", "*", "+"] {
        let msg = format!("Expected pattern before '{c}'");
        match Regex::compile(c) {
            Ok(_) => panic!(),
            Err(err) => assert_eq!(err.to_string(), msg)
        }
    }
}

#[test]
fn find_matches() {
    let pattern = "A(bc)*D";
    let regex = Regex::compile(pattern).unwrap();

    let mut matches = regex.find_matches("AD_AD");
    let m = matches.next().unwrap();
    assert_eq!((0,2), m.span());
    assert_eq!(2, m.slice().len());
    assert_eq!("AD", m.slice());

    let m = matches.next().unwrap();
    assert_eq!((3,5), m.span());
    assert_eq!(2, m.slice().len());
    assert_eq!("AD", m.slice());


    let pattern = "";
    let regex = Regex::compile(pattern).unwrap();

    let mut matches = regex.find_matches("AD");
    let m = matches.next().unwrap();
    assert_eq!((0,0), m.span());
    assert_eq!(0, m.slice().len());
    assert_eq!("", m.slice());

    let m = matches.next().unwrap();
    assert_eq!((1,1), m.span());
    assert_eq!(0, m.slice().len());
    assert_eq!("", m.slice());

    assert!(matches.next().is_none());
}

#[test]
fn range() {
    template(
        "^[a-z01]+$",
        &[
            "avcd",
            "0101baba1"
        ],
        &[
            "avcdZZka",
            "0101baba91"
        ]
    );
    template(
        "^[^a-z01]+$",
        &[
            "99882",
        ],
        &[
            "avcd",
            "0101baba1",
            "avcdZZka",
            "0101baba91"
        ]
    );
}

#[test]
fn min_max() {
    template(
        "^a{3,5}$",
        &[
            "aaa",
            "aaaa",
            "aaaaa",
        ],
        &[
            "a",
            "aa",
            "aaaaaa",
            "aaaaaaa",
        ]
    );

    template(
        "^a{3,}$",
        &[
            "aaa",
            "aaaa",
            "aaaaa",
            "aaaaaa",
            "aaaaaaa",
        ],
        &[
            "a",
            "aa",
        ]
    );

    template(
        "^a{,5}$",
        &[
            "a",
            "aa",
            "aaa",
            "aaaa",
            "aaaaa",
        ],
        &[
            "aaaaaa",
            "aaaaaaa",
        ]
    );
}

#[test]
fn lazy() {
    let regex = Regex::compile(".*?b").unwrap();
    assert_eq!(2, regex.find_matches("aaaaaabaaaaaab").count());

    let regex = Regex::compile(".*b").unwrap();
    assert_eq!(1, regex.find_matches("aaaaaabaaaaaab").count());

    let regex = Regex::compile(".+?b").unwrap();
    assert_eq!(2, regex.find_matches("aaaaaabaaaaaab").count());

    let regex = Regex::compile(".+b").unwrap();
    assert_eq!(1, regex.find_matches("aaaaaabaaaaaab").count());
}

#[test]
fn capture() {
    template (
        "^ab(.)c\\1$",
        &[
            "ab1c1",
            "ab2c2",
        ],
        &[
            "ab1c2",
            "ab2c1",
        ],
    );
    template (
        "^ab( [a-z]* )c\\1$",
        &[
            "ab abcd c abcd ",
            "ab ahc c ahc ",
        ],
        &[
            "ab ahc c ahc",
            "ab ahc cahc ",
            "ab ahc cahc",
            "ab ahcc ahc",
            "ab ag2a c ag2a ",
            "ab1c2",
            "ab2c1",
        ],
    );
    template (
        "^1(.*?)2\\1(.*?)3\\k<2>4$",
        &[
            "1abc2abcdef3def4",
            "1abc2abc34",
        ],
        &[
            "1abc2abcd34"
        ]
    );
}

#[test]
fn capture_or() {
    template (
        "^(abc|def)123\\1$",
        &[
            "abc123abc",
            "def123def",
        ],
        &[
            "abc123def",
            "def123abc",
        ],
    );
}

#[test]
fn case_sensitive() {
    template_with_conf(
        "abc[a-z]",
        RegexConf { case_sensitive: false },
        &[
            "abcz",
            "ABCz",
            "AbcZ",
            "abCZbABc",
        ],
        &[
            "abz",
            "abdc"
        ]
    );
    template_with_conf(
        "abc[a-z]",
        RegexConf { case_sensitive: true },
        &[
            "abcz",
            "abca",
        ],
        &[
            "ABC",
            "Abc",
            "abcZ",
            "abCbABc",
            "ab",
            "abdc"
        ]
    )
}

#[test]
fn range_with_star() {
    template(
        "([aeiou].*){3,}",
        &[
            "aei",
            "assdseki",
        ],
        &[
            "asi",
            "ao",
        ]
    );
}

#[test]
fn test_following() {
    template(
        "a(b.*c)+d",
        &[
            "abcd"
        ],
        &[]
    );
}

#[test]
fn replace_regex() {
    let input = "abcdacb";
    let replaced = input.replace_regex("a.?b", "0").unwrap();
    assert!(matches!(replaced, Cow::Owned(_)));
    assert_eq!(replaced, "0cd0");

    let input = "abcd";
    let replaced = input.replace_regex("[0-9]", "P").unwrap();
    assert!(matches!(replaced, Cow::Borrowed(_)));
    assert_eq!(replaced, input);
}
