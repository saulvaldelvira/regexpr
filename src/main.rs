#[cfg(feature = "gui")]
mod gui;

#[cfg(not(feature = "gui"))]
fn start_tui() {
    use regexpr::Regex;
    use std::io::{Write, stdin, stdout};
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
            let mut it = regex.find_matches(&line);
            if it.clone().next().is_none() {
                println!("No matches");
            } else {
                println!("=== Matches ===");
                for (i, m) in (&mut it).enumerate() {
                    println!("{}) {m}", i + 1);
                }
                if it.get_groups().iter().any(|l| !l.is_empty()) {
                    println!("===== Groups ======");
                    for (i, m) in it.get_groups().iter().enumerate() {
                        println!("{}) \"{m}\"", i + 1);
                    }
                }
                println!("===================");
            }
            print!("> ");
            stdout().flush().unwrap();
        });

        println!();
    }
}

pub fn main() {
    #[cfg(feature = "gui")]
    gui::start_gui().unwrap_or_else(|err| {
        eprintln!("ERROR: {err}");
        std::process::exit(1);
    });

    #[cfg(not(feature = "gui"))]
    start_tui();
}
