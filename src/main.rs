fn main() {
    let mut args = std::env::args().into_iter();
    let _ = args.next();
    let search_term = args.next().expect("no search term provided");
    let replacement_term = args.next().expect("no replacement term provided");

    let boundary_chars = ["_", "-"];
    let boundaries_pattern = format!("[{}]", boundary_chars.join(""));
    let boundaries = regex::Regex::new(&boundaries_pattern).unwrap();
    let tokenized_search_term = boundaries.split(&search_term).collect::<Vec<&str>>();
    let tokenized_replacement_term = boundaries.split(&replacement_term).collect::<Vec<&str>>();

    let mut search_terms = vec![search_term.clone()];
    let mut replacement_terms = vec![replacement_term.clone()];
    for boundary_char in boundary_chars {
        search_terms.push(tokenized_search_term.join(boundary_char));
        replacement_terms.push(tokenized_replacement_term.join(boundary_char));
    }
    let ac = aho_corasick::AhoCorasick::new(search_terms);
    ac.stream_replace_all(std::io::stdin(), std::io::stdout(), &replacement_terms)
        .unwrap();
}
