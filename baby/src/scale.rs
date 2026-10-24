// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DawsMacs
//! Does learning stay light when the world gets big?
//!
//! Everything before this was measured on 13 units and 792 lessons. This module generates a much larger synthetic world: hundreds
//! of units, vocabularies of thousands of words, phrasings that overlap across units, and noise, so the cost of teaching, of
//! learning one more lesson, of answering, and of tidying up can be measured as the world grows, along with whether the baby
//! still refuses what it should.
//!
//! The world is still synthetic and tidy in structure (that is a stated limit), but every axis the handover listed is a knob:
//!
//!   units            how many kinds of lesson (60% facts, then rules, selects and built expressions, cycling through nine semantics)
//!   entities         size of the shared vocabulary that fact units draw their subjects from (so words overlap across units)
//!   facts_per_unit   how many facts each fact unit holds
//!   noise            fraction of taught lessons whose answer is wrong (label noise)
//!   typos            fraction of test questions with one word misspelled
//!
//! Every unit gets its own operator or attribute word (so units are told apart by a word, as "plus" and "minus" are), and phrasings
//! are drawn from six shared styles built from a small pool of filler words, so the overlap between units is heavy.

use crate::data::NUM;
use crate::kinds::{Item, World};
use crate::ledger::{EnergyModel, Ledger};
use crate::platform;
use crate::rng::Rng;
use crate::school::School;
use std::collections::HashMap;
use std::time::Instant;

#[derive(Clone, Debug)]
pub struct Config {
    pub units: usize,
    pub entities: usize,
    pub facts_per_unit: usize,
    pub noise: f32,
    pub typos: f32,
    /// Numeric units use digits: operands 0-30 (taught up to 12), answers up to 99, instead of the words zero to ten.
    pub digits: bool,
    pub seed: u64,
}

pub struct ScaleWorld {
    pub names: Vec<String>,
    /// The mechanism each unit should end up with: facts are remembered, sums and the like are rules, and so on.
    pub expect: Vec<&'static str>,
    /// The operation of each numeric unit (None for a unit of facts): enough to answer any question about it, which a simulated teacher needs.
    pub sems: Vec<Option<Sem>>,
    pub world: World,
    /// Queries whose taught answer was deliberately made wrong, with the answer they should have had.
    pub corrupted: HashMap<String, String>,
    /// Misspelled rewordings and the answer they should have had.
    pub misspelled: Vec<(String, String)>,
    /// Questions that must never be answered: (query, why it is a trap).
    pub must_refuse: Vec<(String, &'static str)>,
}

const CONS: &[u8] = b"bdfgklmnprstvz";
const VOW: &[u8] = b"aeiou";

/// A pronounceable made-up word of `syllables` consonant-vowel pairs. Three syllables (6 letters) for entities and answers, four
/// for operator and attribute words, so none can collide with a number word ("zero", "nine") or a filler word.
pub(crate) fn word(mut i: usize, syllables: usize) -> String {
    let mut s = String::new();
    for _ in 0..syllables {
        let k = i % (CONS.len() * VOW.len());
        i /= CONS.len() * VOW.len();
        s.push(CONS[k / VOW.len()] as char);
        s.push(VOW[k % VOW.len()] as char);
    }
    s
}

/// Six phrasing styles, each four templates: three are taught, the fourth is a rewording made only of words the others use.
const FACT_STYLES: [[&str; 4]; 6] = [
    ["what is the {A} of {0}", "{A} of {0}", "the {A} of {0}", "what is {0} {A}"],
    ["what {A} for {0}", "{A} for {0}", "for {0} the {A}", "for {0} what {A}"],
    ["{0} {A}", "{A} {0}", "what is {0} {A}", "what {A} {0}"],
    ["name the {A} of {0}", "the {A} of {0}", "{A} {0}", "name {0} {A}"],
    ["how is {0} {A}", "{0} is {A} how", "{A} {0}", "is {0} {A} how"],
    ["give {A} for {0}", "{A} for {0}", "for {0} give {A}", "give {0} {A}"],
];
pub(crate) const BINARY: [&str; 4] = ["what is {0} {A} {1}", "{0} {A} {1}", "{0} {A} {1} equals", "what is {0} {A} {1} equals"];
pub(crate) const UNARY: [&str; 4] = ["what is {A} {0}", "{A} {0}", "{0} {A}", "what is {0} {A}"];
const SELECT: [&str; 4] = ["which is {A} {0} or {1}", "{A} {0} or {1}", "{0} or {1} which is {A}", "which {A} {0} or {1}"];

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Sem {
    Sum,
    Minus,
    After,
    Double,
    Half,
    Max,
    Min,
    Times,
    Apart,
}
const SEMS: [Sem; 9] = [Sem::Sum, Sem::Minus, Sem::After, Sem::Double, Sem::Half, Sem::Max, Sem::Min, Sem::Times, Sem::Apart];

struct Fact {
    slots: Vec<String>,
    answer: String,
    templates: [String; 4],
}

pub(crate) fn fill(template: &str, attr: &str, slots: &[String]) -> String {
    let mut s = template.replace("{A}", attr);
    for (i, slot) in slots.iter().enumerate() {
        s = s.replace(&format!("{{{i}}}"), slot);
    }
    s
}

pub(crate) fn templ(t: [&str; 4], attr: &str) -> [String; 4] {
    [t[0].replace("{A}", attr), t[1].replace("{A}", attr), t[2].replace("{A}", attr), t[3].replace("{A}", attr)]
}

/// (taught-range facts, beyond-range facts) for a numeric unit, mirroring `kinds.rs`: the taught range is small numbers. In words that is
/// up to five of zero to ten; in digits up to twelve of 0 to 30 (and answers stay under 100), sampled so a unit has a few dozen facts.
fn numeric_facts(sem: Sem, attr: &str, digits: bool, rng: &mut Rng) -> (Vec<Fact>, Vec<Fact>) {
    let (max, near_to, ans_max) = if digits { (30usize, 12usize, 99usize) } else { (10, 5, 10) };
    let n = |i: usize| if digits { i.to_string() } else { NUM[i].to_string() };
    let mut out: (Vec<Fact>, Vec<Fact>) = (Vec::new(), Vec::new());
    fn put(out: &mut (Vec<Fact>, Vec<Fact>), is_near: bool, f: Fact) {
        if is_near { out.0.push(f) } else { out.1.push(f) }
    }
    match sem {
        Sem::Sum | Sem::Times | Sem::Apart | Sem::Minus => {
            for a in 0..=max {
                for b in 0..=max {
                    let ans = match sem {
                        Sem::Sum if a + b <= ans_max => a + b,
                        Sem::Times if a * b <= ans_max => a * b,
                        Sem::Apart => a.abs_diff(b),
                        Sem::Minus if b <= a => a - b,
                        _ => continue,
                    };
                    let is_near = if sem == Sem::Minus { a <= near_to } else { a <= near_to && b <= near_to };
                    put(&mut out, is_near, Fact { slots: vec![n(a), n(b)], answer: n(ans), templates: templ(BINARY, attr) });
                }
            }
        }
        Sem::Max | Sem::Min => {
            for a in 0..=max {
                for b in 0..=max {
                    if a != b {
                        let ans = if sem == Sem::Max { a.max(b) } else { a.min(b) };
                        put(&mut out, a <= near_to && b <= near_to, Fact { slots: vec![n(a), n(b)], answer: n(ans), templates: templ(SELECT, attr) });
                    }
                }
            }
        }
        Sem::After => {
            for a in 0..max {
                put(&mut out, a <= near_to + 1, Fact { slots: vec![n(a)], answer: n(a + 1), templates: templ(UNARY, attr) });
            }
        }
        Sem::Double => {
            for a in 0..=(ans_max / 2).min(max) {
                put(&mut out, a <= near_to, Fact { slots: vec![n(a)], answer: n(2 * a), templates: templ(UNARY, attr) });
            }
        }
        Sem::Half => {
            for a in 0..=(ans_max / 2).min(max) {
                put(&mut out, a <= near_to, Fact { slots: vec![n(2 * a)], answer: n(a), templates: templ(UNARY, attr) });
            }
        }
    }
    if digits {
        rng.shuffle(&mut out.0);
        rng.shuffle(&mut out.1);
        out.0.truncate(36);
        out.1.truncate(30);
    }
    out
}

/// What a teacher who knows the operation would answer to a question about it, in the same terms the generator uses: `None` where the
/// generator defines no fact (an answer with no number word, a negative result, operands out of the number line). `digits` as in `Config`.
pub fn teacher_truth(sem: Sem, nums: &[usize], digits: bool) -> Option<usize> {
    let (max, ans_max) = if digits { (30usize, 99usize) } else { (10, 10) };
    // the operands of every operation stop at `max`, except "half of": its input is twice a number that stops at `max` and at half the answers
    let limit = if sem == Sem::Half { 2 * (ans_max / 2).min(max) } else { max };
    if nums.iter().any(|&x| x > limit) {
        return None;
    }
    let ans = match (sem, nums) {
        (Sem::Sum, [a, b]) => a + b,
        (Sem::Times, [a, b]) => a * b,
        (Sem::Apart, [a, b]) => a.abs_diff(*b),
        (Sem::Minus, [a, b]) if b <= a => a - b,
        (Sem::Max, [a, b]) if a != b => *a.max(b),
        (Sem::Min, [a, b]) if a != b => *a.min(b),
        (Sem::After, [a]) if *a < max => a + 1,
        (Sem::Double, [a]) => 2 * a,
        (Sem::Half, [a]) if a % 2 == 0 => a / 2,
        _ => return None,
    };
    (ans <= ans_max).then_some(ans)
}

/// Misspell one word of a question by dropping a letter from its longest word that is not a number word.
fn misspell(q: &str, rng: &mut Rng) -> String {
    let mut words: Vec<String> = q.split_whitespace().map(|w| w.to_string()).collect();
    let cands: Vec<usize> = (0..words.len()).filter(|&i| words[i].len() >= 6 && !crate::quantity::is_number(&words[i])).collect();
    if let Some(&i) = cands.get(rng.below(cands.len().max(1))) {
        let at = rng.below(words[i].len());
        words[i].remove(at);
    }
    words.join(" ")
}

pub fn generate(c: &Config) -> ScaleWorld {
    let mut rng = Rng::new(c.seed.wrapping_add(4242));
    assert!(c.entities + c.facts_per_unit < 150_000, "entity vocabulary is limited to 150k words");
    let answer_base = 200_000usize; // answers and entities live in disjoint index ranges
    let mut w = World { train: Vec::new(), reworded: Vec::new(), new_items: Vec::new(), beyond: Vec::new() };
    let mut names = Vec::new();
    let mut expect: Vec<&'static str> = Vec::new();
    let mut fact_id = 0usize;
    // (unit index, attribute word, entities the unit holds) for the traps below.
    let mut fact_units: Vec<(usize, String, Vec<usize>)> = Vec::new();
    let mut numeric_units: Vec<(usize, String, Sem)> = Vec::new();
    let mut sems: Vec<Option<Sem>> = Vec::new();
    let mut corrupted: HashMap<String, String> = HashMap::new();

    for u in 0..c.units {
        let attr = word(1_000_000 + u * 7 + 1, 4);
        names.push(format!("u{u}-{attr}"));
        let numeric = u % 5 >= 3; // 40% numeric (rules, selects, built expressions), 60% facts
        let (near, far): (Vec<Fact>, Vec<Fact>) = if numeric {
            let sem = SEMS[(u / 5 * 2 + (u % 5 - 3)) % SEMS.len()];
            numeric_units.push((u, attr.clone(), sem));
            sems.push(Some(sem));
            expect.push(match sem {
                Sem::Sum | Sem::Minus | Sem::After | Sem::Double | Sem::Half => "rule",
                Sem::Max | Sem::Min => "select",
                Sem::Times | Sem::Apart => "program",
            });
            numeric_facts(sem, &attr, c.digits, &mut rng)
        } else {
            let style = FACT_STYLES[rng.below(FACT_STYLES.len())];
            let mut ents = Vec::new();
            while ents.len() < c.facts_per_unit.min(c.entities) {
                let e = rng.below(c.entities);
                if !ents.contains(&e) {
                    ents.push(e);
                }
            }
            // each unit has its own arbitrary mapping from subject to answer, drawn from a pool the size of the entity vocabulary / 4
            let pool = (c.entities / 4).max(8);
            let facts = ents
                .iter()
                .map(|&e| Fact { slots: vec![word(e, 3)], answer: word(answer_base + rng.below(pool), 3), templates: templ(style, &attr) })
                .collect();
            expect.push("remember");
            sems.push(None);
            fact_units.push((u, attr.clone(), ents));
            (facts, Vec::new())
        };
        let mut near = near;
        rng.shuffle(&mut near);
        let held = if numeric { ((near.len() as f32) * 0.2).round().max(1.0) as usize } else { 0 };
        for (i, f) in near.iter().enumerate() {
            for (t, template) in f.templates.iter().enumerate() {
                let query = fill(template, "", &f.slots);
                let mut item = Item { unit: u, fact: fact_id, template: t, query, answer: f.answer.clone() };
                if i < held {
                    w.new_items.push(item);
                } else if t < 3 {
                    if rng.uniform() < c.noise {
                        item.answer = if numeric {
                            match f.answer.parse::<usize>() {
                                Ok(v) => (v + 1 + rng.below(9)).to_string(),
                                Err(_) => NUM[(NUM.iter().position(|x| *x == f.answer).unwrap() + 1 + rng.below(10)) % 11].to_string(),
                            }
                        } else {
                            word(answer_base + rng.below((c.entities / 4).max(8)), 3)
                        };
                        if item.answer != f.answer {
                            corrupted.insert(item.query.clone(), f.answer.clone());
                        }
                    }
                    w.train.push(item);
                } else {
                    w.reworded.push(item);
                }
            }
            fact_id += 1;
        }
        for f in &far {
            for (t, template) in f.templates.iter().enumerate() {
                w.beyond.push(Item { unit: u, fact: fact_id, template: t, query: fill(template, "", &f.slots), answer: f.answer.clone() });
            }
            fact_id += 1;
        }
    }

    // Misspelled rewordings: one word missing a letter. The truth is the unmisspelled answer; refusing is fine, answering wrongly is not.
    let mut misspelled = Vec::new();
    for it in &w.reworded {
        if rng.uniform() < c.typos {
            misspelled.push((misspell(&it.query, &mut rng), it.answer.clone()));
        }
    }

    // Traps: things the baby has no business answering.
    let mut must_refuse: Vec<(String, &'static str)> = Vec::new();
    let stride = (fact_units.len() / 150).max(1);
    for (k, (_, attr, ents)) in fact_units.iter().enumerate() {
        if k % stride != 0 {
            continue;
        }
        // a subject that other units know but this one was never told about
        let other = &fact_units[(k + 1 + rng.below(fact_units.len().max(2) - 1)) % fact_units.len()].2;
        if let Some(&e) = other.iter().find(|e| !ents.contains(e)) {
            must_refuse.push((fill("what is the {A} of {0}", attr, &[word(e, 3)]), "known word, wrong unit: never told this fact"));
        }
        // a subject nobody has ever heard of
        must_refuse.push((fill("what is the {A} of {0}", attr, &[word(c.entities + 1 + k, 3)]), "unmet word"));
        // nonsense made of familiar words
        // a DIFFERENT unit's attribute (the same unit's would be a legitimate question with a stray word)
        let (_, other_attr, other_ents) = &fact_units[(k + 1 + rng.below(fact_units.len().max(2) - 1)) % fact_units.len()];
        must_refuse.push((fill("what is the {A} of {0}", other_attr, &[word(other_ents[0], 3)]) + " " + attr, "two attributes at once"));
    }
    for (_, attr, sem) in &numeric_units {
        let num = |words: &str, digits: &str| -> String { if c.digits { digits.to_string() } else { words.to_string() } };
        match sem {
            // in words a result past "ten" has no word, so it must be refused; in digits it is simply a bigger number
            Sem::Sum if !c.digits => must_refuse.push((fill("what is {0} {A} {1}", attr, &["nine".into(), "nine".into()]), "result beyond the number line")),
            Sem::Minus => must_refuse.push((fill("what is {0} {A} {1}", attr, &[num("two", "2"), num("five", "5")]), "negative result")),
            Sem::Times if !c.digits => must_refuse.push((fill("what is {0} {A} {1}", attr, &["six".into(), "seven".into()]), "result beyond the number line")),
            _ => {}
        }
        must_refuse.push((fill("what is {A} {0} {1} {2}", attr, &[num("two", "2"), num("three", "3"), num("four", "4")]), "too many operands"));
    }
    ScaleWorld { names, expect, sems, world: w, corrupted, misspelled, must_refuse }
}

// ---------------------------------------------------------------------------------------------------------------------------
// The measurements.

#[derive(Default, Clone, Copy)]
pub struct Tally {
    pub right: usize,
    pub refused: usize,
    pub wrong: usize,
}

impl Tally {
    fn add(&mut self, truth: &str, got: &Option<String>) {
        match got {
            None => self.refused += 1,
            Some(a) if a == truth => self.right += 1,
            Some(_) => self.wrong += 1,
        }
    }
    fn n(&self) -> usize {
        self.right + self.refused + self.wrong
    }
    fn pct(&self, x: usize) -> f64 {
        100.0 * x as f64 / self.n().max(1) as f64
    }
    fn show(&self) -> String {
        format!("right {:5.1}%  refused {:5.1}%  WRONG {:5.2}%  (n={})", self.pct(self.right), self.pct(self.refused), self.pct(self.wrong), self.n())
    }
}

/// Every k-th item so that at most `cap` are graded (grading is the slow part at scale, and every unit still appears).
fn sample<T>(v: &[T], cap: usize) -> impl Iterator<Item = &T> {
    let stride = (v.len() / cap.max(1)).max(1);
    v.iter().step_by(stride)
}

fn grade(school: &School, items: &[Item], cap: usize) -> (Tally, f64) {
    let mut t = Tally::default();
    let mut routed = 0usize;
    let mut shown = 0;
    let mut shown_ref = 0;
    for it in sample(items, cap) {
        let r = school.ask(&it.query);
        if r.answer.is_some() && r.answer.as_deref() != Some(it.answer.as_str()) && shown < 3 && std::env::var("SCALE_WRONG").is_ok() {
            shown += 1;
            println!("      WRONG: {:?} should be {:?}, got {:?}: {}", it.query, it.answer, r.answer, r.how);
        }
        if r.answer.is_none() && shown_ref < 4 && std::env::var("SCALE_REFUSED").is_ok() {
            shown_ref += 1;
            println!("      REFUSED: {:?} (expected {:?}): {}", it.query, it.answer, r.how);
            if let Some(u) = r.unit {
                let w = it.query.split_whitespace().find(|w| w.len() > 6 && !crate::quantity::is_number(w)).unwrap_or("");
                println!("         {}: {}", school.units[u].name, school.units[u].describe_word(w));
            }
        }
        t.add(&it.answer, &r.answer);
        routed += (r.unit == Some(it.unit)) as usize;
    }
    (t, 100.0 * routed as f64 / t.n().max(1) as f64)
}

/// One size, measured. Prints a report; returns the numbers for the sweep table.
pub fn run(c: &Config, reorg: bool, grade_cap: usize) -> Vec<(&'static str, f64)> {
    let mut row: Vec<(&'static str, f64)> = Vec::new();
    let t = Instant::now();
    let sw = generate(c);
    let gen_s = t.elapsed().as_secs_f64();
    let lessons = sw.world.train.len();
    let baseline_mb = platform::resident_mb();
    println!("== {} units, {} lessons, vocabulary of {} subjects, noise {:.0}%, typos {:.0}%, seed {} (world built in {:.2} s)", c.units, lessons, c.entities, 100.0 * c.noise, 100.0 * c.typos, c.seed, gen_s);

    // Teaching from nothing.
    let t = Instant::now();
    let mut school = School::from_items(&sw.world.train, sw.names.clone(), Some(&sw.world.reworded), c.seed, true);
    let teach_s = t.elapsed().as_secs_f64();
    let (rss, peak) = (platform::resident_mb(), platform::peak_mb());
    let mut by_mech: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
    let mut agree = 0usize;
    for (u, unit) in school.units.iter().enumerate() {
        *by_mech.entry(unit.chosen.name()).or_default() += 1;
        agree += (unit.chosen.name() == sw.expect[u]) as usize;
    }
    println!(
        "  teaching from nothing: {:.2} s ({:.2} ms per lesson); memory {} resident, {} peak (generator + test data alone: {})",
        teach_s, 1000.0 * teach_s / lessons as f64, platform::fmt_mb(rss, 0), platform::fmt_mb(peak, 0), platform::fmt_mb(baseline_mb, 0)
    );
    println!("  reliability bar it chose: {:.2} (it measures {:.1}% of its lessons as wrong; the world has {:.1}%)", school.bar(), 100.0 * crate::mechanisms::estimate_noise(&(0..sw.names.len()).map(|u| sw.world.train.iter().filter(|i| i.unit == u).cloned().collect()).collect::<Vec<_>>()), 100.0 * sw.corrupted.len() as f64 / lessons.max(1) as f64);
    println!("  chose {:?}; {:.1}% of units got the mechanism they should (facts: remember, sums: rule, ...)", by_mech, 100.0 * agree as f64 / c.units as f64);
    for (u, unit) in school.units.iter().enumerate().filter(|(u, unit)| unit.chosen.name() != sw.expect[*u]).take(6) {
        println!(
            "      unit {} should {} but chose {}: held-back evidence [remember {:.2}, rule {:.2}, select {:.2}, program {:.2}, network {:.2}], {} facts",
            unit.name, sw.expect[u], unit.chosen.name(), unit.evidence[0], unit.evidence[1], unit.evidence[2], unit.evidence[3], unit.evidence[4], unit.facts()
        );
    }
    row.extend([("units", c.units as f64), ("lessons", lessons as f64), ("vocab", school.known_words().len() as f64), ("teach_s", teach_s), ("rss_mb", rss), ("peak_mb", peak), ("agree", 100.0 * agree as f64 / c.units as f64)].into_iter().filter(|(_, v)| !v.is_nan()));

    // The scorecard, WRONG apart from refused.
    println!("  scorecard (up to {grade_cap} questions each):");
    let (mut taught_clean, mut taught_noisy) = (Tally::default(), Tally::default());
    let mut routed_taught = 0usize;
    let mut repeated_wrong = 0usize;
    for it in sample(&sw.world.train, grade_cap) {
        let r = school.ask(&it.query);
        match sw.corrupted.get(&it.query) {
            // graded against the TRUTH, not against what it was told: right = it was not fooled, repeated = it said the wrong lesson back
            Some(truth) => {
                taught_noisy.add(truth, &r.answer);
                repeated_wrong += (r.answer.as_deref() == Some(it.answer.as_str())) as usize;
            }
            None => taught_clean.add(&it.answer, &r.answer),
        }
        routed_taught += (r.unit == Some(it.unit)) as usize;
    }
    let (reworded, route_re) = grade(&school, &sw.world.reworded, grade_cap);
    // Why were rewordings refused? (a refusal is acceptable, but it is a miss, and the reason says where)
    let mut why: std::collections::BTreeMap<String, (usize, String, String)> = std::collections::BTreeMap::new();
    for it in sample(&sw.world.reworded, grade_cap) {
        let r = school.ask(&it.query);
        if r.answer.is_none() {
            let kind = format!("{} / {}", sw.expect[it.unit], r.how.chars().take(70).collect::<String>());
            let e = why.entry(kind).or_insert((0, it.query.clone(), sw.names[it.unit].clone()));
            e.0 += 1;
        }
    }
    let mut why: Vec<_> = why.into_iter().collect();
    why.sort_by(|a, b| b.1 .0.cmp(&a.1 .0));
    for (kind, (n, q, u)) in why.iter().take(4) {
        println!("      refused {n:3}x  [{kind}]  e.g. {q:?} in {u}");
    }
    let (never, _) = grade(&school, &sw.world.new_items, grade_cap);
    let (beyond, _) = grade(&school, &sw.world.beyond, grade_cap);
    let mut typo = Tally::default();
    for (q, truth) in sample(&sw.misspelled, grade_cap) {
        typo.add(truth, &school.ask(q).answer);
    }
    println!("    taught                {}", taught_clean.show());
    if taught_noisy.n() > 0 {
        println!("    taught, but told wrong  {}   (graded against the truth; it repeated the wrong lesson {} times)", taught_noisy.show(), repeated_wrong);
    }
    println!("    reworded              {}   routed to the right unit {:.1}%", reworded.show(), route_re);
    println!("    never taught          {}", never.show());
    println!("    beyond the taught range {}", beyond.show());
    if typo.n() > 0 {
        println!("    one word misspelled     {}", typo.show());
    }
    let mut trap_by: std::collections::BTreeMap<&str, (usize, usize)> = std::collections::BTreeMap::new();
    for (q, why) in &sw.must_refuse {
        let e = trap_by.entry(why).or_default();
        e.0 += 1;
        let r = school.ask(q);
        e.1 += r.answer.is_some() as usize;
        if r.answer.is_some() {
            println!("      ANSWERED a trap ({why}): {q:?} -> {:?}, {}", r.answer, r.how);
        }
    }
    let (traps, answered): (usize, usize) = trap_by.values().fold((0, 0), |a, b| (a.0 + b.0, a.1 + b.1));
    println!("    must-refuse traps     {} asked, {} ANSWERED", traps, answered);
    for (why, (n, a)) in &trap_by {
        println!("        {:<46} {:4} asked, {:3} answered", why, n, a);
    }
    row.extend([
        ("taught_ok", taught_clean.pct(taught_clean.right)), ("taught_wrong", taught_clean.pct(taught_clean.wrong)), ("noisy_wrong", taught_noisy.pct(taught_noisy.wrong)),
        ("reworded_ok", reworded.pct(reworded.right)), ("reworded_wrong", reworded.pct(reworded.wrong)), ("route", route_re),
        ("never_ok", never.pct(never.right)), ("never_wrong", never.pct(never.wrong)), ("beyond_ok", beyond.pct(beyond.right)), ("beyond_wrong", beyond.pct(beyond.wrong)),
        ("typo_wrong", typo.pct(typo.wrong)), ("traps_answered", answered as f64),
    ]);
    let _ = routed_taught;

    // Answering: time and counted effort.
    let mut qs: Vec<&str> = sample(&sw.world.reworded, 1500).map(|i| i.query.as_str()).collect();
    qs.extend(sw.must_refuse.iter().map(|(q, _)| q.as_str()).take(500));
    let mut ledger = Ledger::default();
    let t = Instant::now();
    for q in &qs {
        let _ = school.ask_with(q, &mut ledger, None);
    }
    let per_q_us = 1e6 * t.elapsed().as_secs_f64() / qs.len() as f64;
    let ops = (ledger.macs + ledger.other_ops + ledger.int_ops + ledger.words_read + ledger.words_written) as f64 / qs.len() as f64;
    let uj = 1e6 * EnergyModel::default().joules(&ledger) / qs.len() as f64;
    println!("  answering: {:.0} us per question, {:.0} counted operations, about {:.2} uJ estimated", per_q_us, ops, uj);
    row.extend([("ask_us", per_q_us), ("ask_ops", ops), ("ask_uj", uj)]);

    // Learning one more lesson at this size.
    let mut rng = Rng::new(c.seed.wrapping_add(99));
    let numeric_new: Vec<&Item> = sw.world.new_items.iter().step_by((sw.world.new_items.len() / 30).max(1)).take(30).collect();
    let t = Instant::now();
    let mut skipped = 0;
    for it in &numeric_new {
        if let Ok(r) = school.learn(&it.query, &it.answer, Some(&sw.names[it.unit])) {
            skipped += (!r.surprised) as usize;
        }
    }
    let known_ms = 1000.0 * t.elapsed().as_secs_f64() / numeric_new.len().max(1) as f64;
    println!("  a lesson in a unit that already works it out: {:.2} ms each ({} of {} needed no learning)", known_ms, skipped, numeric_new.len());

    let fact_units: Vec<usize> = (0..c.units).filter(|&u| sw.expect[u] == "remember").collect();
    let mut times = Vec::new();
    for j in 0..30usize {
        let u = fact_units[rng.below(fact_units.len())];
        let attr = sw.names[u].split('-').nth(1).unwrap().to_string();
        let q = format!("what is the {attr} of {}", word(c.entities + 5000 + j, 3));
        let t = Instant::now();
        let _ = school.learn(&q, &word(200_000 + rng.below(100), 3), Some(&sw.names[u]));
        times.push(1000.0 * t.elapsed().as_secs_f64());
    }
    let new_fact_ms = times.iter().sum::<f64>() / times.len() as f64;
    println!("  a brand-new fact (new subject) in an existing unit: {:.2} ms each (first {:.2}, last {:.2})", new_fact_ms, times[0], times[times.len() - 1]);

    let t = Instant::now();
    let retaught: Vec<&Item> = sample(&sw.world.train, 30).take(30).collect();
    for it in &retaught {
        let _ = school.learn(&it.query, &it.answer, Some(&sw.names[it.unit]));
    }
    let retaught_ms = 1000.0 * t.elapsed().as_secs_f64() / retaught.len().max(1) as f64;
    println!("  the same lesson taught again (same answer): {:.2} ms each", retaught_ms);

    let t = Instant::now();
    let mut opened = false;
    let mut first_ms = 0.0;
    for (k, n) in (0..=6usize).enumerate() {
        let t1 = Instant::now();
        let _ = school.learn_anywhere(&format!("what is {} less than ten", NUM[n]), NUM[10 - n]);
        if k == 0 {
            first_ms = 1000.0 * t1.elapsed().as_secs_f64();
        }
    }
    if school.units.last().map_or(false, |u| u.name.contains("less")) || school.pen.is_empty() {
        opened = true;
    }
    println!("  7 lessons of a kind it has never seen, no unit named: {:.1} ms in all (first {:.2} ms); a unit {}", 1000.0 * t.elapsed().as_secs_f64(), first_ms, if opened { "opened" } else { "did not open" });
    row.extend([("known_ms", known_ms), ("newfact_ms", new_fact_ms), ("retaught_ms", retaught_ms), ("newkind_ms", 1000.0 * t.elapsed().as_secs_f64() / 7.0)]);

    // Tidying up.
    if reorg {
        let t = Instant::now();
        let r = school.reorganize();
        let secs = t.elapsed().as_secs_f64();
        println!("  reorganizing: {:.2} s, {} merges, {} splits; {} units now", secs, r.merged.len(), r.split.len(), school.units.len());
        let (re2, _) = grade(&school, &sw.world.reworded, 1000);
        let (nv2, _) = grade(&school, &sw.world.new_items, 1000);
        println!("    after it: reworded {}   never taught {}", re2.show(), nv2.show());
        row.extend([("reorg_s", secs), ("reorg_merges", r.merged.len() as f64), ("reorg_wrong", re2.pct(re2.wrong) + nv2.pct(nv2.wrong))]);
    }

    // Keeping it.
    let path = format!("runs/scale-{}.school", c.units);
    let t = Instant::now();
    if school.save(&path).is_ok() {
        let save_s = t.elapsed().as_secs_f64();
        let kb = std::fs::metadata(&path).map(|m| m.len() as f64 / 1024.0).unwrap_or(0.0);
        let t = Instant::now();
        let loaded = School::load(&path).is_ok();
        println!("  saved: {:.0} KB in {:.2} s ({:.1} bytes per lesson); loaded back {} in {:.2} s", kb, save_s, 1024.0 * kb / lessons as f64, if loaded { "fine" } else { "WITH ERRORS" }, t.elapsed().as_secs_f64());
        row.extend([("file_kb", kb), ("save_s", save_s), ("load_s", t.elapsed().as_secs_f64())]);
        let _ = std::fs::remove_file(&path);
    }
    row
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn cfg(units: usize) -> Config {
        Config { units, entities: 400, facts_per_unit: 10, noise: 0.0, typos: 0.1, digits: false, seed: 1 }
    }

    #[test]
    fn the_generated_world_is_consistent_and_does_not_leak() {
        let sw = generate(&cfg(30));
        assert_eq!(sw.names.len(), 30);
        let train: HashSet<&str> = sw.world.train.iter().map(|i| i.query.as_str()).collect();
        for i in sw.world.reworded.iter().chain(&sw.world.new_items).chain(&sw.world.beyond) {
            assert!(!train.contains(i.query.as_str()), "leaked: {}", i.query);
        }
        assert!(sw.world.train.iter().all(|i| i.unit < 30));
        assert!(sw.must_refuse.len() > 30, "traps were generated: {}", sw.must_refuse.len());
    }

    #[test]
    fn a_rewording_uses_only_words_its_unit_was_taught() {
        // The baby refuses a question with a word its unit never saw (that guard closed the "code for five" bluff), so a rewording
        // that smuggles one in measures the guard, not rewording. An early version of this generator did exactly that.
        let sw = generate(&cfg(40));
        for u in 0..40 {
            let seen: HashSet<String> = sw.world.train.iter().filter(|i| i.unit == u).flat_map(|i| crate::data::tokenize(&i.query)).collect();
            for it in sw.world.reworded.iter().filter(|i| i.unit == u) {
                for t in crate::data::tokenize(&it.query) {
                    assert!(seen.contains(&t), "unit {u}: reworded {:?} uses {t:?}, never taught", it.query);
                }
            }
        }
    }

    #[test]
    fn made_up_words_never_collide_with_number_words() {
        for i in (0..200_000).step_by(7) {
            assert!(!NUM.contains(&word(i, 3).as_str()));
            assert!(!NUM.contains(&word(1_000_000 + i, 4).as_str()));
        }
    }

    #[test]
    fn noise_corrupts_about_the_requested_fraction_and_only_taught_lessons() {
        let mut c = cfg(60);
        c.noise = 0.2;
        let sw = generate(&c);
        let frac = sw.corrupted.len() as f32 / sw.world.train.len() as f32;
        assert!((0.12..0.28).contains(&frac), "{frac}");
        assert!(sw.world.reworded.iter().all(|i| !sw.corrupted.contains_key(&i.query)));
    }

    #[test]
    fn the_simulated_teacher_agrees_with_the_generator_on_every_question_the_generator_makes() {
        // the teacher answers a check question from the unit's operation alone; it must reproduce every taught, reworded, never-taught and
        // beyond-range answer the generator itself wrote, in words and in digits
        for digits in [false, true] {
            let mut c = cfg(40);
            c.digits = digits;
            let sw = generate(&c);
            assert_eq!(sw.sems.len(), sw.names.len());
            let mut checked = 0;
            for it in sw.world.train.iter().chain(&sw.world.reworded).chain(&sw.world.new_items).chain(&sw.world.beyond) {
                let Some(sem) = sw.sems[it.unit] else { continue };
                let nums: Vec<usize> = crate::data::tokenize(&it.query)
                    .iter()
                    .filter_map(|t| NUM.iter().position(|n| n == t).or_else(|| t.parse::<usize>().ok().filter(|_| digits)))
                    .collect();
                let truth = teacher_truth(sem, &nums, digits).map(|a| if digits { a.to_string() } else { NUM[a].to_string() });
                assert_eq!(truth.as_deref(), Some(it.answer.as_str()), "{:?} ({sem:?}, digits {digits})", it.query);
                checked += 1;
            }
            assert!(checked > 200, "{checked}");
        }
        // and it declines what has no answer in words
        assert_eq!(teacher_truth(Sem::Sum, &[6, 7], false), None, "13 has no number word");
        assert_eq!(teacher_truth(Sem::Minus, &[2, 5], false), None, "negative");
        assert_eq!(teacher_truth(Sem::Max, &[3, 3], false), None, "a tie has no larger");
        assert_eq!(teacher_truth(Sem::Half, &[3], false), None, "odd");
    }
}
