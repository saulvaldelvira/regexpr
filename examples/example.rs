use regexpr::Regex;

fn main() {
    /* let rule = "^(abc|cba)$"; */
    /* let r = Regex::compile(rule).unwrap(); */

    /* println!("{rule} : {r}"); */

    /* for st in ["abc", "cba", ".abc", ".cba", ".cga"] { */
    /*     println!("{st} => {}", */
    /*         r.test(st)); */
    /* } */

    let regex = Regex::compile(".*b").unwrap();
    for m in regex.find_matches("aaaaaabaaaaaab") {
        println!("{m:#?}");
    }
}
