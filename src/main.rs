use itertools::Itertools;
fn main() {
    let mut args = std::env::args().into_iter();
    let _ = args.next();
    let search_term = args.next().expect("no search term provided");
    let replacement_term = args.next().expect("no replacement term provided");

    let tokenized_search_term = tokenize(&search_term);
    let tokenized_replacement_term = tokenize(&replacement_term);

    let mut search_terms = vec![search_term.clone()];
    let mut replacement_terms = vec![replacement_term.clone()];
    for variant in generate_variants(tokenized_search_term) {
        search_terms.push(variant);
    }
    for variant in generate_variants(tokenized_replacement_term) {
        replacement_terms.push(variant);
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
    let mut boundaries = vec![];
    let mut iter = search_term.chars().peekable();
    let mut index = 0;
    let boundary_characters = ['_', '-'];
    while let Some(c) = iter.next() {
        // TODO: this is_first_character handling is ugly - would be good to handle it in just one
        // place.
        let is_first_character = index == 0;
        if is_first_character {
            boundaries.push(index);
        }
        let curr_is_uppercase = c.is_uppercase();
        let curr_is_boundary = boundary_characters.contains(&c);
        if curr_is_boundary {
            boundaries.push(index);
            iter.next();
            index += 1;
            boundaries.push(index);
            index += 1;
            continue;
        }
        let next_character = iter.peek();
        let next_character_is_case_change = next_character.is_some()
            && !boundary_characters.contains(next_character.unwrap())
            && next_character.unwrap().is_uppercase() ^ curr_is_uppercase;
        if next_character_is_case_change {
            if curr_is_uppercase {
                if !is_first_character {
                    boundaries.push(index);
                }
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
// Join a tokenized term back together using different strategies, e.g. PascalCase, camelCase.
fn generate_variants(tokenized_term: Vec<&str>) -> Vec<String> {
    use CapitalizationStrategy::*;
    let strategies = vec![
        // camelCase
        Strategy {
            capitalization: vec![RestTokensCapitalized],
            joiner: "",
        },
        // PascalCase
        Strategy {
            capitalization: vec![FirstTokenCapitalized, RestTokensCapitalized],
            joiner: "",
        },
        // snake_case
        Strategy {
            capitalization: vec![NoCharactersCapitalized],
            joiner: "_",
        },
        // kebab-case
        Strategy {
            capitalization: vec![NoCharactersCapitalized],
            joiner: "-",
        },
        // Title_Case
        Strategy {
            capitalization: vec![FirstTokenCapitalized, RestTokensCapitalized],
            joiner: "_",
        },
        // CONSTANT_CASE
        Strategy {
            capitalization: vec![AllCharactersCapitalized],
            joiner: "_",
        },
    ];
    let mut results = vec![];
    for strategy in strategies {
        let capitalization = strategy.capitalization;
        // TODO: do this at the type level?
        if capitalization.contains(&NoCharactersCapitalized) && capitalization.len() > 1 {
            panic!("unsatisfiable: {:?}", capitalization);
        }
        let mut result = String::from("");
        for (index, token) in tokenized_term.clone().iter().enumerate() {
            let first_token = index == 0;
            let mut first_char = true;
            use unicode_segmentation::UnicodeSegmentation;
            for char in token.graphemes(true) {
                if capitalization.contains(&NoCharactersCapitalized) {
                    result.push_str(&char.to_lowercase());
                    continue;
                }

                let should_be_uppercase = capitalization.contains(&AllCharactersCapitalized)
                    || (first_char
                        && ((capitalization.contains(&FirstTokenCapitalized) && first_token)
                            || (capitalization.contains(&RestTokensCapitalized) && !first_token)));
                if should_be_uppercase {
                    result.push_str(&char.to_uppercase());
                } else {
                    result.push_str(&char.to_lowercase());
                }
                first_char = false;
            }
            let last_token = index == tokenized_term.len() - 1;
            if !last_token {
                result.push_str(strategy.joiner);
            }
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
            ("CONSTANT_CASE", vec!["CONSTANT", "CASE"]),
            ("A_B", vec!["A", "B"]),
            ("A_b", vec!["A", "b"]),
            ("aC", vec!["a", "C"]),
            ("a-c", vec!["a", "c"]),
            ("a_c", vec!["a", "c"]),
            ("a-C", vec!["a", "C"]),
            ("A-c", vec!["A", "c"]),
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
        let tests = vec![
            (
                vec!["all", "cases", "covered"],
                vec![
                    "allCasesCovered",
                    "AllCasesCovered",
                    "all_cases_covered",
                    "all-cases-covered",
                    "All_Cases_Covered",
                    "ALL_CASES_COVERED",
                ],
            ),
            (
                vec!["AlL", "cAsES", "cOvErED"],
                vec![
                    "allCasesCovered",
                    "AllCasesCovered",
                    "all_cases_covered",
                    "all-cases-covered",
                    "All_Cases_Covered",
                    "ALL_CASES_COVERED",
                ],
            ),
        ];
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
