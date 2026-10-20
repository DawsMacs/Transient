// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DawsMacs
//! Does the baby know what it doesn't know?
//!
//! Three signals could tell it to hold back instead of guessing:
//!   confidence    how sure the network is of its best answer
//!   familiarity   how close the question is to questions it was taught (rare words weigh more than common ones)
//!   unmet word    the question contains a word it has never read
//!
//! Each is calibrated on the same footing: set the threshold so that 95% of ordinary reworded questions about known facts still
//! get answered, then see how many unanswerable questions it refuses. Two kinds of unanswerable question:
//!   unmet words        "capital of atlantis", "what color is the zebra": the entity is new
//!   known words, wrong "capital of banana", "color of france": every word is familiar, the question makes no sense
//!   context

use crate::data::{self, Dataset};
use crate::model::{build_answers, build_words, percentile_floor, Model};
use crate::net::{Adam, Config, Net, Reader, Scratch};
use crate::rng::Rng;
use crate::store::Store;

pub fn fit(ds: &Dataset, reader: Reader, seed: u64) -> Model {
    let answers = build_answers(&[&ds.train, &ds.paraphrase_test, &ds.item_test]);
    fit_with(&ds.train, answers, reader, seed)
}

/// Train a network on any list of questions, able to say any of the given answers.
pub fn fit_examples(train: &[crate::data::Example], reader: Reader, seed: u64) -> Model {
    let answers = build_answers(&[train]);
    fit_with(train, answers, reader, seed)
}

fn fit_with(train_set: &[crate::data::Example], answers: Vec<String>, reader: Reader, seed: u64) -> Model {
    let words = build_words(train_set);
    let cfg = Config { vocab: words.len(), dim: 16, len: data::LEN, hidden: 64, classes: answers.len(), reader, buckets: 512 };
    let mut rng = Rng::new(seed.wrapping_add(1000));
    let mut model = Model::new(Net::new(cfg, &mut rng), words, answers, seed);
    model.learn_familiarity(train_set);
    let enc: Vec<(Vec<usize>, usize)> =
        train_set.iter().map(|e| (model.encode(&e.query), model.answer_id(&e.answer).unwrap())).collect();
    let mut scratch = Scratch::new(&cfg);
    let mut adam = Adam::new(cfg.n_params(), 0.01, 0.0);
    let mut grad = vec![0.0f32; cfg.n_params()];
    for _ in 0..300 {
        let mut idx: Vec<usize> = (0..enc.len()).collect();
        rng.shuffle(&mut idx);
        for chunk in idx.chunks(16) {
            grad.iter_mut().for_each(|g| *g = 0.0);
            for &i in chunk {
                model.net.loss_and_grad(&enc[i].0, enc[i].1, &mut grad, &mut scratch);
            }
            let inv = 1.0 / chunk.len() as f32;
            grad.iter_mut().for_each(|g| *g *= inv);
            adam.step(&mut model.net.theta, &grad, &mut scratch.ledger);
        }
    }
    model
}

struct Signals {
    confidence: f32,
    familiarity: f32,
    unmet: bool,
    correct: Option<bool>,
}

struct Gauge {
    taught: Store,
}

impl Gauge {
    fn new(model: &Model) -> Gauge {
        Gauge { taught: model.familiarity_store() }
    }

    fn read(&self, model: &Model, scratch: &mut Scratch, query: &str, answer: Option<&str>) -> Signals {
        let ids = model.encode(query);
        let p = model.net.probs(&ids, scratch);
        let best = p.iter().enumerate().max_by(|a, b| a.1.partial_cmp(b.1).unwrap()).unwrap();
        let familiarity = model.familiarity(&self.taught, query);
        let unmet = data::tokenize(query).iter().any(|word| !model.knows(word));
        Signals {
            confidence: *best.1,
            familiarity,
            unmet,
            correct: answer.map(|a| model.answer_id(a) == Some(best.0)),
        }
    }
}

fn probes(model: &Model) -> (Vec<String>, Vec<String>) {
    let make = |templates: &[&str], items: &[&str]| -> Vec<String> {
        items.iter().flat_map(|i| templates.iter().map(move |t| t.replace("{0}", i))).collect()
    };
    let cap = ["capital of {0}", "what is the capital of {0}"];
    let col = ["what color is the {0}", "color of {0}"];
    let snd = ["what does the {0} say", "{0} sound"];
    let opp = ["opposite of {0}", "what is the opposite of {0}"];
    let mut unmet = Vec::new();
    unmet.extend(make(&cap, &["atlantis", "narnia", "gondor", "wakanda", "utopia", "mars", "venus", "zanzibar", "bolivia", "chile", "nepal", "ghana", "finland"]));
    unmet.extend(make(&col, &["zebra", "rainbow", "galaxy", "wizard", "robot", "emerald", "dragon"]));
    unmet.extend(make(&snd, &["dragon", "unicorn", "robot", "dinosaur", "goat", "wolf"]));
    unmet.extend(make(&opp, &["purple", "tall", "rich", "early", "brave", "tiny"]));
    let mut nonsense = Vec::new();
    nonsense.extend(make(&cap, &["banana", "sky", "dog", "hot", "three", "milk", "up", "lion", "yes", "five", "snow"]));
    nonsense.extend(make(&col, &["france", "canada", "hot", "up", "five", "yes", "dog", "lion"]));
    nonsense.extend(make(&snd, &["france", "hot", "sky", "three", "canada", "banana", "milk"]));
    nonsense.extend(make(&opp, &["france", "banana", "dog", "sky", "three", "snow", "canada"]));
    // Keep each probe in the class it claims: unmet words must really be unmet, nonsense words must really be familiar.
    let words_known = |q: &String| data::tokenize(q).iter().all(|w| model.knows(w));
    unmet.retain(|q| !words_known(q));
    nonsense.retain(|q| words_known(q));
    (unmet, nonsense)
}

#[derive(Default, Clone, Copy)]
struct Tally {
    coverage: f64,
    accuracy: f64,
    refuses_unmet: f64,
    refuses_nonsense: f64,
}

const POLICIES: [&str; 7] = [
    "always answer",
    "confidence",
    "familiarity",
    "unmet word",
    "unmet word + confidence",
    "unmet word + familiarity",
    "all three",
];

fn refuses(policy: usize, s: &Signals, t_conf: f32, t_fam: f32) -> bool {
    let low_conf = s.confidence < t_conf;
    let unfamiliar = s.familiarity < t_fam;
    match policy {
        0 => false,
        1 => low_conf,
        2 => unfamiliar,
        3 => s.unmet,
        4 => s.unmet || low_conf,
        5 => s.unmet || unfamiliar,
        _ => s.unmet || low_conf || unfamiliar,
    }
}

pub fn run(reader: Reader, seeds: u64, keep: f32) -> Vec<(&'static str, f64, f64, f64, f64)> {
    let mut totals = [Tally::default(); 7];
    let mut counts = (0usize, 0usize, 0usize);
    for seed in 1..=seeds {
        let ds = data::build(seed);
        let model = fit(&ds, reader, seed);
        let gauge = Gauge::new(&model);
        let mut s = Scratch::new(&model.net.cfg);
        let positives: Vec<Signals> =
            ds.paraphrase_test.iter().map(|e| gauge.read(&model, &mut s, &e.query, Some(&e.answer))).collect();
        let (unmet_q, nonsense_q) = probes(&model);
        let unmet: Vec<Signals> = unmet_q.iter().map(|q| gauge.read(&model, &mut s, q, None)).collect();
        let nonsense: Vec<Signals> = nonsense_q.iter().map(|q| gauge.read(&model, &mut s, q, None)).collect();
        counts = (positives.len(), unmet.len(), nonsense.len());
        let t_conf = percentile_floor(positives.iter().map(|x| x.confidence).collect(), keep);
        let t_fam = percentile_floor(positives.iter().map(|x| x.familiarity).collect(), keep);
        for (pi, t) in totals.iter_mut().enumerate() {
            let answered: Vec<&Signals> = positives.iter().filter(|x| !refuses(pi, x, t_conf, t_fam)).collect();
            t.coverage += answered.len() as f64 / positives.len() as f64;
            t.accuracy += answered.iter().filter(|x| x.correct == Some(true)).count() as f64 / answered.len().max(1) as f64;
            let frac = |set: &[Signals]| set.iter().filter(|x| refuses(pi, x, t_conf, t_fam)).count() as f64 / set.len().max(1) as f64;
            t.refuses_unmet += frac(&unmet);
            t.refuses_nonsense += frac(&nonsense);
        }
    }
    let _ = counts;
    let n = seeds as f64;
    POLICIES
        .iter()
        .zip(totals)
        .map(|(name, t)| (*name, 100.0 * t.coverage / n, 100.0 * t.accuracy / n, 100.0 * t.refuses_unmet / n, 100.0 * t.refuses_nonsense / n))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn probes_land_in_the_class_they_claim() {
        let ds = data::build(1);
        let model = fit_quick(&ds);
        let (unmet, nonsense) = probes(&model);
        assert!(unmet.len() > 40 && nonsense.len() > 40);
        assert!(unmet.iter().all(|q| data::tokenize(q).iter().any(|w| !model.knows(w))));
        assert!(nonsense.iter().all(|q| data::tokenize(q).iter().all(|w| model.knows(w))));
    }

    fn fit_quick(ds: &Dataset) -> Model {
        let words = build_words(&ds.train);
        let answers = build_answers(&[&ds.train, &ds.paraphrase_test, &ds.item_test]);
        let cfg = Config { vocab: words.len(), dim: 4, len: data::LEN, hidden: 4, classes: answers.len(), reader: Reader::Bag, buckets: 0 };
        let mut model = Model::new(Net::new(cfg, &mut Rng::new(1)), words, answers, 1);
        model.learn_familiarity(&ds.train);
        model
    }

    #[test]
    fn common_words_weigh_less_than_telling_ones() {
        let ds = data::build(1);
        let m = fit_quick(&ds);
        assert!(m.word_weight("the") < m.word_weight("peru"));
        assert!(m.word_weight("peru") < m.word_weight("atlantis"), "an unmet word counts the most");
    }
}
