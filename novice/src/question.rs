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
    let mut shape: Vec<String> = Vec::new();
    let mut nums = Vec::new();
    let mut run: Vec<String> = Vec::new();
    let mut word = |w: String, words: &mut Vec<String>, seq: &mut Vec<String>| {
        if !words.contains(&w) {
            words.push(w.clone());
        }
        seq.push(w);
    };
    // A run that does not make a number is words after all.
    let flush = |run: &mut Vec<String>, nums: &mut Vec<i64>, words: &mut Vec<String>, seq: &mut Vec<String>, word: &mut dyn FnMut(String, &mut Vec<String>, &mut Vec<String>)| {
        if run.is_empty() {
            return;
        }
        match numerals.value_of(run) {
            Some(n) => nums.push(n),
            None => {
                for w in run.drain(..) {
                    word(w, words, seq);
                }
            }
        }
        run.clear();
    };
    for raw in question.split_whitespace() {
        let tok = raw.trim_matches(|c: char| matches!(c, '?' | ',' | '.' | '!' | ';' | ':'));
        if tok.is_empty() {
            continue;
        }
        let lower = tok.to_lowercase();
        if tok.len() <= 9 && tok.bytes().all(|b| b.is_ascii_digit()) {
            flush(&mut run, &mut nums, &mut words, &mut seq, &mut word);
            nums.push(tok.parse().unwrap_or(0));
        } else if let Some(n) = numerals.get(&lower) {
            if numerals.grammar() == Grammar::Single {
                nums.push(n);
            } else {
                run.push(lower);
            }
        } else {
            flush(&mut run, &mut nums, &mut words, &mut seq, &mut word);
            word(lower, &mut words, &mut seq);
        }
    }
    flush(&mut run, &mut nums, &mut words, &mut seq, &mut word);
    Parsed { words, seq, shape: arrangement(question, numerals), nums }
}

/// Words in order with every number as "#" (a run of number words counts once, and only if it makes a number).
fn arrangement(question: &str, numerals: &Lexicon) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut run: Vec<String> = Vec::new();
    let flush = |run: &mut Vec<String>, out: &mut Vec<String>| {
        if run.is_empty() {
            return;
        }
        if numerals.value_of(run).is_some() {
            out.push("#".to_string());
        } else {
            out.append(run);
        }
        run.clear();
    };
    for raw in question.split_whitespace() {
        let tok = raw.trim_matches(|c: char| matches!(c, '?' | ',' | '.' | '!' | ';' | ':'));
        if tok.is_empty() {
            continue;
        }
        let lower = tok.to_lowercase();
        if tok.len() <= 9 && tok.bytes().all(|b| b.is_ascii_digit()) {
            flush(&mut run, &mut out);
            out.push("#".to_string());
        } else if numerals.get(&lower).is_some() {
            if numerals.grammar() == Grammar::Single {
                out.push("#".to_string());
            } else {
                run.push(lower);
            }
        } else {
            flush(&mut run, &mut out);
            out.push(lower);
        }
    }
    flush(&mut run, &mut out);
    out
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
