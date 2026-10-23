// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DawsMacs
//! A baby that *learns* instead of being trained.
//!
//! Training (`baby train`) shows a network the whole dataset many times. Learning is different: lessons arrive one at a time, the
//! baby tries to answer, is told the right word, and has to cope. The experiment here puts the same stream of lessons through
//! several ways of coping and charges each for the energy it spends:
//!
//!   naive    keep nudging the weights on the new lesson only
//!   replay   nudge the weights on the new lesson mixed with rehearsal of old lessons
//!   store    write the lesson into a fact store; never touch the weights
//!   sleep    store while awake; now and then "sleep": replay stored lessons into the weights, then forget what the weights now know
//!   retrain  the conventional way: freeze, and rebuild from scratch on everything seen so far at the end of each stage
//!
//! The baby already knows its *words* (a lexicon) but starts with no *facts*: what it learns are associations.

use crate::data::{Dataset, Example, STAGES};
use crate::ledger::{EnergyModel, Ledger};
use crate::model::{build_answers, build_words, Model};
use crate::net::{Adam, Config, Net, Reader, Scratch};
use crate::rng::Rng;
use crate::store::Store;
use std::time::Instant;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Strategy {
    Naive,
    Replay,
    Store,
    Sleep,
    Retrain,
}

impl Strategy {
    pub const ALL: [Strategy; 5] = [Strategy::Naive, Strategy::Replay, Strategy::Store, Strategy::Sleep, Strategy::Retrain];

    pub fn name(self) -> &'static str {
        match self {
            Strategy::Naive => "naive",
            Strategy::Replay => "replay",
            Strategy::Store => "store",
            Strategy::Sleep => "sleep",
            Strategy::Retrain => "retrain",
        }
    }

    pub fn parse(s: &str) -> Option<Strategy> {
        Strategy::ALL.into_iter().find(|x| x.name() == s)
    }

    fn uses_store(self) -> bool {
        matches!(self, Strategy::Store | Strategy::Sleep)
    }
}

#[derive(Clone, Debug)]
pub struct Settings {
    pub dim: usize,
    pub hidden: usize,
    pub lr: f32,
    /// Most weight updates spent on one lesson.
    pub max_steps: usize,
    /// Old lessons mixed into each update (replay, sleep).
    pub replay_k: usize,
    pub buffer_cap: usize,
    pub sleep_every: usize,
    pub sleep_epochs: usize,
    pub retrain_epochs: usize,
    pub store_threshold: f32,
    /// A right answer below this confidence still counts as a lesson to learn from.
    pub unsure_below: f32,
    pub days: usize,
    /// Plain gradient descent instead of Adam.
    pub sgd: bool,
    /// How the network reads a query (see `net.rs`).
    pub reader: Reader,
    pub buckets: usize,
    /// Letters reader only: let the fact store compare spelling pieces instead of whole words.
    pub piece_keys: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            dim: 16,
            hidden: 64,
            lr: 0.01,
            max_steps: 10,
            replay_k: 8,
            buffer_cap: 128,
            sleep_every: 60,
            sleep_epochs: 20,
            retrain_epochs: 100,
            store_threshold: 0.75,
            unsure_below: 0.6,
            days: 2,
            sgd: false,
            reader: Reader::Concat,
            buckets: 512,
            piece_keys: false,
        }
    }
}

#[derive(Clone)]
pub struct Item {
    /// What the network reads.
    pub ids: Vec<usize>,
    /// What the store compares: word ids.
    pub key: Vec<usize>,
    pub target: usize,
    pub stage: usize,
}

pub struct Learner {
    strategy: Strategy,
    st: Settings,
    cfg: Config,
    net: Net,
    adam: Adam,
    grad: Vec<f32>,
    scratch: Scratch,
    rng: Rng,
    store: Store,
    buffer: Vec<Item>,
    buffer_seen: u64,
    all_lessons: Vec<Item>,
    /// How many heard queries contain each word, for rare-word weighting in the store.
    df: Vec<u32>,
    heard: u32,
    pub updates: u64,
    pub lessons_learned: u64,
    pub sleeps: u64,
    pub store_peak: usize,
}

fn argmax(p: &[f32]) -> usize {
    p.iter().enumerate().max_by(|a, b| a.1.partial_cmp(b.1).unwrap()).map(|(i, _)| i).unwrap_or(0)
}

impl Learner {
    pub fn new(strategy: Strategy, st: Settings, cfg: Config, seed: u64) -> Learner {
        let mut rng = Rng::new(seed.wrapping_add(7000));
        let net = Net::new(cfg, &mut rng);
        Learner {
            strategy,
            adam: if st.sgd { Adam::plain_sgd(cfg.n_params(), st.lr) } else { Adam::new(cfg.n_params(), st.lr, 0.0) },
            grad: vec![0.0; cfg.n_params()],
            scratch: Scratch::new(&cfg),
            store: Store::new(st.store_threshold),
            df: vec![0; cfg.vocab.max(cfg.buckets + 1)],
            st,
            cfg,
            net,
            rng,
            buffer: Vec::new(),
            buffer_seen: 0,
            all_lessons: Vec::new(),
            heard: 0,
            updates: 0,
            lessons_learned: 0,
            sleeps: 0,
            store_peak: 0,
        }
    }

    pub fn ledger(&self) -> Ledger {
        self.scratch.ledger
    }

    pub fn store_len(&self) -> usize {
        self.store.len()
    }

    fn weight(df: &[u32], heard: u32, id: usize) -> f32 {
        (((1 + heard) as f32) / ((1 + df[id]) as f32)).ln().max(0.0) + 0.05
    }

    /// The baby's answer to a query: (word, confidence). The store answers first when it recognizes the question.
    pub fn answer(&mut self, ids: &[usize], key: &[usize]) -> (usize, f32) {
        if self.strategy.uses_store() {
            let (df, heard) = (&self.df, self.heard);
            let w = |id: usize| Learner::weight(df, heard, id);
            if let Some((a, sim)) = self.store.lookup(key, &w, &mut self.scratch.ledger) {
                return (a, sim);
            }
        }
        let p = self.net.probs(ids, &mut self.scratch);
        let a = argmax(&p);
        (a, p[a])
    }

    fn hear(&mut self, ids: &[usize]) {
        let mut seen = ids.to_vec();
        seen.sort_unstable();
        seen.dedup();
        for id in seen {
            if id != crate::model::PAD {
                self.df[id] += 1;
            }
        }
        self.heard += 1;
    }

    fn train_batch(&mut self, batch: &[&Item]) {
        self.grad.iter_mut().for_each(|g| *g = 0.0);
        for it in batch {
            self.net.loss_and_grad(&it.ids, it.target, &mut self.grad, &mut self.scratch);
        }
        let inv = 1.0 / batch.len() as f32;
        self.grad.iter_mut().for_each(|g| *g *= inv);
        self.adam.step(&mut self.net.theta, &self.grad, &mut self.scratch.ledger);
        self.updates += 1;
    }

    fn remember(&mut self, item: &Item) {
        // Reservoir sampling keeps a fair sample of everything learned within a fixed memory.
        self.buffer_seen += 1;
        if self.buffer.len() < self.st.buffer_cap {
            self.buffer.push(item.clone());
        } else {
            let j = (self.rng.next_u64() % self.buffer_seen) as usize;
            if j < self.buffer.len() {
                self.buffer[j] = item.clone();
            }
        }
    }

    fn learn_in_weights(&mut self, item: &Item, replay_k: usize) {
        for _ in 0..self.st.max_steps {
            let mut mix: Vec<Item> = vec![item.clone()];
            for _ in 0..replay_k.min(self.buffer.len()) {
                let j = self.rng.below(self.buffer.len());
                mix.push(self.buffer[j].clone());
            }
            let refs: Vec<&Item> = mix.iter().collect();
            self.train_batch(&refs);
            let p = self.net.probs(&item.ids, &mut self.scratch);
            if p[item.target] >= 0.9 {
                break;
            }
        }
    }

    /// One lesson: try, get told, cope.
    pub fn lesson(&mut self, item: &Item) {
        let (pred, conf) = self.answer(&item.ids, &item.key);
        self.hear(&item.key);
        let right = pred == item.target;
        if right && conf >= self.st.unsure_below {
            return;
        }
        self.lessons_learned += 1;
        match self.strategy {
            Strategy::Naive => {
                self.learn_in_weights(item, 0);
                self.remember(item);
            }
            Strategy::Replay => {
                self.learn_in_weights(item, self.st.replay_k);
                self.remember(item);
            }
            Strategy::Store | Strategy::Sleep => {
                self.store.insert(&item.key, &item.ids, item.target);
                self.store_peak = self.store_peak.max(self.store.len());
                if self.strategy == Strategy::Sleep && self.lessons_learned % self.st.sleep_every as u64 == 0 {
                    self.sleep();
                }
            }
            Strategy::Retrain => self.all_lessons.push(item.clone()),
        }
    }

    /// Replay what is in the store (mixed with older consolidated lessons) into the weights, then drop whatever the weights now know.
    pub fn sleep(&mut self) {
        self.sleeps += 1;
        let pending: Vec<Item> =
            self.store.entries.iter().map(|e| Item { ids: e.ids.clone(), key: e.key.clone(), target: e.target, stage: 0 }).collect();
        let mut pool: Vec<Item> = pending.clone();
        pool.extend(self.buffer.iter().cloned());
        for _ in 0..self.st.sleep_epochs {
            self.rng.shuffle(&mut pool);
            for chunk in pool.chunks(16) {
                let refs: Vec<&Item> = chunk.iter().collect();
                self.train_batch(&refs);
            }
        }
        let mut learned_now = Vec::new();
        for it in &pending {
            let p = self.net.probs(&it.ids, &mut self.scratch);
            if argmax(&p) == it.target && p[it.target] >= 0.8 {
                learned_now.push(it.clone());
            }
        }
        let known: std::collections::HashSet<Vec<usize>> = learned_now.iter().map(|i| i.ids.clone()).collect();
        self.store.retain(|e| !known.contains(&e.ids));
        for it in &learned_now {
            self.remember(it);
        }
    }

    /// The conventional alternative: rebuild from scratch on every lesson so far.
    pub fn retrain_from_scratch(&mut self) {
        self.net = Net::new(self.cfg, &mut self.rng);
        self.adam = if self.st.sgd { Adam::plain_sgd(self.cfg.n_params(), self.st.lr) } else { Adam::new(self.cfg.n_params(), self.st.lr, 0.0) };
        let mut pool = self.all_lessons.clone();
        for _ in 0..self.st.retrain_epochs {
            self.rng.shuffle(&mut pool);
            for chunk in pool.chunks(16) {
                let refs: Vec<&Item> = chunk.iter().collect();
                self.train_batch(&refs);
            }
        }
    }

    /// Accuracy on a set. The work is not charged to the baby's energy bill: it is the examiner's, not the learner's.
    pub fn evaluate(&mut self, items: &[Item]) -> (usize, usize) {
        let saved = self.scratch.ledger;
        let mut correct = 0;
        for it in items {
            correct += (self.answer(&it.ids, &it.key).0 == it.target) as usize;
        }
        self.scratch.ledger = saved;
        (correct, items.len())
    }
}

pub struct Checkpoint {
    pub day: usize,
    /// Highest stage taught so far.
    pub stage: usize,
    pub retained: (usize, usize),
    pub new_phrase: (usize, usize),
    pub joules: f64,
    pub updates: u64,
}

pub struct Outcome {
    pub strategy: Strategy,
    pub checkpoints: Vec<Checkpoint>,
    pub new_sums: (usize, usize),
    pub typos: (usize, usize),
    pub ledger: Ledger,
    pub joules: f64,
    pub arithmetic_share: f64,
    pub updates: u64,
    pub lessons_learned: u64,
    pub lessons_heard: usize,
    pub store_end: usize,
    pub store_peak: usize,
    pub sleeps: u64,
    pub millis: f64,
}

/// Run one baby through the whole curriculum with one strategy.
pub fn run(ds: &Dataset, strategy: Strategy, st: &Settings, seed: u64, energy: &EnergyModel) -> Outcome {
    // The lexicon covers every word the world uses; what has to be learned is which answer goes with which question.
    let mut everything: Vec<Example> = ds.train.clone();
    everything.extend(ds.paraphrase_test.iter().cloned());
    everything.extend(ds.item_test.iter().cloned());
    let words = build_words(&everything);
    let answers = build_answers(&[&everything]);
    let cfg = Config {
        vocab: words.len(),
        dim: st.dim,
        len: crate::data::LEN,
        hidden: st.hidden,
        classes: answers.len(),
        reader: st.reader,
        buckets: st.buckets,
    };
    let mut learner = Learner::new(strategy, st.clone(), cfg, seed);
    let lexicon = Model::new(Net::new(cfg, &mut Rng::new(0)), words, answers, seed);

    let enc = |set: &[Example]| -> Vec<Item> {
        set.iter()
            .map(|e| Item {
                ids: lexicon.encode(&e.query),
                key: if st.piece_keys { lexicon.key_ids(&e.query) } else { lexicon.word_ids(&e.query) },
                target: lexicon.answer_id(&e.answer).unwrap(),
                stage: e.family.stage(),
            })
            .collect()
    };
    let (train, para, sums, typos) = (enc(&ds.train), enc(&ds.paraphrase_test), enc(&ds.item_test), enc(&ds.typo_test));

    // The same lesson order for every strategy: stages in sequence, shuffled within a stage, a fresh shuffle each day.
    let mut order_rng = Rng::new(seed.wrapping_add(55));
    let days: Vec<Vec<Vec<usize>>> = (0..st.days)
        .map(|_| {
            (1..=STAGES)
                .map(|s| {
                    let mut idx: Vec<usize> = (0..train.len()).filter(|&i| train[i].stage == s).collect();
                    order_rng.shuffle(&mut idx);
                    idx
                })
                .collect()
        })
        .collect();

    let started = Instant::now();
    let mut checkpoints = Vec::new();
    let mut heard = 0;
    for (day, stages) in days.iter().enumerate() {
        // From day 2 on there is nothing new to retrain on, so the conventional baseline just stands still.
        for (si, order) in stages.iter().enumerate() {
            let stage = si + 1;
            for &i in order {
                learner.lesson(&train[i]);
                heard += 1;
            }
            if strategy == Strategy::Retrain && day == 0 {
                learner.retrain_from_scratch();
            }
            let last_stage = stage == STAGES;
            if day == 0 || last_stage {
                let upto = if day == 0 { stage } else { STAGES };
                let seen: Vec<Item> = train.iter().filter(|t| t.stage <= upto).cloned().collect();
                let seen_para: Vec<Item> = para.iter().filter(|t| t.stage <= upto).cloned().collect();
                let retained = learner.evaluate(&seen);
                let new_phrase = learner.evaluate(&seen_para);
                checkpoints.push(Checkpoint {
                    day: day + 1,
                    stage: upto,
                    retained,
                    new_phrase,
                    joules: energy.joules(&learner.ledger()),
                    updates: learner.updates,
                });
            }
        }
    }
    let new_sums = learner.evaluate(&sums);
    let typo_acc = learner.evaluate(&typos);
    let ledger = learner.ledger();
    let (a, m) = energy.split_pj(&ledger);
    Outcome {
        strategy,
        checkpoints,
        new_sums,
        typos: typo_acc,
        ledger,
        joules: energy.joules(&ledger),
        arithmetic_share: if a + m > 0.0 { a / (a + m) } else { 0.0 },
        updates: learner.updates,
        lessons_learned: learner.lessons_learned,
        lessons_heard: heard,
        store_end: learner.store_len(),
        store_peak: learner.store_peak,
        sleeps: learner.sleeps,
        millis: started.elapsed().as_secs_f64() * 1000.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data;

    fn quick() -> Settings {
        Settings { days: 1, retrain_epochs: 30, ..Settings::default() }
    }

    #[test]
    fn a_store_only_baby_never_updates_its_weights_and_keeps_everything() {
        let ds = data::build(1);
        let out = run(&ds, Strategy::Store, &quick(), 1, &EnergyModel::default());
        assert_eq!(out.updates, 0);
        let last = out.checkpoints.last().unwrap();
        assert!(last.retained.0 as f32 / last.retained.1 as f32 > 0.95, "store forgot: {:?}", last.retained);
    }

    #[test]
    fn learning_costs_energy_and_the_ledger_sees_it() {
        let ds = data::build(1);
        let out = run(&ds, Strategy::Replay, &quick(), 1, &EnergyModel::default());
        assert!(out.updates > 0 && out.joules > 0.0 && out.ledger.macs > 0);
    }

    #[test]
    fn same_seed_same_outcome() {
        let ds = data::build(1);
        let a = run(&ds, Strategy::Sleep, &quick(), 3, &EnergyModel::default());
        let b = run(&ds, Strategy::Sleep, &quick(), 3, &EnergyModel::default());
        assert_eq!(a.ledger, b.ledger);
        assert_eq!(a.checkpoints.last().unwrap().retained, b.checkpoints.last().unwrap().retained);
    }
}
