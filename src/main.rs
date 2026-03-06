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
            #[allow(unused)]
            let time = std::time::Instant::now();
            let mut it = regex.find_matches(&line);
            if it.clone().next().is_none() {
                println!("No matches");
            } else {
                for (i, m) in (&mut it).enumerate() {
                    println!("{}) {m}", i + 1);
                    if m.get_captures().iter().any(|l| !l.is_empty()) {
                        println!("  Captures:");
                        for (i, m) in m.get_captures().iter().enumerate() {
                            println!("  {}) \"{m}\"", i + 1);
                        }
                    }
                }
            }
            #[cfg(debug_assertions)]
            {
                let elapsed = time.elapsed();
                let ms = elapsed.as_millis();
                if ms > 0 {
                    println!("Time: {ms} ms");
                } else {
                    println!("Time: {} \u{00B5}s", elapsed.as_micros());
                }
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
