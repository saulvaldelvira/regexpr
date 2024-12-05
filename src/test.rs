#![allow(
    clippy::unwrap_used,
    clippy::panic,
    clippy::expect_used,
    unused_must_use,
    clippy::pedantic,
)]

use crate::{Regex, RegexTestable};

fn template(regex: &str, must_pass: &[&str], must_fail: &[&str]) {
    let regex = Regex::compile(regex).unwrap();
    for mp in must_pass {
        if !regex.test(mp) {
            panic!("Should've passed: \"{mp}\"");
        }
    }
    for mf in must_fail {
        if regex.test(mf) {
            panic!("Should've failed: \"{mf}\"");
        }
    }
}

#[test]
fn abc() {
    template(
        "abc",
        &[
            "abc",
            "abcc",
            "aabc",
        ],
        &[
            "ab",
            "a",
            "bc",
        ]
    );
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
            Err(err) => assert_eq!(err, msg)
        }
    }
}

#[test]
fn find_matches() {
    let pattern = "A(bc)*D";
    let regex = Regex::compile(pattern).unwrap();

    let mut matches = regex.find_matches("AD_AD");
    let m = matches.next().unwrap();
    assert_eq!((0,2), m.get_span());
    assert_eq!(2, m.len);
    assert_eq!("AD", m.get_slice());

    let m = matches.next().unwrap();
    assert_eq!((3,5), m.get_span());
    assert_eq!(2, m.len);
    assert_eq!("AD", m.get_slice());


    let pattern = "";
    let regex = Regex::compile(pattern).unwrap();

    let mut matches = regex.find_matches("AD");
    let m = matches.next().unwrap();
    assert_eq!((0,0), m.get_span());
    assert_eq!(0, m.len);
    assert_eq!("", m.get_slice());

    let m = matches.next().unwrap();
    assert_eq!((1,1), m.get_span());
    assert_eq!(0, m.len);
    assert_eq!("", m.get_slice());

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
