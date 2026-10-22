// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DawsMacs
//! The fastest way to learn: write it down. A fact store holds (query, answer) pairs and finds the closest stored query by
//! comparing *sets of words*, weighted by how rare each word is in what the baby has heard (common words like "what", "is",
//! "the" count for little; "peru" counts for a lot). No gradients, no weight changes, nothing forgotten.
//!
//! Because it reads sets, it is position-free: "the capital of peru" finds "capital of peru". Because it reads sets, it is also
//! blind to order, which is its weakness ("three minus two" looks a lot like "three plus two").

use crate::ledger::Ledger;
use crate::model::PAD;

#[derive(Clone, Debug)]
pub struct Entry {
    /// Sorted, de-duplicated word ids (no padding).
    pub key: Vec<usize>,
    /// The original padded word ids, kept so the entry can later be replayed into the network.
    pub ids: Vec<usize>,
    pub target: usize,
}

pub struct Store {
    pub entries: Vec<Entry>,
    pub threshold: f32,
}

fn key_of(ids: &[usize]) -> Vec<usize> {
    let mut k: Vec<usize> = ids.iter().copied().filter(|&i| i != PAD).collect();
    k.sort_unstable();
    k.dedup();
    k
}

/// Weighted Jaccard of two sorted id lists. Returns (similarity, merge steps taken).
fn similarity(a: &[usize], b: &[usize], w: &dyn Fn(usize) -> f32) -> (f32, u64) {
    let (mut i, mut j) = (0, 0);
    let (mut inter, mut union) = (0.0f32, 0.0f32);
    let mut steps = 0u64;
    while i < a.len() || j < b.len() {
        steps += 1;
        if j >= b.len() || (i < a.len() && a[i] < b[j]) {
            union += w(a[i]);
            i += 1;
        } else if i >= a.len() || b[j] < a[i] {
            union += w(b[j]);
            j += 1;
        } else {
            let x = w(a[i]);
            inter += x;
            union += x;
            i += 1;
            j += 1;
        }
    }
    (if union > 0.0 { inter / union } else { 0.0 }, steps)
}

impl Store {
    pub fn new(threshold: f32) -> Store {
        Store { entries: Vec::new(), threshold }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn bytes(&self) -> usize {
        self.entries.iter().map(|e| (e.key.len() + e.ids.len() + 1) * 4).sum()
    }

    /// Write a fact. `key_src` is what the question is *compared* by (word ids); `ids` is what the network *reads* (the same,
    /// unless the network reads spelling), kept so the entry can later be replayed into the weights. An identical question
    /// replaces its old answer.
    pub fn insert(&mut self, key_src: &[usize], ids: &[usize], target: usize) {
        let key = key_of(key_src);
        if let Some(e) = self.entries.iter_mut().find(|e| e.key == key) {
            e.target = target;
            e.ids = ids.to_vec();
        } else {
            self.entries.push(Entry { key, ids: ids.to_vec(), target });
        }
    }

    /// The best stored answer whose similarity reaches the threshold.
    pub fn lookup(&self, ids: &[usize], w: &dyn Fn(usize) -> f32, ledger: &mut Ledger) -> Option<(usize, f32)> {
        let q = key_of(ids);
        ledger.touch(self.bytes());
        let mut best: Option<(usize, f32)> = None;
        for e in &self.entries {
            let (sim, steps) = similarity(&q, &e.key, w);
            ledger.int_ops += steps * 2;
            ledger.other_ops += steps; // the weight adds
            ledger.words_read += steps * 2 + 1;
            if sim >= self.threshold && best.map_or(true, |(_, s)| sim > s) {
                best = Some((e.target, sim));
            }
        }
        best
    }

    pub fn retain(&mut self, mut keep: impl FnMut(&Entry) -> bool) {
        self.entries.retain(|e| keep(e));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat(_: usize) -> f32 {
        1.0
    }

    #[test]
    fn exact_query_is_found_and_a_stranger_is_not() {
        let mut s = Store::new(0.7);
        s.insert(&[5, 6, 7, 0, 0], &[5, 6, 7, 0, 0], 3);
        let mut l = Ledger::default();
        assert_eq!(s.lookup(&[7, 5, 6, 0, 0], &flat, &mut l).map(|x| x.0), Some(3)); // order-free
        assert!(s.lookup(&[9, 10, 11, 0, 0], &flat, &mut l).is_none());
        assert!(l.int_ops > 0);
    }

    #[test]
    fn rare_words_decide_not_common_ones() {
        // ids: 2 = "capital" (common), 3 = "of" (common), 10 = peru, 11 = cuba (rare).
        let w = |id: usize| if id >= 10 { 5.0 } else { 0.2 };
        let mut s = Store::new(0.7);
        s.insert(&[2, 3, 10], &[2, 3, 10], 1);
        let mut l = Ledger::default();
        assert!(s.lookup(&[2, 3, 11], &w, &mut l).is_none(), "cuba must not retrieve peru");
        assert_eq!(s.lookup(&[4, 2, 3, 10], &w, &mut l).map(|x| x.0), Some(1), "an extra common word is tolerated");
    }

    #[test]
    fn same_query_overwrites() {
        let mut s = Store::new(0.7);
        s.insert(&[5, 6], &[5, 6], 1);
        s.insert(&[6, 5], &[6, 5], 2);
        assert_eq!(s.len(), 1);
        assert_eq!(s.entries[0].target, 2);
    }
}
