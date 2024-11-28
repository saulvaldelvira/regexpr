use core::panic;

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
        &["abc"],
        &[
            "ab",
            "a",
            "bc",
            "abcc",
            "aabc"
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
        ],
        &[
            "babc",
            "aabc",
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
        "(abc)*",
        &[
            "abc",
            "abcabc",
            ""
        ],
        &[
            "bbc",
            "ababc",
            "abcab"
        ],
    );
}

#[test]
fn plus() {
    template(
        "a+bc",
        &[
            "abc",
            "aabc",
            "aaaabc",
        ],
        &[
            "bc",
            "bbc",
            "ababc",
        ],
    );
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
