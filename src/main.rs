use std::io::{stdin, stdout, Write};

use regexpr::Regex;

fn main() {
    print!("Enter a regular expression: ");
    stdout().flush().unwrap();

    let mut buf = String::new();
    stdin().read_line(&mut buf).unwrap();
    let buf = buf.trim().replace("\n", "").replace("\r", "");

    let regex = Regex::compile(&buf).expect("Invalid regex");

    print!("> ");
    stdout().flush().unwrap();

    stdin().lines().map_while(Result::ok).for_each(|line| {
        if regex.test(&line) {
            println!("Matches!");
        } else {
            println!("Doesn't match");
        }
        print!("> ");
        stdout().flush().unwrap();
    });
}
