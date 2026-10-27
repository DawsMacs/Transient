// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DawsMacs
//! Ways of asking that mean the same, learned from lessons that differ only in how they ask.
//!
//! Two lessons with the same answer, one with the words of the other and more, that share a word only a few lessons have (the thing asked about), are the
//! same fact in two wordings. The extra words ("what", "how", "the", "of": the frame) change nothing about the answer, and every such pair is one more time that was
//! seen. A word seen to differ that way often enough is a frame word; what is left of a question when its frame words are taken away is the fact it
//! asks. A question that names a taught fact in a frame the fact was never taught in, with frame words that were seen to change nothing, is the same
//! fact in new words, and it is as sure as that evidence: the more pairs showed the frame word to change nothing, the surer.
//!
//! Nothing here knows which words are frame words in any language; a word that makes a difference to the answer (that is, appears only in some
//! facts of a kind) never passes: it is not in the difference of two wordings of one fact.

use std::collections::{BTreeMap, BTreeSet};

/// A word is a frame word once this many pairs of wordings of one fact differed in it.
pub const MIN_SUPPORT: u64 = 6;
/// A word in fewer lessons than this is the thing asked about (or one thing of a kind), not part of the frame or the kind.
const RARE: u32 = 10;

#[derive(Default)]
pub struct Frames {
    /// Frame words and how many pairs of wordings differed in them.
    frame: BTreeMap<String, u64>,
    /// What is left of a question without its frame words, to the answers given for it, the words of the wordings it was taught in, and the orders
    /// in which its content words were taught.
    facts: BTreeMap<String, Fact>,
    /// By the kind (the common words of the content): how many times two orders of the content words of one fact were taught. Where that is
    /// nothing, no order is known to be as good as another.
    orders: BTreeMap<String, u64>,
    df: BTreeMap<String, u32>,
}

#[derive(Default)]
struct Fact {
    answers: BTreeSet<String>,
    words: BTreeSet<String>,
    sequences: BTreeSet<Vec<String>>,
}

/// What a question in new words is worth: the answer, and how many pairs of wordings stand behind the weakest of its new frame words (the
/// number of frame words that were new is how many such words it has).
pub struct Match {
    pub answer: String,
    pub support: u64,
    pub new_words: usize,
    /// Set if the content words of the question come in an order the fact was never taught in: how many times two orders of a fact of this kind
    /// were taught. (A reordered question of a kind with none is not a match at all.)
    pub order_support: Option<u64>,
}

/// The words of a question that are not frame words, in the order they come.
fn content(words: &[String], frame: &BTreeMap<String, u64>) -> Vec<String> {
    words.iter().filter(|w| !frame.contains_key(*w)).cloned().collect()
}

fn key(sequence: &[String]) -> String {
    let mut sorted: Vec<&str> = sequence.iter().map(String::as_str).collect();
    sorted.sort_unstable();
    sorted.join(" ")
}

/// Learn from `lessons` (the words of the question, as a set in order of appearance, and the answer).
pub fn build(lessons: &[(Vec<String>, String)]) -> Frames {
    let mut df: BTreeMap<&str, u32> = BTreeMap::new();
    for (words, _) in lessons {
        for w in words {
            *df.entry(w).or_insert(0) += 1;
        }
    }
    let mut by_answer: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
    for (i, (_, answer)) in lessons.iter().enumerate() {
        by_answer.entry(answer).or_default().push(i);
    }
    let mut support: BTreeMap<String, u64> = BTreeMap::new();
    for ids in by_answer.values() {
        for (n, &i) in ids.iter().enumerate() {
            for &j in &ids[n + 1..] {
                let (a, b): (BTreeSet<&String>, BTreeSet<&String>) = (lessons[i].0.iter().collect(), lessons[j].0.iter().collect());
                // The same fact: they share a word that only a few lessons have (the thing asked about).
                if !a.intersection(&b).any(|w| df[w.as_str()] < RARE) {
                    continue;
                }
                // Wordings of one fact: one has the words of the other and more. (Two lessons whose words are each missing some of the other's
                // are two facts that happen to have an answer in common, a thing and two of its kinds, and tell nothing about the frame.)
                let (small, large) = if a.len() <= b.len() { (&a, &b) } else { (&b, &a) };
                if !small.is_subset(large) {
                    continue;
                }
                for w in large.difference(small) {
                    *support.entry((*w).clone()).or_insert(0) += 1;
                }
            }
        }
    }
    let frame: BTreeMap<String, u64> = support.into_iter().filter(|(_, n)| *n >= MIN_SUPPORT).collect();
    let mut facts: BTreeMap<String, Fact> = BTreeMap::new();
    for (words, answer) in lessons {
        let sequence = content(words, &frame);
        let entry = facts.entry(key(&sequence)).or_default();
        entry.answers.insert(answer.clone());
        entry.words.extend(words.iter().cloned());
        entry.sequences.insert(sequence);
    }
    let mut orders: BTreeMap<String, u64> = BTreeMap::new();
    for (k, fact) in &facts {
        if fact.answers.len() == 1 && fact.sequences.len() > 1 {
            *orders.entry(kind_of(k, &df)).or_insert(0) += fact.sequences.len() as u64 - 1;
        }
    }
    Frames { frame, facts, orders, df: df.into_iter().map(|(w, n)| (w.to_string(), n)).collect() }
}

/// The common words of a fact's content (the kind, not the thing asked about), as one string.
fn kind_of<S: std::borrow::Borrow<str> + Ord>(content_key: &str, df: &BTreeMap<S, u32>) -> String {
    content_key.split(' ').filter(|w| df.get(*w).copied().unwrap_or(0) >= RARE).collect::<Vec<_>>().join(" ")
}

impl Frames {
    /// The fact a question asks, if its content is a taught fact's, it was never taught in all these words, and the frame words that are new to
    /// it are known frame words; none if the content is not a taught fact, or two answers were ever given for it.
    pub fn lookup(&self, words: &[String]) -> Option<Match> {
        let sequence = content(words, &self.frame);
        if sequence.is_empty() {
            return None;
        }
        let fact = self.facts.get(&key(&sequence))?;
        if fact.answers.len() != 1 {
            return None;
        }
        let order_support = if fact.sequences.contains(&sequence) {
            None
        } else {
            // A new order of the content words: only if orders of other facts of this kind were taught and agreed.
            let kind = kind_of(&key(&sequence), &self.df);
            match self.orders.get(&kind) {
                Some(&n) if n > 0 => Some(n),
                _ => return None,
            }
        };
        // Every word of the question is either part of the content or a frame word: a word that is neither would have changed the content.
        let new: Vec<u64> = words.iter().filter(|w| !fact.words.contains(*w)).map(|w| self.frame.get(w).copied().unwrap_or(0)).collect();
        Some(Match { answer: fact.answers.iter().next().cloned().unwrap_or_default(), support: new.iter().copied().min().unwrap_or(u64::MAX), new_words: new.len(), order_support })
    }

    #[cfg(test)]
    pub fn frame_words(&self) -> Vec<&str> {
        self.frame.keys().map(String::as_str).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lesson(q: &str, a: &str) -> (Vec<String>, String) {
        let mut words: Vec<String> = Vec::new();
        for w in q.split_whitespace() {
            if !words.iter().any(|x| x == w) {
                words.push(w.to_string());
            }
        }
        (words, a.to_string())
    }

    /// Facts of two kinds, each in three frames, and a kind in a fourth frame.
    fn lessons() -> Vec<(Vec<String>, String)> {
        let mut out = Vec::new();
        for (i, e) in ["bafoba", "tadaba", "podoba", "mifuba", "ligaba", "nufeba", "pefoba", "dageba", "kuvaba", "retiba", "gobaba", "zinuba"].iter().enumerate() {
            let (a, b) = (format!("ta{i}pa"), format!("tu{i}pa"));
            out.push(lesson(&format!("how is {e} sedavubi"), &a));
            out.push(lesson(&format!("{e} is sedavubi how"), &a));
            out.push(lesson(&format!("sedavubi {e}"), &a));
            out.push(lesson(&format!("what is the madovubi of {e}"), &b));
            out.push(lesson(&format!("madovubi of {e}"), &b));
            out.push(lesson(&format!("the madovubi of {e}"), &b));
            if i < 8 {
                // A third kind, taught in a frame no other kind has ("name ... of").
                out.push(lesson(&format!("name the sedovubi of {e}"), &format!("to{i}pa")));
                out.push(lesson(&format!("the sedovubi of {e}"), &format!("to{i}pa")));
                out.push(lesson(&format!("sedovubi {e}"), &format!("to{i}pa")));
            }
        }
        out
    }

    #[test]
    fn words_in_which_wordings_of_a_fact_differ_are_frame_words_and_the_things_asked_about_are_not() {
        let f = build(&lessons());
        let frame = f.frame_words();
        for w in ["how", "is", "what", "the", "of", "name"] {
            assert!(frame.contains(&w), "{w} in {frame:?}");
        }
        for w in ["sedavubi", "madovubi", "sedovubi", "bafoba", "podoba"] {
            assert!(!frame.contains(&w), "{w} is not part of the frame");
        }
    }

    #[test]
    fn a_fact_asked_in_a_frame_it_was_not_taught_in_is_found_with_the_support_of_the_frame_words() {
        let f = build(&lessons());
        let words = |q: &str| lesson(q, "").0;
        // "name ... of" was taught for another kind only; the fact was taught in other frames.
        let m = f.lookup(&words("name the madovubi of podoba")).expect("the same fact in new words");
        assert_eq!(m.answer, "tu2pa");
        assert!(m.support >= MIN_SUPPORT && m.new_words == 1, "{}", m.new_words);
        // A different order and a different frame word.
        assert_eq!(f.lookup(&words("is mifuba sedavubi how")).map(|m| m.answer), Some("ta3pa".to_string()));
        assert_eq!(f.lookup(&words("what is bafoba sedavubi")).map(|m| m.answer), Some("ta0pa".to_string()));
    }

    #[test]
    fn what_is_not_a_taught_fact_is_not_found() {
        let f = build(&lessons());
        let words = |q: &str| lesson(q, "").0;
        // Two kinds at once, a thing nobody was taught about, a kind the thing was not taught for.
        assert!(f.lookup(&words("what is the sedavubi of podoba madovubi")).is_none());
        assert!(f.lookup(&words("what is the madovubi of kufoba")).is_none());
        assert!(f.lookup(&words("what is the sedovubi of zinuba")).is_none(), "the third kind was taught for eight things only");
    }

    #[test]
    fn the_content_words_in_a_new_order_need_orders_that_were_taught_and_agreed_for_that_kind() {
        let mut all = lessons();
        // One more thing of the kind, taught only with the thing before the kind.
        all.push(lesson("how is xobaba sedavubi", "taxpa"));
        all.push(lesson("xobaba is sedavubi how", "taxpa"));
        let f = build(&all);
        let words = |q: &str| lesson(q, "").0;
        // "sedavubi" facts were taught with the thing before the kind and the kind before the thing, twelve times, and the two always agreed.
        let m = f.lookup(&words("sedavubi xobaba how is")).expect("another order of a kind whose orders agreed");
        assert_eq!(m.answer, "taxpa");
        assert!(m.order_support.is_some_and(|n| n >= 12), "{:?}", m.order_support);
        // A kind taught in one order only: a new order is no match.
        let mut one_order = Vec::new();
        for (i, (a, b)) in [("one", "two"), ("two", "six"), ("three", "four"), ("four", "five"), ("five", "one"), ("six", "three"), ("one", "five"), ("two", "four"), ("three", "six"), ("four", "six"), ("five", "two"), ("six", "four")].iter().enumerate() {
            for frame in ["what is {a} nidovubi {b}", "{a} nidovubi {b} equals"] {
                one_order.push(lesson(&frame.replace("{a}", a).replace("{b}", b), &format!("r{i}")));
            }
        }
        let g = build(&one_order);
        assert!(g.lookup(&words("what is two nidovubi one")).is_none());
        assert!(g.lookup(&words("what is one nidovubi two")).is_some());
    }

    #[test]
    fn a_word_seen_to_differ_in_too_few_pairs_is_not_a_frame_word() {
        let mut few = Vec::new();
        for (i, e) in ["bafoba", "tadaba"].iter().enumerate() {
            few.push(lesson(&format!("how is {e} sedavubi"), &format!("ta{i}pa")));
            few.push(lesson(&format!("{e} sedavubi"), &format!("ta{i}pa")));
        }
        let f = build(&few);
        assert!(f.frame_words().is_empty(), "{:?}", f.frame_words());
    }
}
