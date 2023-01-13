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

    // Use boundary indices to break a search term into a tokenized search term.
    // Boundary characters are removed.
    let boundary_indices = find_boundary_indices(&search_term);

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

fn find_boundary_indices(search_term: &str) -> Vec<u64> {
    let mut boundary_indices: Vec<u64> = vec![];
    let mut last_is_upper_opt = None;
    let mut current_index = 0;
    for char in search_term.chars() {
        // Iterate through characters, noting each time there's a case change or boundary
        // character.
        let current_is_upper = char.is_uppercase();
        if let Some(last_is_upper) = last_is_upper_opt {
            let is_boundary = last_is_upper ^ current_is_upper;
            // NEXT: need to reset boundary when it's over
            if is_boundary {
                boundary_indices.push(current_index)
            }
        } else {
            boundary_indices.push(current_index);
        }
        last_is_upper_opt = Some(current_is_upper);
        current_index += 1;
    }
    boundary_indices
}

#[cfg(test)]
mod tests {
    use crate::find_boundary_indices;
    use pretty_assertions::assert_eq;
    #[test]
    fn test_find_boundary_indices() {
        let tests = vec![
            ("camelCase", vec![0, 5]),
            ("PascalCase", vec![0, 6]),
            ("snake_case", vec![0, 6]),
            ("kebab-case", vec![0, 6]),
            ("Title_Case", vec![0, 6]),
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
}
