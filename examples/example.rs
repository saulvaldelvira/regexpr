use regexpr::Regex;

fn main() {
    /* let rule = "^(abc|cba)$"; */
    /* let r = Regex::compile(rule).unwrap(); */

    /* println!("{rule} : {r}"); */

    /* for st in ["abc", "cba", ".abc", ".cba", ".cga"] { */
    /*     println!("{st} => {}", */
    /*         r.test(st)); */
    /* } */

    /* let regex = Regex::compile(".*b").unwrap(); */
    /* for m in regex.find_matches("aaaaaabaaaaaab") { */
    /*     println!("{m:#?}"); */
    /* } */

    /* let regex = Regex::compile(".*?b").unwrap(); */
    /* assert_eq!(2, regex.find_matches("aabaab").count()); */

    let regex = Regex::compile(".*b").unwrap();
    let mut m = regex.find_matches("aabaab");
    let m = m.next();
    for ma in m {
        println!("MATCH: {ma:#?}");
    }
    /* assert_eq!(1, regex.find_matches("aabaab").count()); */
}
