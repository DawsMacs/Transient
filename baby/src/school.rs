// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DawsMacs
//! One baby. It is taught a curriculum of *units* of lessons; for each unit it works out, from evidence, how to learn it (see
//! `mechanisms.rs`); a question is sent to the unit it resembles, answered with that unit's mechanism, and refused when the
//! baby has no business answering: a kind of question it was never taught, a unit where it could only ever memorise, a word it
//! has never met, or something its mechanism does not understand. Every answer, and every refusal, says where it came from.
//!
//! It keeps every lesson it was ever taught, so it can go on learning after school: a new lesson joins a unit (or opens one),
//! the unit's way of learning is worked out again from all its lessons, and the baby carries on.
//!
//! The work of answering is counted in a `Ledger`. Two choices keep it small: questions are sent to a unit by a precomputed table
//! of how telling each word is for each unit (a few lookups) instead of comparing the question with every lesson ever taught, and
//! remembered questions are found through an index of their content words instead of scanning them all.

use crate::data::tokenize;
use crate::kinds::{Item, World, UNITS};
use crate::ledger::Ledger;
use crate::mechanisms::{self, choose_mechanism, evidence_thrifty, fit, predict, shape, Fitted, Mech, Remembered, SelectRule, Weights};
use crate::model::Model;
use crate::net::Scratch;
use crate::program::{Expr, ProgramRule};
use crate::quantity::{counting_lessons, NumberLine, Quantity};
use std::collections::HashMap;
use std::fmt::Write;

pub struct SchoolUnit {
    pub name: String,
    pub chosen: Mech,
    pub evidence: [f64; 5],
    /// Everything it was ever taught in this unit.
    lessons: Vec<Item>,
    remember: Remembered,
    state: Fitted,
}

impl SchoolUnit {
    pub fn best_evidence(&self) -> f64 {
        self.evidence.iter().cloned().fold(0.0, f64::max)
    }

    /// True when nothing it tried generalised: the baby can only repeat what it was told in this unit.
    pub fn memorise_only(&self) -> bool {
        self.best_evidence() < 0.8
    }

    pub fn facts(&self) -> usize {
        self.lessons.iter().map(|l| l.fact).collect::<std::collections::BTreeSet<_>>().len()
    }

    pub fn lessons(&self) -> usize {
        self.lessons.len()
    }

    /// The expression it built, if its mechanism is a program.
    pub fn program(&self) -> Option<String> {
        match &self.state {
            Fitted::Program(Some(p)) => Some(p.expr.show()),
            _ => None,
        }
    }
}

/// How telling each word's *shape* is for each unit: ln(P(word | unit) / P(word)). A question's score for a unit is the sum over its
/// words, so sending it takes a lookup and a few additions per word.
struct Routing {
    table: HashMap<String, Vec<f32>>,
    units: usize,
}

impl Routing {
    fn build(units: &[SchoolUnit], line: &NumberLine) -> Routing {
        let n_units = units.len();
        let mut count: HashMap<String, Vec<f32>> = HashMap::new();
        let mut per_unit = vec![0.0f32; n_units];
        for (u, unit) in units.iter().enumerate() {
            for l in &unit.lessons {
                per_unit[u] += 1.0;
                for t in shape(&l.query, line) {
                    count.entry(t).or_insert_with(|| vec![0.0; n_units])[u] += 1.0;
                }
            }
        }
        let total: f32 = per_unit.iter().sum();
        let table = count
            .into_iter()
            .map(|(t, c)| {
                let all: f32 = c.iter().sum();
                let p_t = (all + 0.5) / (total + 1.0);
                let scores = (0..n_units).map(|u| (((c[u] + 0.5) / (per_unit[u] + 1.0)) / p_t).ln()).collect();
                (t, scores)
            })
            .collect();
        Routing { table, units: n_units }
    }

    fn scores(&self, shape: &[String], ledger: &mut Ledger) -> Vec<f32> {
        let mut s = vec![0.0f32; self.units];
        for t in shape {
            if let Some(row) = self.table.get(t) {
                for (u, x) in row.iter().enumerate() {
                    s[u] += x;
                }
            }
        }
        ledger.words_read += (shape.len() * (1 + self.units)) as u64;
        ledger.macs += (shape.len() * self.units) as u64;
        ledger.touch(self.table.len() * self.units * 4);
        s
    }
}

pub struct School {
    pub units: Vec<SchoolUnit>,
    weights: Weights,
    routing: Routing,
    line: NumberLine,
    /// How clearly one unit must beat the next for a question to count as a recognisable kind: learned from rewordings.
    pub route_floor: f32,
    pub seed: u64,
    /// Lessons that fit no unit it knows. They wait here until enough of them gather to form a unit of their own.
    pub pen: Vec<(String, String)>,
}

pub struct Reply {
    pub answer: Option<String>,
    pub unit: Option<usize>,
    pub how: String,
}

/// What happened when it was taught something after school.
pub struct Learned {
    pub unit: String,
    pub opened: bool,
    pub corrected: bool,
    pub was: Option<Mech>,
    pub now: Mech,
    pub held_back: f64,
    pub facts: usize,
    /// False when the unit could already work this lesson out: nothing was learned, nothing was stored.
    pub surprised: bool,
}

/// What the pen did with the lessons waiting in it.
pub struct PenReport {
    /// New units opened, with how many lessons each took.
    pub opened: Vec<(String, usize)>,
    /// Existing units that took lessons because the pen's group built exactly the same expression.
    pub joined: Vec<(String, usize)>,
    pub suggestions: Vec<String>,
    pub left: usize,
}

pub enum Outcome {
    Placed(Learned),
    Penned { pen: usize, report: PenReport },
}

/// What reorganising changed.
pub struct Reorg {
    /// (kept unit, unit folded into it)
    pub merged: Vec<(String, String)>,
    /// (unit it was taken from, where the lessons went)
    pub split: Vec<(String, String, usize)>,
}

/// A lesson's identity: the words in the order they were said. ("five minus two" and "two minus five" are different questions.)
fn lesson_key(q: &str) -> String {
    tokenize(q).join(" ")
}

pub struct Compacted {
    pub before: usize,
    pub after: usize,
}

/// The slow, exhaustive way to send a question to a unit (compare it with every lesson ever taught). Kept to measure what the
/// precomputed table saves.
pub struct NaiveRouter {
    shapes: Vec<Vec<Vec<String>>>,
    weights: Weights,
}

impl NaiveRouter {
    pub fn route_public(&self, shape: &[String], ledger: &mut Ledger) -> usize {
        self.route(shape, ledger)
    }

    fn route(&self, shape: &[String], ledger: &mut Ledger) -> usize {
        let mut best = (0, -1.0f32);
        for (u, shapes) in self.shapes.iter().enumerate() {
            for s in shapes {
                let (sim, steps) = self.weights.similarity_steps(shape, s);
                ledger.int_ops += steps;
                ledger.words_read += steps;
                ledger.macs += steps / 2;
                if sim > best.1 {
                    best = (u, sim);
                }
            }
        }
        best.0
    }
}

fn all_items_ref(units: &[SchoolUnit]) -> Vec<&Item> {
    units.iter().flat_map(|u| u.lessons.iter()).collect()
}

fn all_items(units: &[SchoolUnit]) -> Vec<Item> {
    units.iter().flat_map(|u| u.lessons.iter().cloned()).collect()
}

impl School {
    /// Teach a curriculum. `thrifty` skips the network wherever cheaper mechanisms already generalise.
    pub fn train(world: &World, seed: u64, thrifty: bool) -> School {
        let names: Vec<String> = UNITS.iter().map(|s| s.to_string()).collect();
        School::from_items(&world.train, names, Some(&world.reworded), seed, thrifty)
    }

    /// Teach any lessons that are already grouped into named units. `calibration` (rewordings of lessons) sets how decisive a
    /// question must be to be recognised; without it the floor comes from the lessons' own margins, more cautiously.
    pub fn from_items(train: &[Item], names: Vec<String>, calibration: Option<&[Item]>, seed: u64, thrifty: bool) -> School {
        let baby = mechanisms::learn_items(train, names.len(), seed, thrifty);
        let mut units = Vec::new();
        for (u, um) in baby.units.into_iter().enumerate() {
            let idx = Mech::ALL.iter().position(|m| *m == um.chosen).unwrap();
            let mut fitted = um.fitted;
            let state = fitted.swap_remove(idx);
            let lessons: Vec<Item> = train.iter().filter(|i| i.unit == u).cloned().collect();
            units.push(SchoolUnit {
                name: names[u].clone(),
                chosen: um.chosen,
                evidence: um.evidence,
                remember: Remembered::from_items(&lessons),
                lessons,
                state,
            });
        }
        let line = NumberLine::from_counting(&counting_lessons());
        let weights = Weights::from(&all_items(&units));
        let routing = Routing::build(&units, &line);
        let mut school = School { units, weights, routing, line, route_floor: f32::NEG_INFINITY, seed, pen: Vec::new() };
        // How decisive must a question be to count as a recognisable kind? Learned from rewordings of lessons it was taught.
        let own: Vec<&Item> = train.iter().collect();
        let (queries, safety): (Vec<&str>, f32) = match calibration {
            Some(c) => (c.iter().map(|i| i.query.as_str()).collect(), 0.25),
            None => (own.iter().map(|i| i.query.as_str()).collect(), 0.6),
        };
        let mut best: Vec<f32> = queries.iter().map(|q| school.route(q, &mut Ledger::default()).1).collect();
        best.sort_by(|a, b| a.partial_cmp(b).unwrap());
        // Keep 99% recognisable (the floor is a dial: lower recognises more, risks more).
        let p1 = best[(best.len() as f32 * 0.01) as usize];
        school.route_floor = if calibration.is_some() { p1 - safety * p1.abs() } else { p1 * safety };
        school
    }

    /// The unit a question most resembles, and by how much it beats the runner-up. (The margin, not the raw score: a raw score grows
    /// with the number of words, so a short question like "one plus zero" would look less recognisable than a long one.)
    pub fn route(&self, q: &str, ledger: &mut Ledger) -> (usize, f32) {
        let s = self.routing.scores(&shape(q, &self.line), ledger);
        let mut order: Vec<usize> = (0..s.len()).collect();
        order.sort_by(|a, b| s[*b].partial_cmp(&s[*a]).unwrap());
        let margin = if s.len() > 1 { s[order[0]] - s[order[1]] } else { s[order[0]] };
        (order[0], margin)
    }

    /// Words it knows (from any lesson, or the number line).
    pub fn knows(&self, w: &str) -> bool {
        self.weights.knows(w) || self.line.value(w).is_some()
    }

    pub fn known_words(&self) -> Vec<String> {
        let mut v = self.weights.words();
        v.extend(counting_lessons().into_iter().flat_map(|(a, b)| [a, b]));
        v.sort();
        v.dedup();
        v
    }

    pub fn word_weight(&self, w: &str) -> f32 {
        self.weights.w(w)
    }

    pub fn naive_router(&self) -> NaiveRouter {
        let shapes: Vec<Vec<Vec<String>>> =
            self.units.iter().map(|u| u.lessons.iter().map(|l| shape(&l.query, &self.line)).collect()).collect();
        let weights = Weights::from_lists(shapes.iter().enumerate().flat_map(|(u, qs)| qs.iter().map(move |q| (u, q.clone()))).collect());
        NaiveRouter { shapes, weights }
    }

    pub fn ask(&self, q: &str) -> Reply {
        self.ask_with(q, &mut Ledger::default(), None)
    }

    /// Answer, counting the work. With a `NaiveRouter` it does everything the slow way (and skips the "recognisable kind" check,
    /// which belongs to the table), so the two can be compared on cost.
    pub fn ask_with(&self, q: &str, ledger: &mut Ledger, naive: Option<&NaiveRouter>) -> Reply {
        let tokens = tokenize(q);
        ledger.int_ops += tokens.len() as u64;
        ledger.words_read += tokens.len() as u64;
        // A word it has never met cannot be part of something it knows how to answer.
        let unmet: Vec<String> = tokens.iter().filter(|w| !self.knows(w)).cloned().collect();
        if !unmet.is_empty() {
            return Reply { answer: None, unit: None, how: format!("I haven't met the word{} {}", if unmet.len() > 1 { "s" } else { "" }, unmet.join(", ")) };
        }
        let u = match naive {
            Some(r) => r.route(&shape(q, &self.line), ledger),
            None => {
                let (u, score) = self.route(q, ledger);
                if score < self.route_floor {
                    return Reply { answer: None, unit: None, how: "no kind of lesson I was taught is like this question".into() };
                }
                u
            }
        };
        let unit = &self.units[u];
        let said = |what: &str| format!("unit '{}', {what}", unit.name);
        let n = tokens.len() as u64;
        match &unit.state {
            Fitted::Network(m) => {
                let c = m.net.cfg;
                ledger.macs += (c.hidden * c.input() + c.classes * c.hidden) as u64;
                ledger.words_read += (c.hidden * c.input() + c.classes * c.hidden) as u64;
                ledger.touch(4 * c.n_params());
                let mut s = Scratch::new(&c);
                match m.ask(q, 1, &mut s).into_iter().next() {
                    Some((a, p)) if p >= 0.9 => Reply { answer: Some(a), unit: Some(u), how: said(&format!("a small network ({:.0}% sure)", p * 100.0)) },
                    _ => Reply { answer: None, unit: Some(u), how: said("a small network, but it is not sure enough") },
                }
            }
            state => {
                let answer = match state {
                    Fitted::Remember(_) => unit.remember.recall_counted(q, &self.weights, ledger, naive.is_none()),
                    other => {
                        match other {
                            Fitted::Rule(_) => ledger.macs += 3 * n + 11,
                            Fitted::Select(_) => ledger.macs += 2 * n + 4,
                            Fitted::Program(Some(p)) => ledger.macs += p.expr.size() as u64 + n,
                            _ => {}
                        }
                        ledger.words_read += n + 11;
                        predict(other, q, &self.weights)
                    }
                };
                match answer {
                    Some(a) => {
                        let how = match unit.chosen {
                            Mech::Remember if unit.memorise_only() => {
                                said("recalled, not worked out: nothing I tried could work these out, so I only remember what I was told")
                            }
                            Mech::Remember => said("recalled from what I was taught"),
                            Mech::Rule => said(&format!("worked out by a learned rule ({:.0}% on facts I held back)", 100.0 * unit.best_evidence())),
                            Mech::Select => said(&format!("worked out by picking the bigger or smaller ({:.0}% on facts I held back)", 100.0 * unit.best_evidence())),
                            Mech::Program => said(&format!(
                                "worked out by an expression I built, {} ({:.0}% on facts I held back)",
                                unit.program().unwrap_or_default(),
                                100.0 * unit.best_evidence()
                            )),
                            Mech::Network => said("a small network"),
                        };
                        Reply { answer: Some(a), unit: Some(u), how }
                    }
                    None => {
                        let why = match unit.chosen {
                            Mech::Remember => "I only memorise this kind of thing and nothing I was taught is close enough",
                            _ => "I couldn't work it out: it has a word or a shape I haven't learned how to handle",
                        };
                        Reply { answer: None, unit: Some(u), how: said(why) }
                    }
                }
            }
        }
    }

    // ---- learning after school ----

    fn rebuild_tables(&mut self) {
        self.renumber();
        self.weights = Weights::from(&all_items(&self.units));
        self.routing = Routing::build(&self.units, &self.line);
    }

    /// Work out again how to learn a unit, from every lesson it now has.
    fn relearn_unit(&mut self, u: usize) {
        let items = self.units[u].lessons.clone();
        let ev = evidence_thrifty(&items, &self.weights, self.seed);
        let facts = self.units[u].facts();
        let chosen = choose_mechanism(&ev, facts);
        let state = fit(chosen, &items, &self.weights, self.seed);
        let unit = &mut self.units[u];
        unit.evidence = ev;
        unit.chosen = chosen;
        unit.remember = Remembered::from_items(&items);
        unit.state = state;
    }

    /// The cheap way to take on a lesson when nothing structural has changed: refit the unit's current mechanism on all its lessons and
    /// check it still explains every one. If so there is nothing to re-decide (no cross-validation). Returns false when it does not,
    /// or when there is no settled mechanism to refit: then the unit must be worked out again from scratch.
    fn refit_in_place(&mut self, u: usize) -> bool {
        let unit = &self.units[u];
        let settled_rule = matches!(unit.chosen, Mech::Rule | Mech::Select | Mech::Program) && unit.best_evidence() >= 0.95;
        // A unit of facts that is already known not to be workable-out is re-tested only every ten new facts.
        let settled_facts = unit.chosen == Mech::Remember && unit.best_evidence() < 0.25 && unit.facts() >= 10 && unit.facts() % 10 != 0;
        if !(settled_rule || settled_facts) {
            return false;
        }
        let items = unit.lessons.clone();
        let chosen = unit.chosen;
        let state = fit(chosen, &items, &self.weights, self.seed);
        if settled_rule && !items.iter().all(|l| predict(&state, &l.query, &self.weights).as_deref() == Some(l.answer.as_str())) {
            return false;
        }
        let unit = &mut self.units[u];
        unit.remember = Remembered::from_items(&items);
        unit.state = if chosen == Mech::Remember { Fitted::Remember(unit.remember.clone()) } else { state };
        true
    }

    /// Teach it one more lesson, after school. With `unit` it joins that unit (or opens one of that name); without, it joins the
    /// unit the question resembles, if it recognises a kind at all. Then the unit's way of learning is worked out again.
    pub fn learn(&mut self, query: &str, answer: &str, unit: Option<&str>) -> Result<Learned, String> {
        let tokens = tokenize(query);
        if tokens.is_empty() {
            return Err("nothing to learn: the question is empty".into());
        }
        let (u, opened) = match unit {
            Some(name) => match self.units.iter().position(|x| x.name == name) {
                Some(u) => (u, false),
                None => {
                    // A new kind of lesson. It starts as plain memory until it has enough facts to be tested.
                    self.units.push(SchoolUnit {
                        name: name.to_string(),
                        chosen: Mech::Remember,
                        evidence: [0.0; 5],
                        lessons: Vec::new(),
                        remember: Remembered::from_items(&[]),
                        state: Fitted::Remember(Remembered::from_items(&[])),
                    });
                    (self.units.len() - 1, true)
                }
            },
            None => {
                let unmet: Vec<&String> = tokens.iter().filter(|w| !self.knows(w)).collect();
                if !unmet.is_empty() {
                    return Err(format!(
                        "I haven't met {}, so I can't tell which kind of lesson this is. Say where it belongs with --unit NAME (an existing unit to add a phrasing, or a new name to open one)",
                        unmet.iter().map(|w| format!("'{w}'")).collect::<Vec<_>>().join(", ")
                    ));
                }
                let (u, score) = self.route(query, &mut Ledger::default());
                if score < self.route_floor {
                    return Err("this doesn't look like any kind of lesson I know. Give it a name with --unit NAME to open a new unit".into());
                }
                (u, false)
            }
        };
        let was = if opened { None } else { Some(self.units[u].chosen) };
        // Surprise-gated: if the unit can already work this out, there is nothing to learn and nothing to keep.
        if !opened
            && self.units[u].lessons.iter().all(|l| lesson_key(&l.query) != lesson_key(query))
            && self.unit_predicts(u, query).as_deref() == Some(answer)
        {
            let unit = &self.units[u];
            return Ok(Learned { unit: unit.name.clone(), opened: false, corrected: false, was, now: unit.chosen, held_back: unit.best_evidence(), facts: unit.facts(), surprised: false });
        }
        let key = lesson_key(query);
        let corrected;
        if let Some(existing) = self.units[u].lessons.iter_mut().find(|l| lesson_key(&l.query) == key) {
            existing.answer = answer.to_string();
            corrected = true;
        } else {
            let fact = all_items(&self.units).iter().map(|l| l.fact).max().map_or(0, |m| m + 1);
            let template = self.units[u].lessons.len() % 4;
            self.units[u].lessons.push(Item { unit: u, fact, template, query: query.to_string(), answer: answer.to_string() });
            corrected = false;
        }
        self.rebuild_tables();
        if !self.refit_in_place(u) {
            self.relearn_unit(u);
        }
        let unit = &self.units[u];
        Ok(Learned { unit: unit.name.clone(), opened, corrected, was, now: unit.chosen, held_back: unit.best_evidence(), facts: unit.facts(), surprised: true })
    }

    fn renumber(&mut self) {
        for (i, u) in self.units.iter_mut().enumerate() {
            for l in &mut u.lessons {
                l.unit = i;
            }
        }
    }

    fn word_keyed(&self, u: usize) -> bool {
        matches!(self.units[u].chosen, Mech::Rule | Mech::Select) && self.units[u].best_evidence() >= 0.95
    }

    /// Look again at how the lessons are grouped. A unit that is no longer fully explained (its newer lessons stopped following one
    /// rule) is taken apart if its lessons separate into groups that each ARE explained; two units that were taught separately but
    /// are right for each other's lessons under one rule, or a rule per word, become one.
    pub fn reorganise(&mut self) -> Reorg {
        let mut report = Reorg { merged: Vec::new(), split: Vec::new() };

        // 1. Split what has stopped following one rule.
        let candidates: Vec<String> = self
            .units
            .iter()
            .filter(|u| {
                let numeric = u.lessons.iter().filter(|l| tokenize(&l.query).iter().any(|w| self.line.value(w).is_some())).count();
                u.facts() >= 10 && u.best_evidence() < 0.95 && numeric * 10 >= u.lessons.len() * 8
            })
            .map(|u| u.name.clone())
            .collect();
        for name in candidates {
            let Some(u) = self.units.iter().position(|x| x.name == name) else { continue };
            let pairs: Vec<(String, String)> = self.units[u].lessons.iter().map(|l| (l.query.clone(), l.answer.clone())).collect();
            let d = crate::discover::discover(&pairs, self.seed);
            if d.names.len() < 2 {
                continue;
            }
            let mut moved: Vec<usize> = Vec::new(); // indices into this unit's lessons that leave
            let mut targets: Vec<(usize, Vec<Item>)> = Vec::new();
            // The biggest group stays where it is: the unit is its, and the others leave.
            let biggest = (0..d.names.len()).max_by_key(|g| d.items.iter().filter(|it| it.unit == *g).count()).unwrap_or(0);
            for g in 0..d.names.len() {
                if g == biggest {
                    continue;
                }
                let group: Vec<(&Item, usize)> = d.items.iter().zip(&d.origin).filter(|(it, _)| it.unit == g).map(|(it, o)| (it, *o)).collect();
                let items: Vec<Item> = group.iter().map(|(it, _)| (*it).clone()).collect();
                let facts = items.iter().map(|i| i.fact).collect::<std::collections::HashSet<_>>().len();
                if facts < 5 {
                    continue;
                }
                let best = mechanisms::cv_accuracy(&items, &self.weights, self.seed, &[1, 2, 3]).into_iter().fold(0.0, f64::max);
                if best < 0.9 {
                    continue;
                }
                // Where do they go? To a unit that builds exactly this expression, if there is one; else a unit of their own.
                let own = ProgramRule::fit_unique(&items).map(|p| p.expr.to_text());
                let same = own.as_ref().and_then(|t| {
                    self.units.iter().enumerate().position(|(i, x)| i != u && matches!(&x.state, Fitted::Program(Some(p)) if &p.expr.to_text() == t))
                });
                let target = match same {
                    Some(t) => t,
                    None => {
                        let mut unique = d.names[g].clone();
                        let mut n = 2;
                        while self.units.iter().any(|x| x.name == unique) {
                            unique = format!("{}{n}", d.names[g]);
                            n += 1;
                        }
                        self.units.push(SchoolUnit {
                            name: unique,
                            chosen: Mech::Remember,
                            evidence: [0.0; 5],
                            lessons: Vec::new(),
                            remember: Remembered::from_items(&[]),
                            state: Fitted::Remember(Remembered::from_items(&[])),
                        });
                        self.units.len() - 1
                    }
                };
                moved.extend(group.iter().map(|(_, o)| *o));
                targets.push((target, items));
            }
            if moved.is_empty() {
                continue;
            }
            let fact_base = all_items(&self.units).iter().map(|l| l.fact).max().map_or(0, |m| m + 1);
            let mut keep: Vec<Item> = Vec::new();
            for (i, l) in std::mem::take(&mut self.units[u].lessons).into_iter().enumerate() {
                if !moved.contains(&i) {
                    keep.push(l);
                }
            }
            self.units[u].lessons = keep;
            for (target, items) in &targets {
                for it in items {
                    self.units[*target].lessons.push(Item { unit: *target, fact: it.fact + fact_base, template: 0, query: it.query.clone(), answer: it.answer.clone() });
                }
                report.split.push((name.clone(), self.units[*target].name.clone(), items.len()));
            }
            self.rebuild_tables();
            let mut touched: Vec<usize> = targets.iter().map(|t| t.0).chain(std::iter::once(u)).collect();
            touched.sort();
            touched.dedup();
            for t in touched {
                self.relearn_unit(t);
            }
        }

        // 2. Merge what turns out to be one.
        loop {
            let n = self.units.len();
            let mut found = None;
            'outer: for a in 0..n {
                for b in a + 1..n {
                    if self.word_keyed(a) && self.word_keyed(b) && crate::discover::can_merge(&self.units[a].lessons, &self.units[b].lessons, &self.weights, self.seed) {
                        found = Some((a, b));
                        break 'outer;
                    }
                }
            }
            let Some((a, b)) = found else { break };
            let gone = self.units.remove(b);
            let keep_name = self.units[a].name.clone();
            report.merged.push((keep_name.clone(), gone.name.clone()));
            self.units[a].lessons.extend(gone.lessons);
            let parts: Vec<&str> = keep_name.split('/').collect();
            self.units[a].name = if parts.len() >= 3 { format!("{}/{}+", parts[0], parts[1]) } else { format!("{keep_name}/{}", gone.name) };
            self.renumber();
            self.rebuild_tables();
            self.relearn_unit(a);
        }
        self.renumber();
        self.rebuild_tables();
        if !report.merged.is_empty() || !report.split.is_empty() {
            self.recalibrate_floor();
        }
        report
    }

    /// Set the recognition floor from the table as it is now: low enough that every lesson it holds is recognised, with room below
    /// for phrasings it has not seen (those are less decisive than taught ones).
    fn recalibrate_floor(&mut self) {
        let mut margins: Vec<f32> = all_items_ref(&self.units).into_iter().map(|l| self.route(&l.query, &mut Ledger::default()).1).collect();
        margins.sort_by(|a, b| a.partial_cmp(b).unwrap());
        if let Some(m) = margins.first() {
            self.route_floor = m - 0.6 * m.abs();
        }
    }

    /// What this one unit would answer, ignoring whether the question was sent to it.
    fn unit_predicts(&self, u: usize, q: &str) -> Option<String> {
        let unit = &self.units[u];
        match &unit.state {
            Fitted::Network(_) => None,
            Fitted::Remember(_) => unit.remember.recall_counted(q, &self.weights, &mut Ledger::default(), true),
            other => predict(other, q, &self.weights),
        }
    }

    // ---- going on learning without being told where things belong ----

    /// Teach it one more lesson without saying which unit it belongs to. If it recognises a kind of lesson it joins that unit.
    /// If not, it goes to the pen; and when enough pen lessons gather to be a kind of their own, a unit opens by itself.
    pub fn learn_anywhere(&mut self, query: &str, answer: &str) -> Result<Outcome, String> {
        let tokens = tokenize(query);
        if tokens.is_empty() {
            return Err("nothing to learn: the question is empty".into());
        }
        let unmet = tokens.iter().any(|w| !self.knows(w));
        let recognised = !unmet && self.route(query, &mut Ledger::default()).1 >= self.route_floor;
        if recognised {
            return self.learn(query, answer, None).map(Outcome::Placed);
        }
        match self.pen.iter_mut().find(|(q, _)| lesson_key(q) == lesson_key(query)) {
            Some(existing) => existing.1 = answer.to_string(),
            None => self.pen.push((query.to_string(), answer.to_string())),
        }
        let report = self.process_pen();
        Ok(Outcome::Penned { pen: self.pen.len(), report })
    }

    /// Look at the lessons waiting in the pen. A group of at least five different facts that follows one rule becomes a unit (or
    /// joins the unit that already builds exactly the same expression). Anything smaller stays, with a note if an existing unit
    /// already gets it right, for the teacher to confirm.
    pub fn process_pen(&mut self) -> PenReport {
        let mut report = PenReport { opened: Vec::new(), joined: Vec::new(), suggestions: Vec::new(), left: 0 };
        let mut used = vec![false; self.pen.len()];
        let mut changed: Vec<usize> = Vec::new();
        if self.pen.len() >= 6 {
            let d = crate::discover::discover(&self.pen, self.seed);
            let fact_base = all_items(&self.units).iter().map(|l| l.fact).max().map_or(0, |m| m + 1);
            for (u, name) in d.names.iter().enumerate() {
                let group: Vec<(&Item, usize)> = d.items.iter().zip(&d.origin).filter(|(it, _)| it.unit == u).map(|(it, o)| (it, *o)).collect();
                let facts = group.iter().map(|(it, _)| it.fact).collect::<std::collections::HashSet<_>>().len();
                if facts < 5 {
                    continue;
                }
                let items: Vec<Item> = group.iter().map(|(it, _)| (*it).clone()).collect();
                let own = ProgramRule::fit_unique(&items).map(|p| p.expr.to_text());
                let same = own.as_ref().and_then(|t| self.units.iter().position(|x| matches!(&x.state, Fitted::Program(Some(p)) if &p.expr.to_text() == t)));
                let target = match same {
                    Some(e) => e,
                    None => {
                        // Name it after the words that are new to the baby: they are what makes this a kind of its own.
                        let new_words: Vec<String> = {
                            let mut v: Vec<(usize, String)> = group
                                .iter()
                                .flat_map(|(it, _)| tokenize(&it.query))
                                .collect::<std::collections::BTreeSet<_>>()
                                .into_iter()
                                .filter(|t| !self.knows(t))
                                .map(|t| (group.iter().filter(|(it, _)| tokenize(&it.query).contains(&t)).count(), t))
                                .collect();
                            v.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
                            v.into_iter().take(2).map(|x| x.1).collect()
                        };
                        let name = if new_words.is_empty() { name.clone() } else { new_words.join("/") };
                        let mut unique = name.clone();
                        let mut n = 2;
                        while self.units.iter().any(|x| x.name == unique) {
                            unique = format!("{name}{n}");
                            n += 1;
                        }
                        self.units.push(SchoolUnit {
                            name: unique,
                            chosen: Mech::Remember,
                            evidence: [0.0; 5],
                            lessons: Vec::new(),
                            remember: Remembered::from_items(&[]),
                            state: Fitted::Remember(Remembered::from_items(&[])),
                        });
                        self.units.len() - 1
                    }
                };
                for (it, o) in &group {
                    self.units[target].lessons.push(Item { unit: target, fact: it.fact + fact_base, template: 0, query: it.query.clone(), answer: it.answer.clone() });
                    used[*o] = true;
                }
                if same.is_some() {
                    report.joined.push((self.units[target].name.clone(), group.len()));
                } else {
                    report.opened.push((self.units[target].name.clone(), group.len()));
                }
                changed.push(target);
                // New facts need fresh ids that cannot collide with the next batch.
                let _ = fact_base;
            }
        }
        let leftover: Vec<(String, String)> =
            self.pen.iter().enumerate().filter(|(i, _)| !used[*i]).map(|(_, l)| l.clone()).collect();
        self.pen = leftover;
        if !changed.is_empty() {
            self.rebuild_tables();
            changed.sort();
            changed.dedup();
            for u in changed {
                self.relearn_unit(u);
            }
        }
        // For what is left: does an existing unit already work these out? Say so, and let the teacher confirm.
        if self.pen.len() >= 3 {
            for unit in &self.units {
                let right = self
                    .pen
                    .iter()
                    .filter(|(q, a)| {
                        let got = match &unit.state {
                            Fitted::Rule(r) => r.answer_ignoring(q, &|_| true).filter(|(_, d)| *d < 0.25).map(|x| x.0),
                            Fitted::Select(s) => s.predict(q, &|_| true),
                            Fitted::Program(Some(p)) => p.predict(q, &|_| true),
                            _ => None,
                        };
                        got.as_deref() == Some(a.as_str())
                    })
                    .count();
                // A built expression is only a fair suggestion if the waiting lessons admit exactly that expression and nothing else.
                let program_ok = match &unit.state {
                    Fitted::Program(Some(p)) => {
                        let items: Vec<Item> = self.pen.iter().enumerate().map(|(i, (q, a))| Item { unit: 0, fact: i, template: 0, query: q.clone(), answer: a.clone() }).collect();
                        ProgramRule::fit_unique(&items).map_or(false, |f| f.expr.to_text() == p.expr.to_text())
                    }
                    _ => true,
                };
                if program_ok && right * 20 >= self.pen.len() * 19 && unit.best_evidence() >= 0.95 {
                    report.suggestions.push(format!(
                        "all {} lessons waiting look like another way of asking unit '{}' (what it already knows gets every one right): confirm with --unit {}",
                        self.pen.len(), unit.name, unit.name
                    ));
                }
            }
        }
        report.left = self.pen.len();
        report
    }

    /// Forget what is already explained. A unit that learned a rule, a selection or an expression does not need every lesson it was
    /// taught: it keeps a small support set (the facts the fitted unit could not have got right without, and every word it needs
    /// to understand). A unit of facts keeps two phrasings of each. What it knows does not change.
    pub fn compact(&mut self) -> Compacted {
        let before: usize = self.units.iter().map(|u| u.lessons.len()).sum();
        let mut removed: Vec<Item> = Vec::new();
        for u in 0..self.units.len() {
            let mask = self.keep_mask(u);
            let lessons = std::mem::take(&mut self.units[u].lessons);
            for (l, keep) in lessons.into_iter().zip(mask) {
                if keep {
                    self.units[u].lessons.push(l);
                } else {
                    removed.push(l);
                }
            }
        }
        // The promise: it must still answer EVERY lesson it was ever taught, kept or let go. Check, and put back whatever it can no
        // longer answer (typically a phrasing the table or the memory only recognised because of that lesson), until it can.
        loop {
            for u in 0..self.units.len() {
                if self.units[u].chosen == Mech::Remember {
                    let r = Remembered::from_items(&self.units[u].lessons);
                    self.units[u].remember = r.clone();
                    self.units[u].state = Fitted::Remember(r);
                }
            }
            self.rebuild_tables();
            // Recognition floor: low enough that EVERYTHING it was taught, kept or let go, is recognised.
            let mut margins: Vec<f32> =
                all_items_ref(&self.units).into_iter().chain(removed.iter()).map(|l| self.route(&l.query, &mut Ledger::default()).1).collect();
            margins.sort_by(|a, b| a.partial_cmp(b).unwrap());
            // Unseen phrasings are less decisive than taught ones, so leave room below the least decisive lesson.
            self.route_floor = margins[0] - 0.6 * margins[0].abs();
            let (back, still): (Vec<Item>, Vec<Item>) =
                std::mem::take(&mut removed).into_iter().partition(|l| self.ask(&l.query).answer.as_deref() != Some(l.answer.as_str()));
            removed = still;
            if back.is_empty() {
                break;
            }
            for l in back {
                let u = l.unit;
                self.units[u].lessons.push(l);
            }
        }
        let after: usize = self.units.iter().map(|u| u.lessons.len()).sum();
        Compacted { before, after }
    }

    fn keep_mask(&self, u: usize) -> Vec<bool> {
        let unit = &self.units[u];
        let n = unit.lessons.len();
        match unit.chosen {
            Mech::Rule | Mech::Select | Mech::Program if unit.best_evidence() >= 0.95 => {
                let mut facts: Vec<usize> = unit.lessons.iter().map(|l| l.fact).collect();
                facts.sort();
                facts.dedup();
                crate::rng::Rng::new(self.seed.wrapping_add(u as u64 * 17 + 3)).shuffle(&mut facts);
                let mut chosen: Vec<usize> = facts.iter().take(6).cloned().collect();
                loop {
                    let items: Vec<Item> = unit.lessons.iter().filter(|l| chosen.contains(&l.fact)).cloned().collect();
                    let fitted = fit(unit.chosen, &items, &self.weights, self.seed);
                    let wrong = facts.iter().find(|f| {
                        !chosen.contains(f)
                            && unit.lessons.iter().filter(|l| l.fact == **f).any(|l| predict(&fitted, &l.query, &self.weights).as_deref() != Some(l.answer.as_str()))
                    });
                    match wrong {
                        Some(f) => chosen.push(*f),
                        None => break,
                    }
                }
                unit.lessons.iter().map(|l| chosen.contains(&l.fact)).collect()
            }
            Mech::Remember => {
                let mut seen: HashMap<usize, usize> = HashMap::new();
                unit.lessons
                    .iter()
                    .map(|l| {
                        let c = seen.entry(l.fact).or_default();
                        *c += 1;
                        *c <= 2
                    })
                    .collect()
            }
            _ => vec![true; n],
        }
    }

    // ---- saving and loading ----

    fn state_text(&self, i: usize, path: &str) -> std::io::Result<String> {
        let mut out = String::new();
        match &self.units[i].state {
            Fitted::Remember(_) => writeln!(out, "state remember").unwrap(),
            Fitted::Rule(r) => {
                let ops = r.all_ops();
                writeln!(out, "state rule {}", ops.len()).unwrap();
                for (w, o) in ops {
                    writeln!(out, "o {w} {} {} {}", o[0], o[1], o[2]).unwrap();
                }
            }
            Fitted::Select(s) => {
                let sc = s.scores();
                writeln!(out, "state select {}", sc.len()).unwrap();
                for (w, x) in sc {
                    writeln!(out, "c {w} {x}").unwrap();
                }
            }
            Fitted::Program(Some(p)) => {
                writeln!(out, "state program {}", p.arity()).unwrap();
                writeln!(out, "expr {}", p.expr.to_text()).unwrap();
                writeln!(out, "vocab {}", p.vocab().join(" ")).unwrap();
            }
            Fitted::Program(None) => writeln!(out, "state none").unwrap(),
            Fitted::Network(m) => {
                let file = format!("{path}.unit{i}.model");
                m.save(&file)?;
                writeln!(out, "state network {file}").unwrap();
            }
        }
        Ok(out)
    }

    fn lessons_text(&self, i: usize) -> String {
        let u = &self.units[i];
        let mut out = String::new();
        writeln!(out, "lessons {}", u.lessons.len()).unwrap();
        for l in &u.lessons {
            writeln!(out, "l {} {} {}\t{}", l.fact, l.template, l.answer, l.query).unwrap();
        }
        out
    }

    /// For each unit: (name, mechanism, lessons kept, bytes of lessons, bytes of what it learned from them).
    pub fn storage(&self) -> Vec<(String, Mech, usize, usize, usize)> {
        (0..self.units.len())
            .map(|i| {
                let state = self.state_text(i, "/dev/null/none").map(|t| t.len()).unwrap_or(0);
                (self.units[i].name.clone(), self.units[i].chosen, self.units[i].lessons.len(), self.lessons_text(i).len(), state)
            })
            .collect()
    }

    pub fn save(&self, path: &str) -> std::io::Result<()> {
        let mut out = String::new();
        writeln!(out, "SCHOOL2").unwrap();
        writeln!(out, "seed {}", self.seed).unwrap();
        writeln!(out, "route_floor {}", self.route_floor).unwrap();
        writeln!(out, "pen {}", self.pen.len()).unwrap();
        for (q, a) in &self.pen {
            writeln!(out, "p {a}\t{q}").unwrap();
        }
        writeln!(out, "units {}", self.units.len()).unwrap();
        for (i, u) in self.units.iter().enumerate() {
            let e = u.evidence;
            writeln!(out, "unit {i} {} {} {} {} {} {} {}", u.name, u.chosen.name(), e[0], e[1], e[2], e[3], e[4]).unwrap();
            out.push_str(&self.lessons_text(i));
            out.push_str(&self.state_text(i, path)?);
        }
        writeln!(out, "end").unwrap();
        if let Some(dir) = std::path::Path::new(path).parent() {
            std::fs::create_dir_all(dir)?;
        }
        std::fs::write(path, out)
    }

    pub fn load(path: &str) -> Result<School, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("cannot read {path}: {e}"))?;
        let mut it = text.lines();
        if it.next() != Some("SCHOOL2") {
            return Err(format!("{path}: not a SCHOOL2 file (teach it again with: baby school)"));
        }
        let field = |l: Option<&str>, key: &str| -> Result<String, String> {
            l.and_then(|l| l.strip_prefix(&format!("{key} "))).map(String::from).ok_or_else(|| format!("{path}: expected '{key}'"))
        };
        let seed: u64 = field(it.next(), "seed")?.parse().map_err(|e| format!("{e}"))?;
        let route_floor: f32 = field(it.next(), "route_floor")?.parse().map_err(|e| format!("{e}"))?;
        let pen_n: usize = field(it.next(), "pen")?.parse().map_err(|e| format!("{e}"))?;
        let mut pen = Vec::new();
        for _ in 0..pen_n {
            let l = it.next().ok_or("pen line")?.strip_prefix("p ").ok_or("bad pen line")?;
            let (a, q) = l.split_once('\t').ok_or("bad pen line")?;
            pen.push((q.to_string(), a.to_string()));
        }
        let count: usize = field(it.next(), "units")?.parse().map_err(|e| format!("{e}"))?;
        let mut units = Vec::new();
        for u in 0..count {
            let head: Vec<&str> = it.next().ok_or("missing unit")?.split_whitespace().collect();
            if head.len() != 9 || head[0] != "unit" {
                return Err(format!("{path}: bad unit line"));
            }
            let chosen = Mech::ALL.iter().find(|m| m.name() == head[3]).copied().ok_or("unknown mechanism")?;
            let mut evidence = [0.0f64; 5];
            for k in 0..5 {
                evidence[k] = head[4 + k].parse().map_err(|e| format!("{e}"))?;
            }
            let n: usize = field(it.next(), "lessons")?.parse().map_err(|e| format!("{e}"))?;
            let mut lessons = Vec::new();
            for _ in 0..n {
                let l = it.next().ok_or("lesson line")?.strip_prefix("l ").ok_or("bad lesson line")?;
                let (meta, query) = l.split_once('\t').ok_or("bad lesson line")?;
                let m: Vec<&str> = meta.split_whitespace().collect();
                lessons.push(Item {
                    unit: u,
                    fact: m[0].parse().map_err(|e| format!("{e}"))?,
                    template: m[1].parse().map_err(|e| format!("{e}"))?,
                    answer: m[2].to_string(),
                    query: query.to_string(),
                });
            }
            let remember = Remembered::from_items(&lessons);
            let state_line: Vec<&str> = it.next().ok_or("state")?.split_whitespace().collect();
            let state = match state_line.as_slice() {
                ["state", "remember"] => Fitted::Remember(remember.clone()),
                ["state", "none"] => Fitted::Program(None),
                ["state", "rule", n] => {
                    let n: usize = n.parse().map_err(|e| format!("{e}"))?;
                    let mut ops = Vec::new();
                    for _ in 0..n {
                        let f: Vec<&str> = it.next().ok_or("rule line")?.split_whitespace().collect();
                        let p = |i: usize| f[i].parse::<f32>().map_err(|e| format!("{e}"));
                        ops.push((f[1].to_string(), [p(2)?, p(3)?, p(4)?]));
                    }
                    Fitted::Rule(Quantity::from_ops(ops))
                }
                ["state", "select", n] => {
                    let n: usize = n.parse().map_err(|e| format!("{e}"))?;
                    let mut sc = Vec::new();
                    for _ in 0..n {
                        let f: Vec<&str> = it.next().ok_or("select line")?.split_whitespace().collect();
                        sc.push((f[1].to_string(), f[2].parse::<f32>().map_err(|e| format!("{e}"))?));
                    }
                    Fitted::Select(SelectRule::from_scores(sc))
                }
                ["state", "program", arity] => {
                    let arity: usize = arity.parse().map_err(|e| format!("{e}"))?;
                    let expr = Expr::from_text(it.next().ok_or("expr")?.strip_prefix("expr ").ok_or("bad expr")?).ok_or("bad expression")?;
                    let vocab: Vec<String> = it.next().ok_or("vocab")?.split_whitespace().skip(1).map(String::from).collect();
                    Fitted::Program(Some(ProgramRule::rebuild(expr, arity, vocab)))
                }
                ["state", "network", file] => Fitted::Network(Box::new(Model::load(file)?)),
                other => return Err(format!("{path}: bad state line {other:?}")),
            };
            units.push(SchoolUnit { name: head[2].to_string(), chosen, evidence, lessons, remember, state });
        }
        let line = NumberLine::from_counting(&counting_lessons());
        let weights = Weights::from(&all_items(&units));
        let routing = Routing::build(&units, &line);
        Ok(School { units, weights, routing, line, route_floor, seed, pen })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kinds;

    fn trained() -> School {
        School::train(&kinds::world(1), 1, true)
    }

    #[test]
    fn it_answers_what_it_can_work_out_and_says_how() {
        let s = trained();
        let r = s.ask("what is seven times one");
        assert_eq!(r.answer.as_deref(), Some("seven"));
        assert!(r.how.contains("expression I built"), "{}", r.how);
        assert_eq!(s.ask("nine minus four").answer.as_deref(), Some("five"));
        assert_eq!(s.ask("which is bigger seven or nine").answer.as_deref(), Some("nine"));
        assert_eq!(s.ask("how far apart are two and ten").answer.as_deref(), Some("eight"));
        assert_eq!(s.ask("capital of peru").answer.as_deref(), Some("lima"));
        assert_eq!(s.ask("what is half of ten").answer.as_deref(), Some("five"));
    }

    #[test]
    fn nonsense_unrelated_and_out_of_range_questions_are_refused_never_answered() {
        let s = trained();
        for q in [
            "capital of banana", "color of france", "what does the sky say", "opposite of paris", "what is the capital of three",
            "double banana", "what is hot times cold", "how far apart are paris and rome", "which is bigger paris or rome",
            "what comes after banana", "half of cat", "what is the meaning of life", "who won the world cup", "tell me a joke", "hello",
            "what time is it", "what is five times five", "what is ten plus ten", "three minus five", "what comes after ten",
            "half of seven", "what is 3 plus 4",
        ] {
            let r = s.ask(q);
            assert!(r.answer.is_none(), "answered '{q}' with {:?} ({})", r.answer, r.how);
        }
    }

    #[test]
    fn in_a_memorise_only_unit_it_never_answers_a_fact_it_was_not_taught() {
        let s = trained();
        let nums = crate::data::NUM;
        let (mut right, mut refused) = (0, 0);
        for n in 0..11 {
            let r = s.ask(&format!("what is the code for {}", nums[n]));
            match r.answer.as_deref() {
                Some(a) => {
                    assert_eq!(a, nums[kinds::CODE[n]], "bluffed on the code for {}", nums[n]);
                    right += 1;
                }
                None => refused += 1,
            }
        }
        assert!(right >= 7 && refused >= 1, "right {right}, refused {refused}");
    }

    #[test]
    fn forgetting_what_is_explained_halves_the_lessons_and_keeps_every_promise() {
        let w = kinds::world(1);
        let mut s = School::train(&w, 1, true);
        let c = s.compact();
        assert!(c.after * 10 < c.before * 6, "kept {} of {}", c.after, c.before);
        // It still answers every lesson it was ever taught, right, and never wrongly on anything else it is asked.
        for it in &w.train {
            assert_eq!(s.ask(&it.query).answer.as_deref(), Some(it.answer.as_str()), "forgot how to answer '{}'", it.query);
        }
        for it in w.reworded.iter().chain(&w.new_items).chain(&w.beyond) {
            if let Some(a) = s.ask(&it.query).answer {
                assert_eq!(a, it.answer, "answered '{}' wrongly after forgetting", it.query);
            }
        }
        for q in ["capital of banana", "three minus five", "what is hot times cold", "tell me a joke", "what comes after ten", "half of cat"] {
            assert!(s.ask(q).answer.is_none(), "answered '{q}' after forgetting");
        }
    }

    #[test]
    fn lessons_that_fit_nothing_wait_in_a_pen_and_become_a_unit_when_enough_gather() {
        let mut s = trained();
        let units = s.units.len();
        for (n, a) in [("zero", "ten"), ("one", "nine"), ("two", "eight"), ("three", "seven"), ("four", "six"), ("five", "five")] {
            let out = s.learn_anywhere(&format!("what is {n} less than ten"), a).unwrap();
            assert!(matches!(out, Outcome::Penned { .. }), "a kind of lesson it has not seen must wait, not be forced into a unit");
        }
        assert_eq!(s.units.len(), units + 1, "six lessons of one new kind should have opened a unit by themselves");
        assert!(s.pen.is_empty());
        let opened = s.units.last().unwrap();
        assert_ne!(opened.chosen, Mech::Remember, "it should have found something that generalises: {:?}", opened.chosen);
        assert_eq!(s.ask("what is eight less than ten").answer.as_deref(), Some("two"));
    }

    #[test]
    fn a_lesson_it_can_already_work_out_costs_nothing_and_is_not_kept() {
        let mut s = trained();
        let lessons_before: usize = s.units.iter().map(|u| u.lessons.len()).sum();
        // The sums rule was taught up to five; "seven plus two" it can already work out.
        let r = s.learn("seven plus two", "nine", Some("sum")).unwrap();
        assert!(!r.surprised, "it already knew this");
        let r = s.learn("what is two multiplied by three", "six", Some("times")).unwrap();
        assert!(r.surprised, "but 'multiplied by' is new");
        let lessons_after: usize = s.units.iter().map(|u| u.lessons.len()).sum();
        assert_eq!(lessons_after, lessons_before + 1, "only the surprising lesson was kept");
    }

    #[test]
    fn a_unit_that_stopped_following_one_rule_is_taken_apart() {
        let mut s = trained();
        // Six multiplication lessons are put under 'sum' by mistake.
        for (q, a) in [
            ("what is two times three", "six"), ("one times five", "five"), ("what is two times five", "ten"),
            ("three times three", "nine"), ("what is zero times four", "zero"), ("four times two", "eight"),
        ] {
            s.learn(q, a, Some("sum")).unwrap();
        }
        let sum = s.units.iter().find(|u| u.name == "sum").unwrap();
        assert!(sum.best_evidence() < 0.95, "pollution should show: {}", sum.best_evidence());
        let times_before = s.units.iter().find(|u| u.name == "times").unwrap().lessons.len();
        let report = s.reorganise();
        assert!(!report.split.is_empty(), "the multiplication lessons should have been taken out");
        let sum = s.units.iter().find(|u| u.name.starts_with("sum")).unwrap();
        assert!(sum.best_evidence() >= 0.95, "sum is itself again: {}", sum.best_evidence());
        let times = s.units.iter().find(|u| u.name == "times").unwrap();
        assert!(times.lessons.len() >= times_before + 6, "they went to the unit that builds the same expression");
        assert_eq!(s.ask("what is two times four").answer.as_deref(), Some("eight"));
        assert_eq!(s.ask("seven plus two").answer.as_deref(), Some("nine"));
    }

    #[test]
    fn units_taught_separately_that_follow_one_rule_become_one() {
        let mut s = trained();
        let units = s.units.len();
        for (a, b) in [("zero", "three"), ("one", "two"), ("two", "four"), ("three", "one"), ("one", "five"), ("four", "four"), ("two", "two")] {
            let n = |w: &str| crate::data::NUM.iter().position(|x| *x == w).unwrap();
            s.learn(&format!("what is {a} added to {b}"), crate::data::NUM[n(a) + n(b)], Some("added")).unwrap();
        }
        assert_eq!(s.units.len(), units + 1);
        let report = s.reorganise();
        assert!(report.merged.iter().any(|(_, gone)| gone == "added"), "'added' should have been folded into the rule family: {:?}", report.merged);
        assert!(s.units.len() <= units, "{} units after merging (had {units})", s.units.len());
        assert_eq!(s.ask("what is five added to three").answer.as_deref(), Some("eight"));
        assert_eq!(s.ask("nine minus four").answer.as_deref(), Some("five"));
        assert!(s.ask("three minus five").answer.is_none(), "merging must not make it answer a negative result");
    }

    #[test]
    fn saving_and_loading_changes_nothing() {
        let s = trained();
        let path = std::env::temp_dir().join(format!("baby_school_{}.school", std::process::id()));
        let path = path.to_str().unwrap().to_string();
        s.save(&path).unwrap();
        let back = School::load(&path).unwrap();
        for u in 0..s.units.len() {
            std::fs::remove_file(format!("{path}.unit{u}.model")).ok();
        }
        std::fs::remove_file(&path).ok();
        for q in [
            "what is seven times one", "nine minus four", "which is smaller ten or four", "how far apart are two and ten",
            "capital of peru", "what comes after eight", "what is double three", "what is the code for four", "capital of atlantis",
            "what color is the sky", "opposite of hot", "what does the cat say", "what is five plus four",
        ] {
            let (a, b) = (s.ask(q), back.ask(q));
            assert_eq!(a.answer, b.answer, "{q}");
            assert_eq!(a.how, b.how, "{q}");
        }
    }

    #[test]
    fn the_precomputed_table_routes_as_well_as_comparing_with_every_lesson() {
        let w = kinds::world(1);
        let s = School::train(&w, 1, true);
        let mut wrong = 0;
        for it in w.reworded.iter().chain(&w.new_items).chain(&w.beyond) {
            if s.route(&it.query, &mut Ledger::default()).0 != it.unit {
                wrong += 1;
            }
        }
        assert_eq!(wrong, 0, "misrouted {wrong}");
    }

    #[test]
    fn after_school_it_learns_a_new_phrasing_and_a_new_kind() {
        let mut s = trained();
        // Before: "multiplied by" is not a phrase it has met.
        assert!(s.ask("what is four multiplied by two").answer.is_none());
        // Told which unit it belongs to, a phrasing joins it. The unit still learns by program.
        for (q, a) in [("what is two multiplied by three", "six"), ("one multiplied by five", "five"), ("what is five multiplied by two", "ten")] {
            s.learn(q, a, Some("times")).unwrap();
        }
        let times = s.units.iter().find(|u| u.name == "times").unwrap();
        assert_eq!(times.chosen, Mech::Program, "still builds the rule");
        assert_eq!(s.ask("what is four multiplied by two").answer.as_deref(), Some("eight"));
        assert_eq!(s.ask("what is two times four").answer.as_deref(), Some("eight"), "what it knew before still works");

        // A new kind: it opens a unit, remembers until it has enough facts, then works out how to learn it.
        let first = s.learn("what is zero less than ten", "ten", Some("lessthan")).unwrap();
        assert!(first.opened && first.now == Mech::Remember);
        for (n, a) in [("one", "nine"), ("two", "eight"), ("three", "seven"), ("four", "six"), ("five", "five")] {
            s.learn(&format!("what is {n} less than ten"), a, Some("lessthan")).unwrap();
        }
        let last = s.learn("six less than ten", "four", Some("lessthan")).unwrap();
        assert_ne!(last.now, Mech::Remember, "with seven facts it should have found something that generalises: {:?}", last.now);
        assert_eq!(s.ask("what is eight less than ten").answer.as_deref(), Some("two"));
    }
}




