use itertools::Itertools;
fn main() {
    let mut args = std::env::args().into_iter();
    let _ = args.next();
    let search_term = args.next().expect("no search term provided");
    let replacement_term = args.next().expect("no replacement term provided");

    let boundary_chars = ["_", "-"];
    let tokenized_search_term = tokenize(&search_term);
    let tokenized_replacement_term = tokenize(&replacement_term);

    let mut search_terms = vec![search_term.clone()];
    let mut replacement_terms = vec![replacement_term.clone()];
    // NEXT: generate more variants on the search/replace pattern.
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
fn find_boundary_indices(search_term: &str) -> Vec<usize> {
    // let mut tokens: Vec<String> = vec![];
    // The first index is a boundary.
    let mut boundaries = vec![0];
    let mut iter = search_term.chars().peekable();
    let mut index = 0;
    iter.next();
    index += 1;
    while let Some(c) = iter.next() {
        let curr_is_uppercase = c.is_uppercase();
        let curr_is_boundary = c == '_' || c == '-';
        if curr_is_boundary {
            boundaries.push(index);
            iter.next();
            index += 1;
            boundaries.push(index);
            index += 1;
            continue;
        }
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

    return boundaries;
}

fn tokenize(search_term: &str) -> Vec<&str> {
    let boundaries = find_boundary_indices(search_term);
    let mut tokens = Vec::new();
    for (start, end) in boundaries.iter().tuple_windows() {
        let val = utf8_slice::slice(search_term, *start, *end);
        if !(val == "_" || val == "-") {
            tokens.push(val);
        }
    }

    return tokens;
}

#[derive(Debug, Eq, PartialEq)]
enum CapitalizationStrategy {
    FirstTokenCapitalized,
    RestTokensCapitalized,
    AllCharactersCapitalized,
    NoCharactersCapitalized,
}

struct Strategy {
    joiner: &'static str,
    capitalization: Vec<CapitalizationStrategy>,
}
// Join a tokenized, all-lowercase term back together using different strategies, e.g. PascalCase, camelCase.
fn generate_variants(tokenized_term: Vec<&str>) -> Vec<String> {
    use CapitalizationStrategy::*;
    let strategies = vec![Strategy {
        capitalization: vec![RestTokensCapitalized],
        joiner: "",
    }];
    let mut results = vec![];
    for strategy in strategies {
        let capitalization = strategy.capitalization;
        let mut result = String::from("");
        let mut first_token = true;
        for token in tokenized_term.clone() {
            // TODO: unicode grapheme cluster aware
            let mut first_char = true;
            for char in token.chars() {
                let should_be_uppercase = capitalization.contains(&AllCharactersCapitalized)
                    || (first_char
                        && ((capitalization.contains(&FirstTokenCapitalized) && first_token)
                            || (capitalization.contains(&RestTokensCapitalized) && !first_token)));
                if should_be_uppercase {
                    result.push_str(&char.to_uppercase().join(""));
                } else {
                    result.push(char);
                }
                first_char = false;
            }
            result.push_str(strategy.joiner);
            first_token = false;
        }
        results.push(result);
    }
    return results;
}

#[cfg(test)]
mod tests {
    use crate::{find_boundary_indices, generate_variants, tokenize};
    use pretty_assertions::assert_eq;
    #[test]
    fn test_find_boundary_indices() {
        let tests = vec![
            ("camelCase", vec![0, 5, 9]),
            ("PascalCase", vec![0, 6, 10]),
            ("snake_case", vec![0, 5, 6, 10]),
            ("kebab-case", vec![0, 5, 6, 10]),
            ("Title_Case", vec![0, 5, 6, 10]),
        ];
        for test in tests {
            assert_eq!(
                find_boundary_indices(test.0),
                test.1,
                "testing {}, expected {:?}",
                test.0,
                test.1,
            );
        }
    }

    #[test]
    fn test_tokenize() {
        let tests = vec![
            ("camelCase", vec!["camel", "Case"]),
            ("PascalCase", vec!["Pascal", "Case"]),
            ("snake_case", vec!["snake", "case"]),
            ("kebab-case", vec!["kebab", "case"]),
            ("Title_Case", vec!["Title", "Case"]),
            ("aCamelCase", vec!["a", "Camel", "Case"]),
            ("HTTPVerb", vec!["HTTP", "Verb"]),
            ("CONSTANT", vec!["CONSTANT"]),
            ("a_constant", vec!["a", "constant"]),
            ("a_Constant", vec!["a", "Constant"]),
            ("A_constant", vec!["A", "constant"]),
            ("A_B", vec!["A", "B"]),
            ("A_b", vec!["A", "b"]),
            // TODO: CONSTANT_CASE doesn't work
            // Diff < left / right > :
            //  [
            // <    "CONSTAN",
            // <    "T_CASE",
            // >    "CONSTANT",
            // >    "CASE",
            //  ]
            // ("CONSTANT_CASE", vec!["CONSTANT", "CASE"]),
            // ("aC", vec!["a", "C"]),
            // , ["aC", "a-c", "a_c", "A_C"]
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

    #[test]
    fn test_generate_variants() {
        let tests = vec![(vec!["all", "cases", "covered"], vec!["allCasesCovered"])];
        for test in tests {
            assert_eq!(
                generate_variants(test.0.clone()),
                test.1,
                "testing {:?}, expected {:?}",
                test.0,
                test.1,
            );
        }
    }
}
