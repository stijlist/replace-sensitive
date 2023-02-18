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

// Takes a string and, if it's in camelCase, PascalCase, snake_case, kebab-case, or Title_Case,
// splits it into "tokens"
// Token boundaries occur at the beginning of the string, before the capital letter when case
// changes, before and after boundary characters, and at the end of the string.
// We want to split the string before / after the boundary character but including the case change.
//
// e.g.
//     "camelCase"  => ["camel", "Case"]
//     "aCamelCase" => ["a", "Camel", "Case"]
//     "HTTPVerb"   => ["HTTP", "Verb"]
//     "PascalCase" => ["Pascal", "Case"]
//     "snake_case" => ["snake", "case"]
//     "kebab-case" => ["kebab", "case"]
//     "Title_Case" => ["Title", "Case"]
//     "CONSTANT_CASE" => ["CONSTANT", "CASE"]
//     "CONSTANT"      => ["CONSTANT"]
//     "a_constant"    => ["a", "constant"]
//     "a_Constant"    => ["a", "Constant"]
//     "A_constant"    => ["a", "Constant"]
//     "A_B"           => ["A", "B"]
//     "A_b"           => ["A", "b"]
//     "aC"            => ["a", "C"] => ["aC", "a-c", "a_c", "A_C"]
fn tokenize(search_term: &str) -> Vec<u64> {
    // let mut tokens: Vec<String> = vec![];
    // The first index is a boundary.
    let mut boundaries = vec![0];
    let mut iter = search_term.chars().peekable();
    let mut index = 0;
    iter.next();
    index += 1;
    while let Some(c) = iter.next() {
        let curr_is_uppercase = c.is_uppercase();
        let next_character = iter.peek();
        let next_character_is_case_change =
            next_character.is_some() && next_character.unwrap().is_uppercase() ^ curr_is_uppercase;
        if next_character_is_case_change {
            if curr_is_uppercase {
                boundaries.push(index);
            } else {
                boundaries.push(index + 1);
            }
            iter.next();
            index += 1;
        }
        index += 1;
    }
    // The last index is also a boundary.
    boundaries.push(index);

    // [0, 1, 2]
    // ["a", "C"]
    // for (start, end) in boundaries.tuple_windows() {
    // NEXT: figure out how to split search_term
    // tokens.push(search_term.take(
    // }

    return boundaries;
}

#[cfg(test)]
mod tests {
    use crate::tokenize;
    use pretty_assertions::assert_eq;
    #[test]
    fn test_tokenize() {
        let tests = vec![
            ("PascalCase", vec![0, 6, 10]),
            ("camelCase", vec![0, 5, 9]), // ["camelCase", "CamelCase", "camel_case", "camel-case", "Camel_Case"]
                                          // ("camelCase", vec!["camel", "Case"]), // ["camelCase", "CamelCase", "camel_case", "camel-case", "Camel_Case"]
                                          // ("PascalCase", vec!["Pascal", "Case"]),
                                          // ("snake_case", vec!["snake", "case"]),
                                          // ("kebab-case", vec!["kebab", "case"]),
                                          // ("Title_Case", vec!["Title", "Case"]),
        ];
        for test in tests {
            assert_eq!(
                tokenize(test.0),
                test.1,
                "testing {}, expected {:?}",
                test.0,
                test.1,
            );
        }
    }
}
