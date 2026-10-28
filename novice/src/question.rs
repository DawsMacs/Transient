// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DawsMacs
//! Reading a question: the numbers in it, in order, and the other words, lowercased, as a set (`words`, no repeats) and as said (`seq`, every one, in
//! order). Nothing else is assumed about wording.

use crate::numerals::{Grammar, Lexicon};

/// A question taken apart.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Parsed {
    pub words: Vec<String>,
    /// The same words as said, repeats and order kept: "sed sed" is not "sed". Questions with no numbers are lookups by these.
    pub seq: Vec<String>,
    /// The question as an arrangement: its words in order, repeats kept, each number (a run of number words is one) as "#".
    pub shape: Vec<String>,
    /// For each number, in order, the form it was written in: 0 for digits, 1 and up for a family of number words.
    pub forms: Vec<u8>,
    pub nums: Vec<i64>,
}

/// Split a question into words and numbers. A number is a run of up to nine ASCII digits standing alone (so a negative number reads as a word).
pub fn parse(question: &str) -> Parsed {
    parse_with(question, &Lexicon::default())
}

/// [`parse`], where the words of `numerals` (learned from the lessons, see `numerals`) are numbers too: each on its own, or, where the lexicon
/// makes numbers from runs of words ("twenty three"), a run of consecutive number words is one number.
pub fn parse_with(question: &str, numerals: &Lexicon) -> Parsed {
    let mut words: Vec<String> = Vec::new();
    let mut seq: Vec<String> = Vec::new();
    let mut nums = Vec::new();
    let mut forms: Vec<u8> = Vec::new();
    let mut shape: Vec<String> = Vec::new();
    // A run of words of one family that make a number together; a run that does not make a number is words after all.
    let mut run: Vec<String> = Vec::new();
    let mut family = 0usize;
    fn word(w: String, words: &mut Vec<String>, seq: &mut Vec<String>, shape: &mut Vec<String>) {
        if !words.contains(&w) {
            words.push(w.clone());
        }
        seq.push(w.clone());
        shape.push(w);
    }
    fn flush(run: &mut Vec<String>, family: usize, numerals: &Lexicon, nums: &mut Vec<i64>, forms: &mut Vec<u8>, words: &mut Vec<String>, seq: &mut Vec<String>, shape: &mut Vec<String>) {
        if run.is_empty() {
            return;
        }
        match numerals.value_of_run(family, run) {
            Some(n) => {
                nums.push(n);
                forms.push(family as u8 + 1);
                shape.push("#".to_string());
            }
            None => {
                for w in run.drain(..) {
                    word(w, words, seq, shape);
                }
            }
        }
        run.clear();
    }
    for raw in question.split_whitespace() {
        let tok = raw.trim_matches(|c: char| matches!(c, '?' | ',' | '.' | '!' | ';' | ':'));
        if tok.is_empty() {
            continue;
        }
        let lower = tok.to_lowercase();
        if tok.len() <= 9 && tok.bytes().all(|b| b.is_ascii_digit()) {
            flush(&mut run, family, numerals, &mut nums, &mut forms, &mut words, &mut seq, &mut shape);
            nums.push(tok.parse().unwrap_or(0));
            forms.push(0);
            shape.push("#".to_string());
        } else if let Some((f, n)) = numerals.owner(&lower) {
            if numerals.grammar(f) == Grammar::Single {
                flush(&mut run, family, numerals, &mut nums, &mut forms, &mut words, &mut seq, &mut shape);
                nums.push(n);
                forms.push(f as u8 + 1);
                shape.push("#".to_string());
            } else {
                if !run.is_empty() && f != family {
                    flush(&mut run, family, numerals, &mut nums, &mut forms, &mut words, &mut seq, &mut shape);
                }
                family = f;
                run.push(lower);
            }
        } else {
            flush(&mut run, family, numerals, &mut nums, &mut forms, &mut words, &mut seq, &mut shape);
            word(lower, &mut words, &mut seq, &mut shape);
        }
    }
    flush(&mut run, family, numerals, &mut nums, &mut forms, &mut words, &mut seq, &mut shape);
    Parsed { words, seq, shape, forms, nums }
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
