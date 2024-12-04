use std::io::{stdin, stdout, Write};

use regexpr::Regex;

fn main() {
    loop {
        print!("Enter a regular expression: ");
        stdout().flush().unwrap();

        let mut buf = String::new();
        stdin().read_line(&mut buf).unwrap();
        let buf = buf.trim().replace("\n", "").replace("\r", "");

        let regex = Regex::compile(&buf).expect("Invalid regex");

        print!("> ");
        stdout().flush().unwrap();

        stdin().lines().map_while(Result::ok).for_each(|line| {
            let it = regex.find_matches(&line);
            if it.clone().next().is_none() {
                println!("No matches");
            } else {
                println!("=== Matches ===");
                for (i,m) in it.enumerate() {
                    println!("{}) {m}", i + 1);
                }
                println!("===============");
            }
            print!("> ");
            stdout().flush().unwrap();
        });

        println!();
    }
}
