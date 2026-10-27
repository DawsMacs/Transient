// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DawsMacs
//! Reading a question: the numbers in it, in order, and the other words, lowercased and without repeats. Nothing else is assumed about wording.

/// A question taken apart.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Parsed {
    pub words: Vec<String>,
    pub nums: Vec<i64>,
}

/// Split a question into words and numbers. A number is a run of up to nine ASCII digits standing alone (so a negative number reads as a word).
pub fn parse(question: &str) -> Parsed {
    parse_with(question, &std::collections::BTreeMap::new())
}

/// [`parse`], where the words in `numerals` (learned from the lessons, see `numerals`) are numbers too.
pub fn parse_with(question: &str, numerals: &std::collections::BTreeMap<String, i64>) -> Parsed {
    let mut words: Vec<String> = Vec::new();
    let mut nums = Vec::new();
    for raw in question.split_whitespace() {
        let tok = raw.trim_matches(|c: char| matches!(c, '?' | ',' | '.' | '!' | ';' | ':'));
        if tok.is_empty() {
            continue;
        }
        if tok.len() <= 9 && tok.bytes().all(|b| b.is_ascii_digit()) {
            nums.push(tok.parse().unwrap_or(0));
        } else if let Some(&n) = numerals.get(&tok.to_lowercase()) {
            nums.push(n);
        } else {
            let w = tok.to_lowercase();
            if !words.contains(&w) {
                words.push(w);
            }
        }
    }
    Parsed { words, nums }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_and_words_are_separated() {
        let p = parse("What is 11 fegisido 1?");
        assert_eq!(p.words, vec!["what", "is", "fegisido"]);
        assert_eq!(p.nums, vec![11, 1]);
    }

    #[test]
    fn order_of_numbers_is_kept_and_words_do_not_repeat() {
        let p = parse("7 x 7 x 3 equals");
        assert_eq!(p.nums, vec![7, 7, 3]);
        assert_eq!(p.words, vec!["x", "equals"]);
    }

    #[test]
    fn a_negative_number_or_a_very_long_one_is_a_word() {
        assert_eq!(parse("-4 and 1234567890").nums, Vec::<i64>::new());
    }
}
