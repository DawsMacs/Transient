// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DawsMacs
//! Which of the numbers in a question matter.
//!
//! A word problem often holds more numbers than its rule uses ("Jerry has 5 apples and 3 pens, then buys 8 more apples: how many apples?"); the
//! programs the baby builds take one or two. This learns, from lessons where it knows which numbers were used, a score for each number in a
//! text, and `focus` keeps the best ones and blanks the others so the rest of the baby sees an ordinary one- or two-number question.
//!
//! What a number is scored on is general, not specific to any corpus: the words just before and after it, where it stands, and above all whether
//! the words next to it also occur in the question (the last sentence, found from the text itself: after the last sentence boundary before the
//! final question mark). A distractor is usually about a different thing than the one asked about. Learned by an averaged perceptron over the
//! examples, in a fixed-seed shuffle, so it is deterministic and needs no dependencies.

use crate::data::tokenize;
use crate::quantity::NumberLine;
use crate::rng::Rng;
use std::collections::{HashMap, HashSet};

/// Words that say nothing about what a number counts (function words); not compared with the question.
const FUNCTION_WORDS: [&str; 52] = [
    "the", "a", "an", "of", "to", "in", "on", "is", "are", "was", "were", "how", "many", "much", "do", "does", "did", "have", "has", "had", "that", "this", "and", "or",
    "for", "with", "at", "by", "from", "it", "its", "they", "their", "he", "she", "his", "her", "there", "what", "if", "than", "more", "less", "will", "would", "can",
    "could", "all", "each", "per", "now", "then",
];

/// The word a blanked number becomes: a word, not a number, and the same everywhere.
pub const BLANK: &str = "numx";

const EPOCHS: usize = 12;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Selector {
    weights: HashMap<String, f32>,
    /// How many examples it learned from (0: it knows nothing and `focus` leaves a question alone).
    pub examples: usize,
}

/// The tokens of a text, and where the numbers are among them.
struct Read {
    tokens: Vec<String>,
    numbers: Vec<usize>,
}

fn read(text: &str, line: &NumberLine) -> Read {
    let tokens = tokenize(text);
    let numbers = (0..tokens.len()).filter(|&i| line.value(&tokens[i]).is_some()).collect();
    Read { tokens, numbers }
}

/// The last sentence of a text: after the last sentence boundary that comes before the final question mark (or the whole text's tail if there is
/// none). The question of a story problem is what decides what to do with its numbers.
pub fn question_of(text: &str) -> &str {
    let head = match text.rfind('?') {
        Some(j) => &text[..j],
        None => text,
    };
    match head.rfind(['.', '!', '?']) {
        Some(k) => &text[k + 1..],
        None => text,
    }
}

fn is_content(w: &str, line: &NumberLine) -> bool {
    !FUNCTION_WORDS.contains(&w) && line.value(w).is_none()
}

fn features(r: &Read, which: usize, question: &HashSet<String>, line: &NumberLine) -> Vec<String> {
    let at = r.numbers[which];
    let n = r.numbers.len();
    let tag = |w: &str| if line.value(w).is_some() { "NUM".to_string() } else { w.to_string() };
    let mut f: Vec<String> = Vec::new();
    for off in 1..=3usize {
        if at + off < r.tokens.len() {
            f.push(format!("R{off}:{}", tag(&r.tokens[at + off])));
        }
        if at >= off {
            f.push(format!("L{off}:{}", tag(&r.tokens[at - off])));
        }
    }
    f.push(format!("pos:{which}/{n}"));
    f.push(if which + 1 == n { "last".into() } else { "notlast".into() });
    f.push(if which == 0 { "first".into() } else { "notfirst".into() });
    for w in &r.tokens[at.saturating_sub(5)..at] {
        f.push(format!("Lbag:{w}"));
    }
    for w in r.tokens.iter().skip(at + 1).take(5) {
        f.push(format!("Rbag:{w}"));
    }
    // does what the number counts also appear in the question?
    let near: Vec<&String> = r.tokens.iter().skip(at + 1).take(3).filter(|w| is_content(w, line)).collect();
    let left: Vec<&String> = r.tokens[at.saturating_sub(4)..at].iter().filter(|w| is_content(w, line)).collect();
    f.push(format!("nearInQ:{}", near.iter().filter(|w| question.contains(w.as_str())).count().min(2)));
    f.push(format!("leftInQ:{}", left.iter().filter(|w| question.contains(w.as_str())).count().min(2)));
    f.push(if near.first().map_or(false, |w| question.contains(w.as_str())) { "near1InQ".into() } else { "near1NotQ".into() });
    f.push(if near.iter().chain(left.iter()).any(|w| question.contains(w.as_str())) { "anyInQ".into() } else { "noneInQ".into() });
    f
}

fn question_words(text: &str, line: &NumberLine) -> HashSet<String> {
    tokenize(question_of(text)).into_iter().filter(|w| is_content(w, line)).collect()
}

impl Selector {
    /// Saved as `selector N` followed by N lines `sw <weight> <feature>` (features hold no spaces).
    pub fn to_lines(&self) -> String {
        let mut keys: Vec<&String> = self.weights.keys().collect();
        keys.sort();
        let mut out = format!("selector {} {}\n", self.examples, keys.len());
        for k in keys {
            out.push_str(&format!("sw {} {}\n", self.weights[k], k));
        }
        out
    }

    /// Read what `to_lines` wrote (the next line must start with `selector`); None if it does not, or is damaged.
    pub fn from_lines<'a>(lines: &mut std::iter::Peekable<impl Iterator<Item = &'a str>>) -> Option<Selector> {
        let head = lines.peek()?.strip_prefix("selector ")?;
        let mut parts = head.split_whitespace();
        let examples: usize = parts.next()?.parse().ok()?;
        let n: usize = parts.next()?.parse().ok()?;
        lines.next();
        let mut weights = HashMap::new();
        for _ in 0..n {
            let l = lines.next()?.strip_prefix("sw ")?;
            let (w, k) = l.split_once(' ')?;
            weights.insert(k.to_string(), w.parse().ok()?);
        }
        Some(Selector { weights, examples })
    }

    /// Learn from examples: a text and the positions (among its numbers, counting from 0) of the numbers that were used.
    pub fn train(examples: &[(String, Vec<usize>)], line: &NumberLine) -> Selector {
        let mut samples: Vec<(Vec<String>, f32)> = Vec::new();
        for (text, used) in examples {
            let r = read(text, line);
            let q = question_words(text, line);
            for which in 0..r.numbers.len() {
                samples.push((features(&r, which, &q, line), if used.contains(&which) { 1.0 } else { -1.0 }));
            }
        }
        let mut w: HashMap<String, f32> = HashMap::new();
        let mut acc: HashMap<String, f32> = HashMap::new();
        let mut t = 1.0f32;
        let mut rng = Rng::new(0x5E1EC7 ^ samples.len() as u64);
        let mut order: Vec<usize> = (0..samples.len()).collect();
        for _ in 0..EPOCHS {
            rng.shuffle(&mut order);
            for &i in &order {
                let (f, y) = &samples[i];
                let score: f32 = f.iter().map(|k| w.get(k).copied().unwrap_or(0.0)).sum();
                if y * score <= 0.0 {
                    for k in f {
                        *w.entry(k.clone()).or_default() += y;
                        *acc.entry(k.clone()).or_default() += y * t;
                    }
                }
                t += 1.0;
            }
        }
        let weights = w.into_iter().map(|(k, v)| { let a = acc.get(&k).copied().unwrap_or(0.0); (k, v - a / t) }).collect();
        Selector { weights, examples: examples.len() }
    }

    /// The score of each number in the text, in order.
    pub fn scores(&self, text: &str, line: &NumberLine) -> Vec<f32> {
        let r = read(text, line);
        let q = question_words(text, line);
        (0..r.numbers.len()).map(|which| features(&r, which, &q, line).iter().map(|k| self.weights.get(k).copied().unwrap_or(0.0)).sum()).collect()
    }

    /// The text with all but the `keep` best-scoring numbers replaced by `BLANK` (the numbers kept stay in their places). None if the text has no
    /// more numbers than that, or the selector has learned nothing: the text is then as it was.
    pub fn focus(&self, text: &str, line: &NumberLine, keep: usize) -> Option<String> {
        if self.examples == 0 {
            return None;
        }
        let r = read(text, line);
        if r.numbers.len() <= keep {
            return None;
        }
        let scores = self.scores(text, line);
        let mut ranked: Vec<usize> = (0..scores.len()).collect();
        // (a tie keeps the later number: a story usually ends with what it asks about)
        ranked.sort_by(|&a, &b| scores[b].partial_cmp(&scores[a]).unwrap_or(std::cmp::Ordering::Equal).then(b.cmp(&a)));
        let kept: HashSet<usize> = ranked.into_iter().take(keep).map(|k| r.numbers[k]).collect();
        let out: Vec<String> = r
            .tokens
            .iter()
            .enumerate()
            .map(|(i, t)| if line.value(t).is_some() && !kept.contains(&i) { BLANK.to_string() } else { t.clone() })
            .collect();
        Some(out.join(" "))
    }
}

/// The positions (among the numbers of `text`) of the pair whose combination `rule` says equals `answer`, if exactly one pair does (then there
/// is no doubt which numbers were used). `rule` combines two numbers; it must not care about their order (the baby's programs for adding,
/// subtracting by distance, multiplying and dividing larger by smaller do not), so each unordered pair is tried once.
pub fn used_pair(text: &str, answer: f32, line: &NumberLine, rule: &dyn Fn(f32, f32) -> f32) -> Option<Vec<usize>> {
    let r = read(text, line);
    let values: Vec<f32> = r.numbers.iter().filter_map(|&i| line.value(&r.tokens[i])).collect();
    let mut found: Vec<(usize, usize)> = Vec::new();
    for i in 0..values.len() {
        for j in (i + 1)..values.len() {
            let v = rule(values[i], values[j]);
            if v.is_finite() && (v - answer).abs() < 0.01 {
                found.push((i, j));
            }
        }
    }
    // two pairs made of equal numbers in different places are the same choice as far as the answer goes, but not as far as the words go: skip
    (found.len() == 1).then(|| vec![found[0].0, found[0].1])
}

/// The text (as the baby reads it) with every number blanked except those at the given positions among its numbers.
pub fn keep_only(text: &str, keep: &[usize], line: &crate::quantity::NumberLine) -> String {
    let tokens = tokenize(text);
    let mut k = 0usize;
    tokens
        .iter()
        .map(|t| {
            if line.value(t).is_some() {
                let here = keep.contains(&k);
                k += 1;
                if here { t.clone() } else { BLANK.to_string() }
            } else {
                t.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line() -> NumberLine {
        NumberLine::from_counting(&crate::quantity::counting_lessons())
    }

    #[test]
    fn the_question_is_the_last_sentence_before_the_final_question_mark() {
        assert_eq!(question_of("Tom has 5 pens. He lost 2 pens. How many pens are left?").trim(), "How many pens are left?");
        assert_eq!(question_of("Tom has 5 pens").trim(), "Tom has 5 pens");
    }

    /// Stories where the number that counts is the one whose noun is the one asked about, learned from examples; the unseen story is blanked right.
    #[test]
    fn it_learns_that_the_number_next_to_the_word_in_the_question_is_the_one_that_counts() {
        let nouns = ["apples", "pears", "plums", "cherries", "grapes", "lemons", "limes", "peaches", "figs", "dates"];
        let others = ["pens", "cups", "hats", "books", "coins", "shoes", "keys", "cards", "maps", "toys"];
        let mut examples = Vec::new();
        for i in 0..80usize {
            let (n, o) = (nouns[i % 10], others[(i / 3) % 10]);
            let (a, b, c) = (2 + i % 7, 3 + (i / 2) % 6, 1 + (i / 5) % 8);
            // the noun asked about carries two numbers; a third number belongs to something else, in a varying place
            let text = match i % 3 {
                0 => format!("Sam has {a} {n} and {c} {o}. He buys {b} more {n}. How many {n} does Sam have?"),
                1 => format!("Sam has {c} {o}. Sam has {a} {n}. Then he buys {b} more {n}. How many {n} does Sam have now?"),
                _ => format!("Sam has {a} {n}. He buys {b} more {n} and also {c} {o}. How many {n} does Sam have?"),
            };
            let positions: Vec<usize> = match i % 3 {
                0 => vec![0, 2],
                1 => vec![1, 2],
                _ => vec![0, 1],
            };
            examples.push((text, positions));
        }
        let l = line();
        let sel = Selector::train(&examples, &l);
        let unseen = "Zoe has 6 kiwis. She has 4 spoons. She buys 5 more kiwis. How many kiwis does Zoe have?";
        let focused = sel.focus(unseen, &l, 2).expect("three numbers, keep two");
        assert!(focused.contains("six") || focused.contains(" 6 "), "{focused}");
        assert_eq!(focused.matches(BLANK).count(), 1, "{focused}");
        assert!(!focused.contains(" 4 ") && focused.contains(BLANK), "the spoons' number is the one blanked: {focused}");
        // nothing learned: left alone; few enough numbers: left alone
        assert!(Selector::default().focus(unseen, &l, 2).is_none());
        assert!(sel.focus("Zoe has 6 kiwis. She buys 5 more. How many?", &l, 2).is_none());
    }

    #[test]
    fn a_selector_survives_being_written_and_read_back() {
        let l = line();
        let ex: Vec<(String, Vec<usize>)> = (0..30usize).map(|i| (format!("Sam has {} apples and {} pens. How many apples?", 2 + i % 5, 3 + i % 4), vec![0])).collect();
        let sel = Selector::train(&ex, &l);
        let text = sel.to_lines();
        let back = Selector::from_lines(&mut text.lines().peekable()).expect("reads back");
        assert_eq!(sel, back);
        assert!(Selector::from_lines(&mut "units 3".lines().peekable()).is_none());
    }

    #[test]
    fn only_a_single_unambiguous_pair_is_taken_as_the_numbers_that_were_used() {
        let l = line();
        let apart = |a: f32, b: f32| (a - b).abs();
        // 9 - 4 = 5 is the only pair
        assert_eq!(used_pair("he had 9 coins and 4 hats and 20 books", 5.0, &l, &apart), Some(vec![0, 1]));
        // 9, 4, 5: both 9-4 and 9-5=4 ... answer 5 is made by (9,4) only; answer 4 by (9,5) only; answer 1 by (5,4) only; but 20 - 15 and 25 - 20 give 5 twice
        assert!(used_pair("20 and 15 and 25", 5.0, &l, &apart).is_none(), "two pairs make 5: no label");
        assert!(used_pair("he had 9 coins", 5.0, &l, &apart).is_none());
    }
}
