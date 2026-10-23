// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DawsMacs
//! The baby's world: a few kinds of little questions, each with a one-word answer, in several phrasings.
//!
//! The data is generated in code (not downloaded), deterministically from a seed, so every experiment is reproducible and the
//! splits are inspectable. Each family has four phrasings (templates). Templates 0-2 are used for training; template 3 is a
//! recombination of words the net has already seen, held out to test whether it copes with a *new phrasing of a known thing*.
//! For arithmetic a few whole sums are also held out, to test whether it learned the rule rather than the table.

use crate::rng::Rng;

/// Word slots per query. Longer queries are cut, shorter ones padded.
pub const LEN: usize = 8;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Family {
    Color,
    Sound,
    Opposite,
    Capital,
    Add,
    Sub,
}

impl Family {
    pub const ALL: [Family; 6] =
        [Family::Color, Family::Sound, Family::Opposite, Family::Capital, Family::Add, Family::Sub];

    pub fn name(self) -> &'static str {
        match self {
            Family::Color => "color",
            Family::Sound => "sound",
            Family::Opposite => "opposite",
            Family::Capital => "capital",
            Family::Add => "add",
            Family::Sub => "sub",
        }
    }

    /// Order in which a child-like curriculum introduces them.
    pub fn stage(self) -> usize {
        match self {
            Family::Color | Family::Sound => 1,
            Family::Opposite => 2,
            Family::Capital => 3,
            Family::Add | Family::Sub => 4,
        }
    }
}

pub const STAGES: usize = 4;

#[derive(Clone, Debug)]
pub struct Example {
    pub family: Family,
    pub template: usize,
    pub query: String,
    pub answer: String,
}

pub struct Dataset {
    /// Seen items, templates 0-2.
    pub train: Vec<Example>,
    /// Seen items, the held-out phrasing (template 3).
    pub paraphrase_test: Vec<Example>,
    /// Sums never seen in any phrasing (arithmetic only).
    pub item_test: Vec<Example>,
    /// The training questions with one spelling mistake in one word: does it still understand?
    pub typo_test: Vec<Example>,
    /// Same, but the misspelled word is always the one that carries the meaning (the thing asked about, or a number).
    pub typo_entity_test: Vec<Example>,
}

struct Spec {
    family: Family,
    templates: [&'static str; 4],
    items: Vec<(Vec<String>, String)>,
}

pub const NUM: [&str; 11] = ["zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten"];

pub const COLORS: &[(&str, &str)] = &[
    ("sky", "blue"), ("grass", "green"), ("banana", "yellow"), ("snow", "white"), ("coal", "black"),
    ("blood", "red"), ("sun", "yellow"), ("milk", "white"), ("tomato", "red"), ("chocolate", "brown"),
    ("lemon", "yellow"), ("cloud", "white"), ("night", "black"), ("leaf", "green"), ("ocean", "blue"),
    ("strawberry", "red"), ("carrot", "orange"), ("plum", "purple"), ("flamingo", "pink"), ("tree", "brown"),
];

pub const SOUNDS: &[(&str, &str)] = &[
    ("dog", "woof"), ("cat", "meow"), ("cow", "moo"), ("duck", "quack"), ("sheep", "baa"), ("pig", "oink"),
    ("horse", "neigh"), ("lion", "roar"), ("owl", "hoot"), ("snake", "hiss"), ("bee", "buzz"), ("frog", "ribbit"),
    ("mouse", "squeak"), ("bird", "tweet"),
];

pub const OPPOSITES: &[(&str, &str)] = &[
    ("hot", "cold"), ("big", "small"), ("up", "down"), ("day", "night"), ("fast", "slow"), ("happy", "sad"),
    ("open", "close"), ("in", "out"), ("old", "young"), ("hard", "soft"), ("wet", "dry"), ("full", "empty"),
    ("light", "dark"), ("long", "short"), ("high", "low"), ("loud", "quiet"), ("good", "bad"), ("yes", "no"),
    ("left", "right"), ("push", "pull"),
];

pub const CAPITALS: &[(&str, &str)] = &[
    ("france", "paris"), ("italy", "rome"), ("spain", "madrid"), ("germany", "berlin"), ("japan", "tokyo"),
    ("egypt", "cairo"), ("china", "beijing"), ("russia", "moscow"), ("canada", "ottawa"), ("greece", "athens"),
    ("peru", "lima"), ("cuba", "havana"), ("kenya", "nairobi"), ("norway", "oslo"), ("ireland", "dublin"),
    ("portugal", "lisbon"), ("india", "delhi"), ("austria", "vienna"), ("sweden", "stockholm"), ("turkey", "ankara"),
];

fn singles(pairs: &[(&str, &str)]) -> Vec<(Vec<String>, String)> {
    pairs.iter().map(|(q, a)| (vec![q.to_string()], a.to_string())).collect()
}

fn specs() -> Vec<Spec> {
    let mut opposites = singles(OPPOSITES);
    opposites.extend(OPPOSITES.iter().map(|(a, b)| (vec![b.to_string()], a.to_string())));

    let mut add = Vec::new();
    let mut sub = Vec::new();
    for a in 0..=5usize {
        for b in 0..=5usize {
            add.push((vec![NUM[a].to_string(), NUM[b].to_string()], NUM[a + b].to_string()));
            if b <= a {
                sub.push((vec![NUM[a].to_string(), NUM[b].to_string()], NUM[a - b].to_string()));
            }
        }
    }

    vec![
        Spec {
            family: Family::Color,
            templates: ["what color is the {0}", "color of {0}", "the {0} is what color", "what is the color of {0}"],
            items: singles(COLORS),
        },
        Spec {
            family: Family::Sound,
            templates: ["what does the {0} say", "{0} sound", "what sound does the {0} make", "what does the {0} make"],
            items: singles(SOUNDS),
        },
        Spec {
            family: Family::Opposite,
            templates: ["opposite of {0}", "what is the opposite of {0}", "{0} opposite", "the opposite of {0}"],
            items: opposites,
        },
        Spec {
            family: Family::Capital,
            templates: ["capital of {0}", "what is the capital of {0}", "{0} capital", "the capital of {0}"],
            items: singles(CAPITALS),
        },
        Spec {
            family: Family::Add,
            templates: ["what is {0} plus {1}", "{0} plus {1}", "{0} plus {1} equals", "what is {0} plus {1} equals"],
            items: add,
        },
        Spec {
            family: Family::Sub,
            templates: ["what is {0} minus {1}", "{0} minus {1}", "{0} minus {1} equals", "what is {0} minus {1} equals"],
            items: sub,
        },
    ]
}

fn fill(template: &str, slots: &[String]) -> String {
    let mut s = template.to_string();
    for (i, slot) in slots.iter().enumerate() {
        s = s.replace(&format!("{{{i}}}"), slot);
    }
    s
}

/// Lowercase, drop punctuation, split into words.
pub fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .map(|w| w.to_string())
        .collect()
}

pub fn build(seed: u64) -> Dataset {
    let mut rng = Rng::new(seed);
    let mut ds = Dataset { train: Vec::new(), paraphrase_test: Vec::new(), item_test: Vec::new(), typo_test: Vec::new(), typo_entity_test: Vec::new() };
    for spec in specs() {
        let n = spec.items.len();
        let mut order: Vec<usize> = (0..n).collect();
        rng.shuffle(&mut order);
        let n_hold = match spec.family {
            Family::Add | Family::Sub => ((n as f32) * 0.15).round() as usize,
            _ => 0,
        };
        let held: Vec<usize> = order[..n_hold].to_vec();
        for (idx, (slots, answer)) in spec.items.iter().enumerate() {
            for (t, template) in spec.templates.iter().enumerate() {
                let ex = Example { family: spec.family, template: t, query: fill(template, slots), answer: answer.clone() };
                if held.contains(&idx) {
                    ds.item_test.push(ex);
                } else if t < 3 {
                    ds.train.push(ex);
                } else {
                    ds.paraphrase_test.push(ex);
                }
            }
        }
    }
    ds.typo_test = typos_of(&ds.train, seed.wrapping_add(999), None);
    let entities = entity_words();
    ds.typo_entity_test = typos_of(&ds.train, seed.wrapping_add(1999), Some(&entities));
    ds
}

/// Every word that can fill a question's blank: the things asked about, and the number words.
pub fn entity_words() -> std::collections::HashSet<String> {
    let mut set: std::collections::HashSet<String> = std::collections::HashSet::new();
    for list in [COLORS, SOUNDS, OPPOSITES, CAPITALS] {
        for (q, _) in list {
            set.insert(q.to_string());
        }
    }
    for (_, a) in OPPOSITES {
        set.insert(a.to_string());
    }
    for n in NUM {
        set.insert(n.to_string());
    }
    set
}

/// Sums and differences outside the range the baby is ever taught (operands up to ten, results up to ten, at least one operand
/// above five), in all four phrasings. A rule extends to these; a memorized table cannot, because it never met the words.
pub fn extrapolation() -> Vec<Example> {
    let templates = |f: Family| -> [&'static str; 4] {
        specs().into_iter().find(|s| s.family == f).map(|s| s.templates).unwrap()
    };
    let (add_t, sub_t) = (templates(Family::Add), templates(Family::Sub));
    let mut out = Vec::new();
    for a in 0..=10usize {
        for b in 0..=10usize {
            if a <= 5 && b <= 5 {
                continue;
            }
            let slots = vec![NUM[a].to_string(), NUM[b].to_string()];
            for (t, template) in add_t.iter().enumerate() {
                if a + b <= 10 {
                    out.push(Example { family: Family::Add, template: t, query: fill(template, &slots), answer: NUM[a + b].to_string() });
                }
            }
            for (t, template) in sub_t.iter().enumerate() {
                if b <= a {
                    out.push(Example { family: Family::Sub, template: t, query: fill(template, &slots), answer: NUM[a - b].to_string() });
                }
            }
        }
    }
    out
}

/// One spelling mistake: drop a letter, swap two neighbors, or replace a letter.
fn misspell(word: &str, rng: &mut Rng) -> String {
    let chars: Vec<char> = word.chars().collect();
    loop {
        let mut c = chars.clone();
        match rng.below(3) {
            0 => {
                c.remove(1 + rng.below(c.len() - 1));
            }
            1 => {
                let i = rng.below(c.len() - 1);
                c.swap(i, i + 1);
            }
            _ => {
                let i = rng.below(c.len());
                c[i] = (b'a' + rng.below(26) as u8) as char;
            }
        }
        let out: String = c.into_iter().collect();
        if out != word {
            return out;
        }
    }
}

/// The same questions with one misspelled word each (a word of four letters or more; shorter questions are left out).
pub fn typos_of(examples: &[Example], seed: u64, only: Option<&std::collections::HashSet<String>>) -> Vec<Example> {
    let mut rng = Rng::new(seed);
    let mut out = Vec::new();
    for e in examples {
        let mut words = tokenize(&e.query);
        let long: Vec<usize> = (0..words.len())
            .filter(|&i| words[i].chars().count() >= 4 && only.map_or(true, |set| set.contains(&words[i])))
            .collect();
        if long.is_empty() {
            continue;
        }
        let i = long[rng.below(long.len())];
        words[i] = misspell(&words[i], &mut rng);
        out.push(Example { family: e.family, template: e.template, query: words.join(" "), answer: e.answer.clone() });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn queries_fit_the_slots() {
        let ds = build(1);
        for ex in ds.train.iter().chain(&ds.paraphrase_test).chain(&ds.item_test) {
            assert!(tokenize(&ex.query).len() <= LEN, "too long: {}", ex.query);
        }
    }

    #[test]
    fn splits_do_not_leak() {
        let ds = build(1);
        let train: HashSet<&str> = ds.train.iter().map(|e| e.query.as_str()).collect();
        for ex in ds.paraphrase_test.iter().chain(&ds.item_test) {
            assert!(!train.contains(ex.query.as_str()), "leaked: {}", ex.query);
        }
        // Held-out sums never appear in training in any phrasing.
        let held_sums: HashSet<(&str, &str)> = ds.item_test.iter().filter(|e| e.template == 1).map(|e| (e.query.as_str(), e.answer.as_str())).collect();
        for ex in &ds.train {
            assert!(!held_sums.contains(&(ex.query.as_str(), ex.answer.as_str())));
        }
        assert!(!ds.item_test.is_empty() && ds.item_test.iter().all(|e| matches!(e.family, Family::Add | Family::Sub)));
    }

    #[test]
    fn the_answers_are_right() {
        let ds = build(1);
        for ex in ds.train.iter().chain(&ds.item_test).chain(&ds.paraphrase_test) {
            let words = tokenize(&ex.query);
            let n = |w: &str| NUM.iter().position(|x| *x == w);
            match ex.family {
                Family::Add => {
                    let nums: Vec<usize> = words.iter().filter_map(|w| n(w)).collect();
                    assert_eq!(NUM[nums[0] + nums[1]], ex.answer, "{}", ex.query);
                }
                Family::Sub => {
                    let nums: Vec<usize> = words.iter().filter_map(|w| n(w)).collect();
                    assert_eq!(NUM[nums[0] - nums[1]], ex.answer, "{}", ex.query);
                }
                _ => {}
            }
        }
    }

    #[test]
    fn typos_differ_from_the_original_by_one_word_and_keep_the_answer() {
        let ds = build(1);
        assert!(ds.typo_test.len() > 300);
        for (t, original) in ds.typo_test.iter().zip(ds.train.iter().filter(|e| tokenize(&e.query).iter().any(|w| w.chars().count() >= 4))) {
            assert_eq!(t.answer, original.answer);
            let (a, b) = (tokenize(&t.query), tokenize(&original.query));
            assert_eq!(a.len(), b.len());
            assert_eq!(a.iter().zip(&b).filter(|(x, y)| x != y).count(), 1, "{} vs {}", t.query, original.query);
        }
    }

    #[test]
    fn deterministic_for_a_seed() {
        let a = build(5);
        let b = build(5);
        assert_eq!(a.train.len(), b.train.len());
        assert_eq!(a.train[10].query, b.train[10].query);
        assert_eq!(a.item_test.iter().map(|e| e.query.clone()).collect::<Vec<_>>(), b.item_test.iter().map(|e| e.query.clone()).collect::<Vec<_>>());
    }

    #[test]
    fn tokenizer_lowercases_and_drops_punctuation() {
        assert_eq!(tokenize("What is the Capital of France?"), vec!["what", "is", "the", "capital", "of", "france"]);
    }
}
