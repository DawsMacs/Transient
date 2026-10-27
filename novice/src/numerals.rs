// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DawsMacs
//! Words that are numbers, found from the lessons: nothing here knows that "two" is a number, in any language.
//!
//! Some words are the answer of lesson after lesson, and the same words are what other questions are about ("five bidevubi zero" is "five"). If a kind made of such words
//! behaves like addition under *some* assignment of whole numbers to the words, that assignment is what the words mean. It is found the way the counting
//! is: one word is nothing (adding it changes nothing), another is the unit, and every lesson of the kind ("one and two make three") then fixes more
//! words from the ones already fixed, until every lesson of the kind holds or one fails. The assignment is kept only if it is the same for every
//! kind that adds. Everything else (subtracting, multiplying, the biggest of two) is then learned by the learner as it learns on written numbers.
//!
//! Limits, plainly: addition is the structure looked for (it is the one that fixes a unit; a kind that is only an order, like the larger of two,
//! does not), and a number is a word only if some lesson says it.

use std::collections::{BTreeMap, BTreeSet};

/// A word is a candidate number only if it is the whole answer of at least this many lessons (one fact, in its three wordings).
const MIN_ANSWERS: u32 = 3;
/// Fewest lessons of one kind to look for addition in it.
const MIN_LESSONS: usize = 8;
/// Fewest words a vocabulary of numbers must have.
const MIN_WORDS: usize = 4;

fn tokens(text: &str) -> Vec<String> {
    text.split_whitespace().map(|t| t.trim_matches(|c: char| matches!(c, '?' | ',' | '.' | '!' | ';' | ':')).to_lowercase()).filter(|t| !t.is_empty()).collect()
}

/// `(a, b, c)`: the two words in the question and the word of the answer.
type Triple = (String, String, String);

/// Fix the words of `triples` as whole numbers, starting from `known`, so that `a + b = c` in every triple; none if that cannot be done
/// (a lesson that fails, two words with one number, a number below zero).
fn propagate(triples: &[Triple], known: &BTreeMap<String, i64>) -> Option<BTreeMap<String, i64>> {
    let mut sigma = known.clone();
    let set = |sigma: &mut BTreeMap<String, i64>, w: &str, v: i64| -> Option<bool> {
        if v < 0 {
            return None;
        }
        match sigma.get(w) {
            Some(&old) => (old == v).then_some(false),
            None => {
                if sigma.values().any(|&x| x == v) {
                    return None;
                }
                sigma.insert(w.to_string(), v);
                Some(true)
            }
        }
    };
    loop {
        let mut changed = false;
        for (a, b, c) in triples {
            let (sa, sb, sc) = (sigma.get(a).copied(), sigma.get(b).copied(), sigma.get(c).copied());
            match (sa, sb, sc) {
                (Some(x), Some(y), _) => changed |= set(&mut sigma, c, x + y)?,
                (Some(x), None, Some(z)) => changed |= set(&mut sigma, b, z - x)?,
                (None, Some(y), Some(z)) => changed |= set(&mut sigma, a, z - y)?,
                _ => {}
            }
        }
        if !changed {
            return Some(sigma);
        }
    }
}

/// Largest number a word may be given when a lesson does not fix it.
const MAX_VALUE: i64 = 40;
/// Most assignments looked at when a lesson set leaves words free, and most steps of that search.
const MAX_SOLUTIONS: usize = 40;
const MAX_STEPS: usize = 200_000;

/// Every way (up to a few) to give all the words of `triples` a number so that every lesson adds, starting from `known`. Words that are free are tried
/// from the smallest number up; the search stops at a limit, and the caller keeps only what every solution found agrees on.
fn solutions(triples: &[Triple], known: &BTreeMap<String, i64>) -> Vec<BTreeMap<String, i64>> {
    let mut words: BTreeMap<&str, usize> = BTreeMap::new();
    for (a, b, c) in triples {
        for w in [a, b, c] {
            *words.entry(w).or_insert(0) += 1;
        }
    }
    let mut out = Vec::new();
    let mut steps = 0usize;
    fn go(triples: &[Triple], sigma: BTreeMap<String, i64>, words: &BTreeMap<&str, usize>, out: &mut Vec<BTreeMap<String, i64>>, steps: &mut usize) {
        if out.len() >= MAX_SOLUTIONS || *steps > MAX_STEPS {
            return;
        }
        *steps += 1;
        let Some(sigma) = propagate(triples, &sigma) else { return };
        // The free word that is in most lessons.
        let free = words.iter().filter(|(w, _)| !sigma.contains_key(**w)).max_by_key(|(w, n)| (**n, std::cmp::Reverse(**w)));
        let Some((word, _)) = free else {
            out.push(sigma);
            return;
        };
        for v in 0..=MAX_VALUE {
            if sigma.values().any(|&x| x == v) {
                continue;
            }
            let mut next = sigma.clone();
            next.insert((*word).to_string(), v);
            go(triples, next, words, out, steps);
        }
    }
    go(triples, known.clone(), &words, &mut out, &mut steps);
    out
}

/// The numbers every solution agrees on.
fn consensus(solved: &[BTreeMap<String, i64>]) -> Option<BTreeMap<String, i64>> {
    let first = solved.first()?;
    Some(first.iter().filter(|(w, v)| solved.iter().all(|s| s.get(*w) == Some(*v))).map(|(w, v)| (w.clone(), *v)).collect())
}

/// The meaning of each number word the lessons give: word to whole number. Empty if no kind of the lessons adds.
pub fn learn(lessons: &[(String, String)]) -> BTreeMap<String, i64> {
    let mut answers: BTreeMap<String, u32> = BTreeMap::new();
    let parsed: Vec<(Vec<String>, String)> = lessons.iter().map(|(q, a)| (tokens(q), a.trim().to_lowercase())).collect();
    for (_, a) in &parsed {
        if !a.is_empty() && !a.contains(char::is_whitespace) {
            *answers.entry(a.clone()).or_insert(0) += 1;
        }
    }
    // (Written numbers are numbers already.)
    let candidates: BTreeSet<String> = answers.iter().filter(|(w, &n)| n >= MIN_ANSWERS && !w.bytes().all(|b| b.is_ascii_digit())).map(|(w, _)| w.clone()).collect();
    if candidates.len() < MIN_WORDS {
        return BTreeMap::new();
    }
    // The kinds made of these words: two of them in the question, one as the answer. Which word of the question names the kind is not known (the frame
    // words are in every lesson, the kind's own word in all of its lessons), so every word that is not a number is tried as the name: a group made of a
    // frame word mixes kinds and fails to add up; the group of the kind's own word is the whole kind.
    let mut kinds: BTreeMap<String, Vec<Triple>> = BTreeMap::new();
    for (q, a) in &parsed {
        let numbers: Vec<&String> = q.iter().filter(|w| candidates.contains(*w)).collect();
        if numbers.len() != 2 || !candidates.contains(a) {
            continue;
        }
        let names: BTreeSet<&String> = q.iter().filter(|w| !candidates.contains(*w)).collect();
        for kind in names {
            kinds.entry(kind.clone()).or_default().push((numbers[0].clone(), numbers[1].clone(), a.clone()));
        }
    }
    // Start from the kind and the two words that fix the most words.
    let mut best: Option<BTreeMap<String, i64>> = None;
    let better = |new: &BTreeMap<String, i64>, old: &Option<BTreeMap<String, i64>>| match old {
        None => true,
        Some(o) => (new.len(), std::cmp::Reverse(new.values().max())) > (o.len(), std::cmp::Reverse(o.values().max())),
    };
    for triples in kinds.values().filter(|t| t.len() >= MIN_LESSONS) {
        for zero in &candidates {
            // Adding the word changes nothing, in every lesson that has it.
            let with_zero: Vec<&Triple> = triples.iter().filter(|(a, b, _)| a == zero || b == zero).collect();
            if with_zero.len() < 2 || !with_zero.iter().all(|(a, b, c)| (a == zero && c == b) || (b == zero && c == a)) {
                continue;
            }
            for one in candidates.iter().filter(|w| *w != zero) {
                let start: BTreeMap<String, i64> = [(zero.clone(), 0), (one.clone(), 1)].into_iter().collect();
                if let Some(sigma) = consensus(&solutions(triples, &start)) {
                    if better(&sigma, &best) {
                        best = Some(sigma);
                    }
                }
            }
        }
    }
    let Some(mut sigma) = best else { return BTreeMap::new() };
    // Every other kind that adds under this assignment says more words: the union of their lessons is fixed together.
    let adds = |triples: &[Triple], sigma: &BTreeMap<String, i64>| {
        let known: Vec<&Triple> = triples.iter().filter(|(a, b, c)| sigma.contains_key(a) && sigma.contains_key(b) && sigma.contains_key(c)).collect();
        let distinct: BTreeSet<&Triple> = known.iter().copied().collect();
        distinct.len() >= 3 && known.iter().all(|(a, b, c)| sigma[a] + sigma[b] == sigma[c])
    };
    let mut union: Vec<Triple> = Vec::new();
    for triples in kinds.values().filter(|t| t.len() >= MIN_LESSONS) {
        if adds(triples, &sigma) {
            union.extend(triples.iter().cloned());
        }
    }
    if let Some(more) = consensus(&solutions(&union, &sigma)) {
        sigma = more;
    }
    if sigma.len() < MIN_WORDS {
        return BTreeMap::new();
    }
    sigma
}

#[cfg(test)]
mod tests {
    use super::*;

    const WORDS: [&str; 11] = ["zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten"];

    /// Lessons of a kind `name` over `pairs`, each in the three wordings the exams use, answering with `rule`.
    fn lessons_of(name: &str, pairs: &[(usize, usize)], rule: impl Fn(usize, usize) -> Option<usize>) -> Vec<(String, String)> {
        let mut out = Vec::new();
        for &(a, b) in pairs {
            if let Some(c) = rule(a, b) {
                for template in ["what is {a} {k} {b}", "{a} {k} {b}", "{a} {k} {b} equals"] {
                    out.push((template.replace("{a}", WORDS[a]).replace("{k}", name).replace("{b}", WORDS[b]), WORDS[c].to_string()));
                }
            }
        }
        out
    }

    fn pairs_up_to(n: usize) -> Vec<(usize, usize)> {
        (0..=n).flat_map(|a| (0..=n).map(move |b| (a, b))).filter(|&(a, b)| (a, b) == (0, 0) || (a * 3 + b * 5) % 4 != 0).collect()
    }

    #[test]
    fn the_words_of_a_kind_that_adds_are_fixed_as_numbers() {
        let mut lessons = lessons_of("bidevubi", &pairs_up_to(5), |a, b| Some(a + b));
        // Kinds about things: an invented answer that repeats, and words that are only questions.
        for (i, w) in ["bafoba", "tadaba", "podoba", "mifuba", "ligaba", "nufeba", "pefoba"].iter().enumerate() {
            for template in ["how is {w} sedavubi", "{w} is sedavubi how", "sedavubi {w}"] {
                lessons.push((template.replace("{w}", w), ["vetupa", "bevapa", "zovapa", "vetupa", "bevapa", "zovapa", "vetupa"][i].to_string()));
            }
        }
        let sigma = learn(&lessons);
        for (i, w) in WORDS.iter().enumerate().take(11) {
            if let Some(&v) = sigma.get(*w) {
                assert_eq!(v, i as i64, "{w}");
            }
        }
        assert!(sigma.len() >= 8, "zero to at least seven: {sigma:?}");
        assert_eq!(sigma["zero"], 0);
        assert_eq!(sigma["six"], 6, "six is only ever an answer, and still fixed");
        assert!(!sigma.contains_key("vetupa") && !sigma.contains_key("sedavubi"), "{sigma:?}");
    }

    #[test]
    fn a_second_kind_that_adds_with_other_pairs_fixes_words_the_first_leaves_out() {
        let mut lessons = lessons_of("nifavubi", &pairs_up_to(3), |a, b| Some(a + b));
        lessons.extend(lessons_of("pudivubi", &[(5, 4), (4, 4), (5, 3), (4, 3), (5, 5), (1, 0), (0, 2), (0, 0), (0, 4)], |a, b| Some(a + b)));
        let sigma = learn(&lessons);
        assert_eq!(sigma.get("ten"), Some(&10));
        assert_eq!(sigma.get("nine"), Some(&9));
    }

    #[test]
    fn kinds_that_are_not_addition_fix_nothing() {
        // Only the larger of two and the absolute difference: an order, and a distance, but no unit and no zero that changes nothing.
        let mut lessons = lessons_of("dudovubi", &pairs_up_to(5), |a, b| Some(a.max(b)));
        lessons.extend(lessons_of("dufavubi", &pairs_up_to(5), |a, b| Some(a.abs_diff(b))));
        assert!(learn(&lessons).is_empty());
    }

    #[test]
    fn numbers_written_with_digits_are_not_words_to_learn() {
        let mut lessons = Vec::new();
        for a in 0..8usize {
            for b in 0..8usize {
                for template in ["what is {a} fegisido {b}", "{a} fegisido {b}", "{a} fegisido {b} equals"] {
                    lessons.push((template.replace("{a}", &a.to_string()).replace("{b}", &b.to_string()), (a + b).to_string()));
                }
            }
        }
        assert!(learn(&lessons).is_empty());
    }

    #[test]
    fn no_numbers_without_lessons_that_say_so() {
        assert!(learn(&[]).is_empty());
        let few = lessons_of("bidevubi", &[(1, 2), (2, 2)], |a, b| Some(a + b));
        assert!(learn(&few).is_empty());
    }
}

