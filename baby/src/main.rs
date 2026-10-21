// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DawsMacs
//! baby: Transient's first tiny network. It reads a short query and says one word.
//!
//!   baby school [--seed N] [--out PATH]     TEACH THE BABY (the unified one): units of lessons, each learned the way evidence says
//!   baby learn  "what is two multiplied by three" six --unit times    keep learning AFTER school: add a lesson, re-work how that unit is learned
//!   baby discover [--seed N] [--out PATH]   hand it a heap of UNLABELLED lessons: it finds the units itself, then is scored like the others
//!   baby cost                               what LEARNING costs: time and memory to teach it, a new lesson, a new kind, and to tidy up
//!   baby scale  [--units N --entities N --facts N --noise F --typos F --seed N --grade N --no-reorg | --sweep 20,60,200]   does learning stay light as the world grows?
//!   baby invent [--seed N]                  hidden kinds nobody wrote a mechanism for, and kinds that must stay unexplained: what does it work out?
//!   baby wonder [--check] [--school PATH]   what would it like to be told? (--check: where would it test its own explanations?) Questions whose answers would settle which of several explanations is meant
//!                                           (answer with: baby learn "<question>" <answer> --unit NAME --wondered)
//!   baby sleep  [--compact]                 re-sort the pen, merge/split units that follow one rule, optionally forget what is explained
//!   baby effort                             what answering costs (operations, words moved, estimated joules): the slow way vs the baby's way
//!   baby ask    "what is two times four"    ask it; it answers, says how it knew, or says why it won't (uses the school)
//!                                           (careful is the default: it will not state as fact a number beyond what it was taught or has had checked;
//!                                            --permissive turns that off and lets rules extrapolate unchecked)
//!   baby data   [--seed N] [--tsv]          show the older single-network world it learns from
//!   baby train  [options]                   teach it (from scratch), then save it
//!   baby ask    "capital of france"         ask a saved baby (frozen: it does not learn here)
//!   baby eval   [--model PATH]              score a saved baby on the held-out sets
//!   baby teach  "capital of atlantis" poseidonis    tell it something new; it remembers instantly (no weight change)
//!   baby know   [--seeds K]                 does it know what it doesn't know? three signals compared
//!   baby rules  [--seeds K]                 rules, not tables: a quantity unit vs memorised sums, and the baby choosing between them
//!   baby kinds  [--seeds K]                 five units of lessons: the baby picks how to learn each one, from evidence
//!   baby curious [--seeds K]                a baby that chooses what to ask the teacher, versus being drip-fed
//!   baby life   [options]                   LEARN instead of train: lessons one at a time, five ways of coping, charged in joules
//!   baby energy [--model PATH]              what one question and one learning step cost, counted and (if allowed) measured
//!
//! train options: --seed N --epochs N --dim N --hidden N --batch N --lr F --wd F --curriculum --out PATH --quiet
//!                --reader concat|bag|bigram|letters --buckets N --typo-aug K
//! life options:  --strategies naive,replay,store,sleep,retrain --seeds K --seed N --days N --verbose
//!                --max-steps N --replay-k N --sleep-every N --sleep-epochs N --threshold F --lr F --optimizer adam|sgd --reader concat|bag|bigram|letters --buckets N --typo-aug K

mod curious;
mod data;
mod discover;
mod invent;
mod kinds;
mod know;
mod ledger;
mod life;
mod mechanisms;
mod memory;
mod model;
mod net;
mod program;
mod quantity;
mod rng;
mod rules;
mod scale;
mod school;
mod store;

use data::{Dataset, Example, Family};
use ledger::{fmt_joules, EnergyModel, Ledger, Rapl, RAPL_PATH};
use life::{Settings, Strategy};
use memory::Memory;
use model::{build_answers, build_words, Model};
use net::{Adam, Config, Net, Reader, Scratch};
use rng::Rng;
use std::collections::HashMap;

const DEFAULT_MODEL: &str = "runs/baby.model";

struct Args {
    positional: Vec<String>,
    opts: HashMap<String, String>,
}

impl Args {
    fn parse(raw: &[String]) -> Args {
        const FLAGS: [&str; 20] = ["curriculum", "quiet", "tsv", "verbose", "guess", "piece-keys", "full", "compact", "no-reorg", "row", "digits", "stage2-first", "active", "wondered", "check", "careful", "permissive", "guesses", "corrections", "calibrate"];
        let mut positional = Vec::new();
        let mut opts = HashMap::new();
        let mut i = 0;
        while i < raw.len() {
            if let Some(name) = raw[i].strip_prefix("--") {
                if FLAGS.contains(&name) {
                    opts.insert(name.to_string(), String::new());
                } else {
                    i += 1;
                    opts.insert(name.to_string(), raw.get(i).cloned().unwrap_or_default());
                }
            } else {
                positional.push(raw[i].clone());
            }
            i += 1;
        }
        Args { positional, opts }
    }

    fn flag(&self, name: &str) -> bool {
        self.opts.contains_key(name)
    }

    fn get<T: std::str::FromStr>(&self, name: &str, default: T) -> T {
        self.opts.get(name).and_then(|v| v.parse().ok()).unwrap_or(default)
    }

    fn string(&self, name: &str, default: &str) -> String {
        self.opts.get(name).cloned().unwrap_or_else(|| default.to_string())
    }
}

struct Encoded {
    ids: Vec<usize>,
    target: usize,
    family: Family,
}

fn encode_all(model: &Model, set: &[Example]) -> Vec<Encoded> {
    set.iter()
        .map(|e| Encoded { ids: model.encode(&e.query), target: model.answer_id(&e.answer).expect("answer in list"), family: e.family })
        .collect()
}

fn argmax(p: &[f32]) -> usize {
    p.iter().enumerate().max_by(|a, b| a.1.partial_cmp(b.1).unwrap()).map(|(i, _)| i).unwrap_or(0)
}

/// (correct, total) overall and per family.
fn accuracy(net: &Net, set: &[Encoded], s: &mut Scratch) -> (usize, usize, HashMap<Family, (usize, usize)>) {
    let mut per: HashMap<Family, (usize, usize)> = HashMap::new();
    let mut correct = 0;
    for e in set {
        let ok = argmax(&net.probs(&e.ids, s)) == e.target;
        correct += ok as usize;
        let slot = per.entry(e.family).or_insert((0, 0));
        slot.0 += ok as usize;
        slot.1 += 1;
    }
    (correct, set.len(), per)
}

fn pct(c: usize, t: usize) -> String {
    if t == 0 {
        "  -  ".to_string()
    } else {
        format!("{:5.1}", 100.0 * c as f32 / t as f32)
    }
}

fn report(net: &Net, sets: [&Vec<Encoded>; 5], s: &mut Scratch) {
    println!("\n{:<10} {:>8} {:>12} {:>12} {:>8} {:>10}", "family", "train %", "new-phrase %", "new-sum %", "typos %", "typo-key %");
    let results: Vec<_> = sets.iter().map(|set| accuracy(net, set, s)).collect();
    for fam in Family::ALL {
        let cell = |i: usize| {
            let (c, t) = results[i].2.get(&fam).copied().unwrap_or((0, 0));
            pct(c, t)
        };
        println!("{:<10} {:>8} {:>12} {:>12} {:>8} {:>10}", fam.name(), cell(0), cell(1), cell(2), cell(3), cell(4));
    }
    println!(
        "{:<10} {:>8} {:>12} {:>12} {:>8} {:>10}",
        "ALL", pct(results[0].0, results[0].1), pct(results[1].0, results[1].1), pct(results[2].0, results[2].1), pct(results[3].0, results[3].1), pct(results[4].0, results[4].1)
    );
}

fn stage_allowed(epoch: usize, epochs: usize) -> usize {
    let ramp = (epochs as f32 * 0.6).max(1.0);
    (1 + (epoch as f32 / (ramp / data::STAGES as f32)) as usize).min(data::STAGES)
}

fn cmd_train(a: &Args) {
    let seed: u64 = a.get("seed", 1);
    let epochs: usize = a.get("epochs", 300);
    let batch: usize = a.get("batch", 16);
    let lr: f32 = a.get("lr", 0.01);
    let wd: f32 = a.get("wd", 0.0);
    let curriculum = a.flag("curriculum");
    let quiet = a.flag("quiet");
    let out = a.string("out", DEFAULT_MODEL);

    let mut ds: Dataset = data::build(seed);
    // Optionally let it meet misspellings while learning (never the exact ones used for testing).
    let typo_aug: u64 = a.get("typo-aug", 0);
    if typo_aug > 0 {
        // No misspelt *word* used in testing may appear in training, in any question: the test must be about spellings it never met.
        let clean: std::collections::HashSet<String> = ds.train.iter().flat_map(|e| data::tokenize(&e.query)).collect();
        let test_misspellings: std::collections::HashSet<String> = ds
            .typo_test
            .iter()
            .chain(&ds.typo_entity_test)
            .flat_map(|e| data::tokenize(&e.query))
            .filter(|w| !clean.contains(w))
            .collect();
        let entities = data::entity_words();
        let originals = ds.train.clone();
        for k in 0..typo_aug {
            let only = if k % 2 == 0 { Some(&entities) } else { None };
            let extra = data::typos_of(&originals, seed.wrapping_add(31_337 + k), only);
            ds.train.extend(extra.into_iter().filter(|e| !data::tokenize(&e.query).iter().any(|w| test_misspellings.contains(w))));
        }
    }
    let words = build_words(&ds.train);
    let answers = build_answers(&[&ds.train, &ds.paraphrase_test, &ds.item_test]);
    let reader = Reader::parse(&a.string("reader", "concat")).unwrap_or_else(|| {
        eprintln!("--reader must be concat, bag, bigram or letters");
        std::process::exit(2);
    });
    let cfg = Config {
        vocab: words.len(),
        dim: a.get("dim", 16),
        len: data::LEN,
        hidden: a.get("hidden", 64),
        classes: answers.len(),
        reader,
        buckets: a.get("buckets", 512),
    };
    let mut rng = Rng::new(seed.wrapping_add(1000));
    let mut model = Model::new(Net::new(cfg, &mut rng), words, answers, seed);
    model.learn_familiarity(&ds.train);

    let train = encode_all(&model, &ds.train);
    let para = encode_all(&model, &ds.paraphrase_test);
    let items = encode_all(&model, &ds.item_test);
    let typos = encode_all(&model, &ds.typo_test);
    let typo_key = encode_all(&model, &ds.typo_entity_test);
    let stage_of: Vec<usize> = ds.train.iter().map(|e| e.family.stage()).collect();

    println!(
        "baby: {} params, {} reader ({} words in, {} words out), {} train / {} new-phrase / {} new-sum examples, {}",
        cfg.n_params(), cfg.reader.name(), cfg.vocab, cfg.classes, train.len(), para.len(), items.len(),
        if curriculum { "curriculum" } else { "shuffled" }
    );

    let mut scratch = Scratch::new(&cfg);
    let mut adam = Adam::new(cfg.n_params(), lr, wd);
    let mut grad = vec![0.0f32; cfg.n_params()];
    let mut steps = 0usize;
    let started = std::time::Instant::now();

    for epoch in 0..epochs {
        let allowed = if curriculum { stage_allowed(epoch, epochs) } else { data::STAGES };
        let mut idx: Vec<usize> = (0..train.len()).filter(|&i| stage_of[i] <= allowed).collect();
        rng.shuffle(&mut idx);
        let mut total = 0.0f32;
        for chunk in idx.chunks(batch) {
            grad.iter_mut().for_each(|g| *g = 0.0);
            for &i in chunk {
                total += model.net.loss_and_grad(&train[i].ids, train[i].target, &mut grad, &mut scratch);
            }
            let inv = 1.0 / chunk.len() as f32;
            grad.iter_mut().for_each(|g| *g *= inv);
            adam.step(&mut model.net.theta, &grad, &mut scratch.ledger);
            steps += 1;
        }
        let last = epoch + 1 == epochs;
        if !quiet && ((epoch + 1) % (epochs / 10).max(1) == 0 || last) {
            let (tc, tt, _) = accuracy(&model.net, &train, &mut scratch);
            let (pc, pt, _) = accuracy(&model.net, &para, &mut scratch);
            let (ic, it, _) = accuracy(&model.net, &items, &mut scratch);
            println!(
                "epoch {:>4}  stage {}  loss {:.4}  train {}%  new-phrase {}%  new-sum {}%",
                epoch + 1, allowed, total / idx.len().max(1) as f32, pct(tc, tt), pct(pc, pt), pct(ic, it)
            );
        }
    }
    println!(
        "{} steps in {:.1}s, about {} of energy (estimated from counted operations, see ledger.rs)",
        steps, started.elapsed().as_secs_f32(), fmt_joules(EnergyModel::default().joules(&scratch.ledger))
    );
    report(&model.net, [&train, &para, &items, &typos, &typo_key], &mut scratch);

    // Numeric questions: network or rule? Decided from sums held back and tested, as in `baby rules`.
    if train.iter().any(|_| true) && ds.train.iter().any(rules::is_sum) {
        let d = rules::decide(&ds, cfg.reader, seed);
        if d.chosen == rules::Mechanism::Rule {
            model.rule = rules::fit_rule(&ds.train).active_ops();
        }
        println!(
            "numeric questions: it chose the {} (on sums held back: network {:.0}%, rule {:.0}%)",
            if d.chosen == rules::Mechanism::Rule { "rule" } else { "network" },
            100.0 * d.network_on_held_back,
            100.0 * d.rule_on_held_back
        );
    }
    model.calibrate(&ds.paraphrase_test, 0.95, &mut scratch);
    println!("calibrated refusal floors from its own experience: confidence {:.3}, familiarity {:.3}", model.conf_floor, model.fam_floor);
    match model.save(&out) {
        Ok(()) => println!("\nsaved {out}"),
        Err(e) => eprintln!("could not save {out}: {e}"),
    }
}

fn memory_path(a: &Args, model_path: &str) -> String {
    a.string("memory", &format!("{}.memory", model_path.trim_end_matches(".model").trim_end_matches(".school")))
}

/// Edit distance between two words.
fn edit_distance(a: &str, b: &str) -> usize {
    let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for i in 1..=a.len() {
        let mut cur = vec![i];
        for j in 1..=b.len() {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            cur.push((prev[j] + 1).min(cur[j - 1] + 1).min(prev[j - 1] + cost));
        }
        prev = cur;
    }
    prev[b.len()]
}

/// A word it has never met that is one or two letters from exactly one word it knows (and is long enough for that to mean
/// something) is probably a misspelling of it. Returns the question with such words replaced, and what it assumed.
fn assume_spellings(model: &Model, memory: &Memory, query: &str) -> (String, Vec<(String, String)>) {
    let known: Vec<&str> = model.words.iter().map(String::as_str).filter(|w| !w.starts_with('<')).chain(data::NUM).collect();
    assume_spellings_with(&known, &|w| model.knows(w), memory, query)
}

fn assume_spellings_with(known: &[&str], is_known: &dyn Fn(&str) -> bool, memory: &Memory, query: &str) -> (String, Vec<(String, String)>) {
    let mut fixes = Vec::new();
    let words: Vec<String> = data::tokenize(query)
        .into_iter()
        .map(|w| {
            if is_known(&w) || memory.knows(&w) || data::NUM.contains(&w.as_str()) || w.chars().count() < 5 {
                return w;
            }
            let limit = if w.chars().count() >= 7 { 2 } else { 1 };
            let mut best: Vec<(&str, usize)> = known.iter().map(|k| (*k, edit_distance(&w, k))).filter(|(_, d)| *d <= limit).collect();
            best.sort_by_key(|x| x.1);
            match best.as_slice() {
                [(only, _)] => {
                    fixes.push((w.clone(), only.to_string()));
                    only.to_string()
                }
                [(first, d1), (_, d2), ..] if d1 < d2 => {
                    fixes.push((w.clone(), first.to_string()));
                    first.to_string()
                }
                _ => w,
            }
        })
        .collect();
    (words.join(" "), fixes)
}

const DEFAULT_SCHOOL: &str = "runs/baby.school";

fn ask_school(a: &Args, school_path: &str, query: &str) {
    let mut school = school::School::load(school_path).unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(1);
    });
    if a.flag("permissive") {
        school.set_careful(false);
    } else if a.flag("careful") {
        school.set_careful(true);
    }
    let memory = Memory::open(&memory_path(a, school_path), a.get("match", 0.6));
    let weight = |w: &str| school.word_weight(w);
    let mut ledger = Ledger::default();
    // 0. Numbers are plain whole numbers. Check the question as written: the steps below rebuild it from bare words, which drops a sign.
    if let Some(bad) = school::unreadable_number(query) {
        println!("?");
        eprintln!("  I only read plain whole numbers, not '{bad}' (signs, decimals, fractions, percentages).");
        return;
    }
    // 1. What it has been told comes first.
    if let Some((answer, sim)) = memory.lookup(query, &weight, &mut ledger) {
        println!("{answer}");
        eprintln!("  (from what you taught me, match {sim:.2})");
        return;
    }
    // 2. Maybe a misspelling of something it knows. Say what it assumed.
    let known_owned = school.known_words();
    let known: Vec<&str> = known_owned.iter().map(String::as_str).collect();
    let (fixed, fixes) = assume_spellings_with(&known, &|w| school.knows(w), &memory, query);
    if !fixes.is_empty() {
        let said: Vec<String> = fixes.iter().map(|(from, to)| format!("'{from}' to mean '{to}'")).collect();
        eprintln!("  (I took {})", said.join(" and "));
        if let Some((answer, sim)) = memory.lookup(&fixed, &weight, &mut ledger) {
            println!("{answer}");
            eprintln!("  (from what you taught me, match {sim:.2})");
            return;
        }
    }
    // 3. A word it has never met: say so instead of guessing.
    let unmet: Vec<String> = data::tokenize(&fixed).into_iter().filter(|w| !school.knows(w) && !memory.knows(w)).collect();
    if !unmet.is_empty() && !a.flag("guess") {
        println!("?");
        eprintln!(
            "  I haven't met the word{} {}. If you tell me, I'll remember: baby teach \"{query}\" <answer>",
            if unmet.len() > 1 { "s" } else { "" },
            unmet.join(", ")
        );
        return;
    }
    // 4. Otherwise the unit it belongs to answers, or says why it won't.
    let reply = school.ask(&fixed);
    match reply.answer {
        Some(answer) => {
            println!("{answer}");
            eprintln!("  ({})", reply.how);
        }
        None if reply.guess.is_some() => {
            let g = reply.guess.unwrap();
            println!("probably {}", g.answer);
            let record = if g.tried >= 5 {
                format!("my guesses this far out have been right {:.0}% of the time, over {} checked", 100.0 * g.rate, g.tried)
            } else {
                format!("only {} of my guesses this far out have been checked, so I can't say how often they are right", g.tried)
            };
            eprintln!("  (a guess, not a fact: {}. {record}. If you know the answer, tell me and I'll learn from it: baby learn \"{query}\" <answer>)", reply.how);
        }
        None => {
            println!("?");
            eprintln!("  I don't know this one: {}. Teach me: baby teach \"{query}\" <answer>", reply.how);
        }
    }
}

fn print_scorecard(school: &school::School, world: &kinds::World) {
    // The honest scorecard: not only how often it is right, but how often it answers WRONGLY instead of saying it doesn't know.
    println!("  {:<22} {:>7} {:>9} {:>8}   {:>24}", "questions", "right", "refused", "WRONG", "of the refused: a guess");
    println!("  {:<22} {:>7} {:>9} {:>8}   {:>11} {:>12}", "", "", "", "", "right", "wrong");
    let sets: [(&str, &Vec<kinds::Item>); 4] =
        [("taught", &world.train), ("reworded", &world.reworded), ("never taught", &world.new_items), ("beyond the taught range", &world.beyond)];
    for (name, set) in sets {
        let (mut right, mut refused, mut wrong, mut guess_right, mut guess_wrong) = (0usize, 0usize, 0usize, 0usize, 0usize);
        for it in set.iter() {
            let reply = school.ask(&it.query);
            match reply.answer {
                Some(a) if a == it.answer => right += 1,
                Some(_) => wrong += 1,
                None => {
                    refused += 1;
                    match reply.guess {
                        Some(g) if g.answer == it.answer => guess_right += 1,
                        Some(_) => guess_wrong += 1,
                        None => {}
                    }
                }
            }
        }
        let t = set.len().max(1) as f64;
        println!(
            "  {:<22} {:>6.1}% {:>8.1}% {:>7.1}%   {:>10.1}% {:>11.1}%",
            name,
            100.0 * right as f64 / t,
            100.0 * refused as f64 / t,
            100.0 * wrong as f64 / t,
            100.0 * guess_right as f64 / t,
            100.0 * guess_wrong as f64 / t
        );
    }
}

fn cmd_grade(a: &Args) {
    let path = a.string("school", DEFAULT_SCHOOL);
    let seed: u64 = a.get("seed", 1);
    let school = school::School::load(&path).unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(1);
    });
    println!("grading {path} on the curriculum with seed {seed}\n");
    print_scorecard(&school, &kinds::world(seed));
}

fn cmd_school(a: &Args) {
    let seed: u64 = a.get("seed", 1);
    let out = a.string("out", DEFAULT_SCHOOL);
    let started = std::time::Instant::now();
    let world = kinds::world(seed);
    println!(
        "Teaching the baby {} lessons in {} units. For each unit it will work out, from facts it holds back, how to learn it.\n",
        world.train.len(),
        kinds::UNITS.len()
    );
    let mut school = school::School::train(&world, seed, !a.flag("full"));
    school.set_careful(!a.flag("permissive"));
    println!("  {:<9} {:<9} {:>9}   built / note", "unit", "learns by", "held back");
    for u in &school.units {
        let note = match (u.program(), u.memorise_only()) {
            (Some(p), _) => p,
            (None, true) => "memorise only: nothing here can be worked out".to_string(),
            (None, false) => String::new(),
        };
        println!("  {:<9} {:<9} {:>8.0}%   {}", u.name, u.chosen.name(), 100.0 * u.best_evidence(), note);
    }
    println!("\nrecognises a kind of question when one kind of lesson beats the next by at least {:.2}\n", school.route_floor);
    print_scorecard(&school, &world);
    match school.save(&out) {
        Ok(()) => println!("taught in {:.1}s, saved {out} (try: baby ask \"how far apart are two and nine\")", started.elapsed().as_secs_f32()),
        Err(e) => {
            eprintln!("could not save {out}: {e}");
            std::process::exit(1);
        }
    }
}

fn cmd_ask(a: &Args) {
    let query = a.positional.get(1..).map(|w| w.join(" ")).unwrap_or_default();
    if query.is_empty() {
        eprintln!("usage: baby ask \"capital of france\" [--model PATH] [--top N] [--guess]");
        std::process::exit(2);
    }
    // The unified baby, if one has been taught (`baby school`) and no older model was asked for.
    let school_path = a.string("school", DEFAULT_SCHOOL);
    if !a.opts.contains_key("model") && std::path::Path::new(&school_path).exists() {
        return ask_school(a, &school_path, &query);
    }
    let path = a.string("model", DEFAULT_MODEL);
    let model = Model::load(&path).unwrap_or_else(|e| {
        eprintln!("{e}\n(teach one first: baby school)");
        std::process::exit(1);
    });
    let memory = Memory::open(&memory_path(a, &path), a.get("match", 0.6));
    let weight = |w: &str| model.word_weight(w);
    let mut ledger = Ledger::default();

    // 1. What it has been told comes first: no weights involved, any word, any answer.
    if let Some((answer, sim)) = memory.lookup(&query, &weight, &mut ledger) {
        println!("{answer}");
        eprintln!("  (from what you taught me, match {sim:.2})");
        return;
    }
    // 1b. Maybe a misspelling of something it knows. Say what it assumed, then carry on with that.
    let original = query.clone();
    let (query, fixes) = assume_spellings(&model, &memory, &query);
    if !fixes.is_empty() {
        let said: Vec<String> = fixes.iter().map(|(from, to)| format!("'{from}' to mean '{to}'")).collect();
        eprintln!("  (I took {})", said.join(" and "));
        if let Some((answer, sim)) = memory.lookup(&query, &weight, &mut ledger) {
            println!("{answer}");
            eprintln!("  (from what you taught me, match {sim:.2})");
            return;
        }
    }
    let _ = original;
    // 2. Quantities: if it decided a rule serves these better than the network, and the question is an operation it knows.
    if !model.rule.is_empty() {
        if let Some((answer, dist)) = quantity::Quantity::from_ops(model.rule.clone()).answer(&query) {
            if dist < 0.25 {
                println!("{answer}");
                eprintln!("  (worked out by the rule, not recalled)");
                return;
            }
        }
    }
    // 3. A word it has never met: say so instead of guessing.
    let unmet: Vec<String> = data::tokenize(&query).into_iter().filter(|w| !model.knows(w) && !memory.knows(w)).collect();
    if !unmet.is_empty() && !a.flag("guess") {
        println!("?");
        eprintln!(
            "  I haven't met the word{} {}. If you tell me, I'll remember: baby teach \"{query}\" <answer>",
            if unmet.len() > 1 { "s" } else { "" },
            unmet.join(", ")
        );
        return;
    }
    // 4. Otherwise the network answers, unless it isn't sure.
    let mut s = Scratch::new(&model.net.cfg);
    let top = model.ask(&query, a.get("top", 1).max(1), &mut s);
    let conf_floor = if model.conf_floor > 0.0 { model.conf_floor } else { 0.5 };
    let familiarity = if model.fam_floor > 0.0 { Some(model.familiarity(&model.familiarity_store(), &query)) } else { None };
    let unfamiliar = familiarity.map_or(false, |f| f < model.fam_floor);
    if (top[0].1 < conf_floor || unfamiliar) && !a.flag("guess") {
        println!("?");
        eprintln!(
            "  I don't know this one ({}); my best guess would be {} ({:.2}). Teach me: baby teach \"{query}\" <answer>",
            if unfamiliar { "nothing I was taught is like it" } else { "I'm not sure enough" },
            top[0].0,
            top[0].1
        );
        return;
    }
    println!("{}", top[0].0);
    if a.get::<usize>("top", 1) > 1 {
        for (w, p) in &top {
            eprintln!("  {w:<12} {:.3}", p);
        }
    }
}

fn cmd_teach(a: &Args) {
    let args = &a.positional[1.min(a.positional.len())..];
    if args.len() < 2 {
        eprintln!("usage: baby teach \"capital of atlantis\" poseidonis   (the last word is the answer)");
        std::process::exit(2);
    }
    let answer = &args[args.len() - 1];
    let query = args[..args.len() - 1].join(" ");
    let path = a.string("model", DEFAULT_MODEL);
    let mem_path = memory_path(a, &path);
    let mut memory = Memory::open(&mem_path, 0.6);
    match memory.teach(&query, answer) {
        Ok(()) => println!("learned: \"{query}\" -> {answer}   ({} facts taught so far, kept in {mem_path}; no weights changed)", memory.len()),
        Err(e) => {
            eprintln!("could not save to {mem_path}: {e}");
            std::process::exit(1);
        }
    }
}

fn cmd_know(a: &Args) {
    let seeds: u64 = a.get("seeds", 4);
    let keep: f32 = a.get("keep", 0.95);
    println!("Does the baby know what it doesn't know? Mean of {seeds} seeds. Each signal's threshold keeps {:.0}% of reworded real questions answerable.\n", keep * 100.0);
    for reader in [Reader::Concat, Reader::Bag, Reader::Letters] {
        println!("reader: {}", reader.name());
        println!("  {:<26} {:>9} {:>11} {:>14} {:>16}", "when it refuses", "answers%", "right when", "refuses unmet", "refuses nonsense");
        println!("  {:<26} {:>9} {:>11} {:>14} {:>16}", "", "(real)", "answering%", "words %", "(known words) %");
        for (name, cov, acc, un, non) in know::run(reader, seeds, keep) {
            println!("  {:<26} {:>9.1} {:>11.1} {:>14.1} {:>16.1}", name, cov, acc, un, non);
        }
        println!();
    }
    println!("unmet words = e.g. \"capital of atlantis\"; nonsense = every word familiar, the question makes no sense (\"capital of banana\").");
}

fn cmd_eval(a: &Args) {
    let path = a.string("model", DEFAULT_MODEL);
    let model = Model::load(&path).unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(1);
    });
    let ds = data::build(model.data_seed);
    let (train, para, items, typos, typo_key) = (
        encode_all(&model, &ds.train),
        encode_all(&model, &ds.paraphrase_test),
        encode_all(&model, &ds.item_test),
        encode_all(&model, &ds.typo_test),
        encode_all(&model, &ds.typo_entity_test),
    );
    let mut s = Scratch::new(&model.net.cfg);
    report(&model.net, [&train, &para, &items, &typos, &typo_key], &mut s);
}

fn cmd_data(a: &Args) {
    let ds = data::build(a.get("seed", 1));
    if a.flag("tsv") {
        for (name, set) in [("train", &ds.train), ("new-phrase", &ds.paraphrase_test), ("new-sum", &ds.item_test)] {
            for e in set {
                println!("{name}\t{}\t{}\t{}\t{}", e.family.name(), e.template, e.query, e.answer);
            }
        }
        return;
    }
    println!("{:<10} {:>6} {:>11} {:>8}", "family", "train", "new-phrase", "new-sum");
    for fam in Family::ALL {
        let n = |set: &[Example]| set.iter().filter(|e| e.family == fam).count();
        println!("{:<10} {:>6} {:>11} {:>8}", fam.name(), n(&ds.train), n(&ds.paraphrase_test), n(&ds.item_test));
    }
    println!("\nexamples:");
    for e in ds.train.iter().step_by(ds.train.len() / 10) {
        println!("  {:<34} -> {}", e.query, e.answer);
    }
}

fn mean(xs: &[f64]) -> f64 {
    xs.iter().sum::<f64>() / xs.len().max(1) as f64
}

fn rate(c: (usize, usize)) -> f64 {
    if c.1 == 0 { 0.0 } else { 100.0 * c.0 as f64 / c.1 as f64 }
}

fn cmd_life(a: &Args) {
    let mut st = Settings::default();
    st.days = a.get("days", st.days);
    st.max_steps = a.get("max-steps", st.max_steps);
    st.replay_k = a.get("replay-k", st.replay_k);
    st.sleep_every = a.get("sleep-every", st.sleep_every);
    st.sleep_epochs = a.get("sleep-epochs", st.sleep_epochs);
    st.store_threshold = a.get("threshold", st.store_threshold);
    st.lr = a.get("lr", st.lr);
    st.dim = a.get("dim", st.dim);
    st.hidden = a.get("hidden", st.hidden);
    st.sgd = a.string("optimizer", "adam") == "sgd";
    st.reader = Reader::parse(&a.string("reader", "concat")).unwrap_or_else(|| {
        eprintln!("--reader must be concat, bag, bigram or letters");
        std::process::exit(2);
    });
    st.buckets = a.get("buckets", st.buckets);
    st.piece_keys = a.flag("piece-keys");
    let seeds: u64 = a.get("seeds", 3);
    let base: u64 = a.get("seed", 1);
    let verbose = a.flag("verbose");
    let strategies: Vec<Strategy> = a
        .string("strategies", "naive,replay,store,sleep,retrain")
        .split(',')
        .filter_map(Strategy::parse)
        .collect();
    let energy = EnergyModel::default();

    println!(
        "A baby is born knowing its words but no facts. {} days of lessons (colors and sounds, then opposites, capitals, sums), one lesson at a time.",
        st.days
    );
    println!("Mean over {seeds} seeds. Energy is ESTIMATED from counted operations (45 nm figures); trust the ratios, not the absolute joules.\n");
    println!(
        "{:<8} {:>9} {:>11} {:>9} {:>8} {:>9} {:>11} {:>7} {:>8} {:>10}",
        "method", "retained%", "new-phrase%", "new-sum%", "typos%", "updates", "energy", "math%", "store", "learned/n"
    );

    for strat in strategies {
        let mut rows: Vec<life::Outcome> = Vec::new();
        for k in 0..seeds {
            let seed = base + k;
            let ds = data::build(seed);
            rows.push(life::run(&ds, strat, &st, seed, &energy));
        }
        let fin = |f: &dyn Fn(&life::Outcome) -> f64| mean(&rows.iter().map(f).collect::<Vec<_>>());
        let retained = fin(&|o| rate(o.checkpoints.last().unwrap().retained));
        let newp = fin(&|o| rate(o.checkpoints.last().unwrap().new_phrase));
        let sums = fin(&|o| rate(o.new_sums));
        println!(
            "{:<8} {:>9.1} {:>11.1} {:>9.1} {:>8.1} {:>9.0} {:>11} {:>6.0}% {:>8.0} {:>6.0}/{:<4.0}",
            strat.name(),
            retained,
            newp,
            sums,
            fin(&|o| rate(o.typos)),
            fin(&|o| o.updates as f64),
            fmt_joules(fin(&|o| o.joules)),
            100.0 * fin(&|o| o.arithmetic_share),
            fin(&|o| o.store_end as f64),
            fin(&|o| o.lessons_learned as f64),
            fin(&|o| o.lessons_heard as f64),
        );
        if verbose {
            for cp in &rows[0].checkpoints {
                println!(
                    "           day {} through stage {}: retained {:5.1}%  new-phrase {:5.1}%   energy so far {}  updates {}",
                    cp.day, cp.stage, rate(cp.retained), rate(cp.new_phrase), fmt_joules(cp.joules), cp.updates
                );
            }
            println!("           sleeps {}, store peak {}, wall {:.0} ms", rows[0].sleeps, rows[0].store_peak, rows[0].millis);
        }
    }
    println!("\nretained = answers right on every lesson taught (as taught); new-phrase = right on a rewording of taught facts;");
    println!("new-sum = right on sums never taught; learned/n = lessons that needed learning out of lessons heard; math% = share of energy spent on arithmetic (the rest is memory traffic).");
}

fn cmd_energy(a: &Args) {
    let path = a.string("model", DEFAULT_MODEL);
    let model = Model::load(&path).unwrap_or_else(|e| {
        eprintln!("{e}\n(train one first: baby train)");
        std::process::exit(1);
    });
    let em = EnergyModel::default();
    let cfg = model.net.cfg;
    let ds = data::build(model.data_seed);
    let queries: Vec<Vec<usize>> = ds.train.iter().map(|e| model.encode(&e.query)).collect();
    let targets: Vec<usize> = ds.train.iter().map(|e| model.answer_id(&e.answer).unwrap()).collect();
    let mut s = Scratch::new(&cfg);

    s.ledger = Ledger::default();
    model.net.probs(&queries[0], &mut s);
    let ask = s.ledger;

    let mut grad = vec![0.0f32; cfg.n_params()];
    let mut adam = Adam::new(cfg.n_params(), 0.01, 0.0);
    let mut theta = model.net.theta.clone();
    let train_net = Net::from_theta(cfg, theta.clone());
    s.ledger = Ledger::default();
    for i in 0..16 {
        train_net.loss_and_grad(&queries[i], targets[i], &mut grad, &mut s);
    }
    adam.step(&mut theta, &grad, &mut s.ledger);
    let step16 = s.ledger;
    s.ledger = Ledger::default();
    train_net.loss_and_grad(&queries[0], targets[0], &mut grad, &mut s);
    adam.step(&mut theta, &grad, &mut s.ledger);
    let step1 = s.ledger;

    let line = |name: &str, l: &Ledger| {
        let (arith, mem) = em.split_pj(l);
        println!(
            "{:<26} {:>10} MACs {:>10} words moved   {:>10}   (math {:.0}% / memory {:.0}%)",
            name, l.macs, l.words_read + l.words_written, fmt_joules((arith + mem) * 1e-12), 100.0 * arith / (arith + mem), 100.0 * mem / (arith + mem)
        );
    };
    println!("baby: {} parameters = {} KB of fp32 weights; working set {} KB", cfg.n_params(), cfg.n_params() * 4 / 1024, s.ledger.ws_bytes / 1024);
    println!("Counted, then priced with 45 nm figures (an ESTIMATE; compare the ratios):\n");
    line("answer one question", &ask);
    line("learn one lesson (1 step)", &step1);
    line("learn, batch of 16", &step16);
    let gpt = 2.0 * 7.0e9;
    println!(
        "\nFor scale: answering costs {} MACs; a 7-billion-parameter model spends about {:.1e} per token (about {:.0}x more arithmetic),",
        ask.macs, gpt / 2.0, gpt / 2.0 / ask.macs as f64
    );
    println!("and its weights (~4 GB at 4-bit) cannot sit in cache, so every token pays DRAM prices ({} pJ/word vs {} here).", em.pj_word_dram, em.pj_per_word(ask.ws_bytes));

    // Wall clock on this machine.
    let n = 600_000usize;
    let t = std::time::Instant::now();
    let r = Rapl::open();
    let e0 = r.as_ref().and_then(|r| r.read_uj());
    let mut sink = 0usize;
    for i in 0..n {
        sink = sink.wrapping_add(model.net.probs(&queries[i % queries.len()], &mut s).len());
    }
    let secs = t.elapsed().as_secs_f64();
    let e1 = r.as_ref().and_then(|r| r.read_uj());
    std::hint::black_box(sink);
    println!("\nMeasured here: {:.2} us per question ({:.0} questions/s on one core).", secs / n as f64 * 1e6, n as f64 / secs);
    match (&r, e0, e1) {
        (Some(r), Some(e0), Some(e1)) => {
            let total = r.delta_j(e0, e1);
            let i0 = r.read_uj().unwrap();
            std::thread::sleep(std::time::Duration::from_secs(2));
            let idle = r.delta_j(i0, r.read_uj().unwrap()) / 2.0;
            let above = (total - idle * secs).max(0.0);
            println!(
                "Package energy (RAPL): {:.2} J over {:.2} s = {:.1} W; idle {:.1} W; so {} above idle per question ({} total).",
                total, secs, total / secs, idle, fmt_joules(above / n as f64), fmt_joules(total / n as f64)
            );
            println!("Estimate said {} per question: the gap is what a general-purpose CPU spends around the arithmetic.", fmt_joules(em.joules(&ask)));
        }
        _ => println!(
            "Real energy not measured: this user cannot read {RAPL_PATH}/energy_uj. To allow it until reboot:\n  sudo chmod a+r {RAPL_PATH}/energy_uj"
        ),
    }
}

fn cmd_curious(a: &Args) {
    let seeds: u64 = a.get("seeds", 5);
    let threshold: f32 = a.get("threshold", 0.75);
    println!("A baby overhears 429 questions (143 different facts, each asked 3 ways) and may ask a teacher for answers.");
    println!("Accuracy on the overheard questions / on rewordings it never overheard, after at most N answers requested. Mean of {seeds} seeds.\n");
    print!("{:<9}", "method");
    for n in curious::MARKS {
        print!(" {:>10}", format!("N={n}"));
    }
    println!(" {:>10}", "asked");
    for pick in curious::Pick::ALL {
        let curves: Vec<curious::Curve> = (1..=seeds).map(|s| curious::run(&data::build(s), pick, s, threshold)).collect();
        print!("{:<9}", pick.name());
        for n in curious::MARKS {
            let (p, r) = (
                mean(&curves.iter().map(|c| curious::at(c, n).0).collect::<Vec<_>>()),
                mean(&curves.iter().map(|c| curious::at(c, n).1).collect::<Vec<_>>()),
            );
            print!(" {:>10}", format!("{:.0}/{:.0}%", 100.0 * p, 100.0 * r));
        }
        print!(" {:>10.0}", mean(&curves.iter().map(|c| c.labels as f64).collect::<Vec<_>>()));
        println!("   (match trusted at {:.2})", mean(&curves.iter().map(|c| c.final_threshold as f64).collect::<Vec<_>>()));
    }
    println!("\nrandom = asks about everything in random order; stream = asks only after being wrong; curious = asks about whatever is least like what it knows, and stops when nothing is;\ntrusting = stream, but it learns how alike two questions must be to trust a match, instead of being told.");
}

fn cmd_rules(a: &Args) {
    let seeds: u64 = a.get("seeds", 5);
    println!("Sums: a table or a rule? Right answers on sums only. Mean of {seeds} seeds.\n");
    println!("{:<30} {:>8} {:>10} {:>14} {:>16}", "mechanism", "taught", "reworded", "never taught", "beyond the range");
    println!("{:<30} {:>8} {:>10} {:>14} {:>16}", "", "", "", "(same numbers)", "(up to ten)");
    let extra = data::extrapolation();
    let readers = [Reader::Bag, Reader::Letters];
    // Rows: bag network, letters network, quantity rule, the baby's own choice.
    let mut acc = [[0.0f64; 4]; 4];
    let (mut chose_rule, mut val_net, mut val_rule) = (0u64, 0.0f64, 0.0f64);
    let mut sample: Vec<(String, [f32; 3])> = Vec::new();
    let frac = |c: (usize, usize)| c.0 as f64 / c.1.max(1) as f64;
    for seed in 1..=seeds {
        let ds = data::build(seed);
        let sets: [&[Example]; 4] = [&ds.train, &ds.paraphrase_test, &ds.item_test, &extra];
        let models: Vec<Model> = readers.iter().map(|r| know::fit(&ds, *r, seed)).collect();
        let rule = rules::fit_rule(&ds.train);
        let d = rules::decide(&ds, Reader::Bag, seed);
        chose_rule += (d.chosen == rules::Mechanism::Rule) as u64;
        val_net += d.network_on_held_back;
        val_rule += d.rule_on_held_back;
        for k in 0..4 {
            acc[0][k] += frac(rules::net_right(&models[0], sets[k]));
            acc[1][k] += frac(rules::net_right(&models[1], sets[k]));
            acc[2][k] += frac(rules::rule_right(&rule, sets[k]));
            acc[3][k] += if d.chosen == rules::Mechanism::Rule {
                frac(rules::rule_right(&rule, sets[k]))
            } else {
                frac(rules::net_right(&models[0], sets[k]))
            };
        }
        if seed == 1 {
            sample = ["plus", "minus", "equals", "what", "is"].iter().filter_map(|w| rule.operator(w).map(|o| (w.to_string(), o))).collect();
        }
    }
    let n = seeds as f64;
    let names = ["bag network (memorises)", "letters network (memorises)", "quantity rule (fitted)", "the baby's own choice"];
    for (i, name) in names.iter().enumerate() {
        println!("{:<30} {:>7.0}% {:>9.0}% {:>13.0}% {:>15.0}%", name, 100.0 * acc[i][0] / n, 100.0 * acc[i][1] / n, 100.0 * acc[i][2] / n, 100.0 * acc[i][3] / n);
    }
    println!(
        "\nHolding back a fifth of the sums it was taught and trying both ways, the baby measured: network {:.0}% vs rule {:.0}% on the held-back sums,\nand chose the rule in {} of {} seeds.",
        100.0 * val_net / n, 100.0 * val_rule / n, chose_rule, seeds
    );
    println!("\nWhat the rule learned about words (seed 1): multipliers for the first number, the second number, and a constant.");
    for (w, o) in sample {
        println!("  {w:<8} {:>6.2} {:>6.2} {:>6.2}", o[0], o[1], o[2]);
    }
}

fn cmd_kinds(a: &Args) {
    let seeds: u64 = a.get("seeds", 3);
    let units = kinds::UNITS;
    let systems: [(&str, Option<mechanisms::Mech>); 6] = [
        ("the baby's own choices", None),
        ("remember everywhere", Some(mechanisms::Mech::Remember)),
        ("rule everywhere", Some(mechanisms::Mech::Rule)),
        ("select everywhere", Some(mechanisms::Mech::Select)),
        ("program everywhere", Some(mechanisms::Mech::Program)),
        ("network everywhere", Some(mechanisms::Mech::Network)),
    ];
    let mut evidence = vec![[0.0f64; 5]; units.len()];
    let mut chosen = vec![[0u32; 5]; units.len()];
    let mut programs: Vec<Option<String>> = vec![None; units.len()];
    // [system][set][unit] = sum over seeds of accuracy (0 if the unit has no items in that set)
    let mut acc = vec![vec![vec![0.0f64; units.len()]; 3]; systems.len()];
    let mut has = vec![vec![false; units.len()]; 3];
    let (mut right_unit, mut all_unit) = (0usize, 0usize);
    for seed in 1..=seeds {
        let w = kinds::world(seed);
        let baby = mechanisms::learn(&w, seed);
        for (u, um) in baby.units.iter().enumerate() {
            for m in 0..5 {
                evidence[u][m] += um.evidence[m];
            }
            if seed == 1 {
                programs[u] = um.program.clone();
            }
            chosen[u][mechanisms::Mech::ALL.iter().position(|x| *x == um.chosen).unwrap()] += 1;
        }
        let sets: [&Vec<kinds::Item>; 3] = [&w.reworded, &w.new_items, &w.beyond];
        for (si, (_, forced)) in systems.iter().enumerate() {
            for (k, set) in sets.iter().enumerate() {
                for u in 0..units.len() {
                    let items: Vec<&kinds::Item> = set.iter().filter(|i| i.unit == u).collect();
                    if items.is_empty() {
                        continue;
                    }
                    has[k][u] = true;
                    let ok = items.iter().filter(|i| baby.answer(&i.query, *forced).as_deref() == Some(i.answer.as_str())).count();
                    acc[si][k][u] += ok as f64 / items.len() as f64;
                }
            }
        }
        for set in sets {
            for i in set.iter() {
                all_unit += 1;
                right_unit += (baby.unit_of(&i.query) == i.unit) as usize;
            }
        }
    }
    let n = seeds as f64;
    println!("{} units of lessons. The teacher says which questions belong together, never how to learn them. Mean of {seeds} seeds.\n", units.len());
    println!("What the baby measured on facts it held back (right %), and what it chose:");
    println!("  {:<9} {:>9} {:>7} {:>8} {:>8} {:>8}   chose", "unit", "remember", "rule", "select", "program", "network");
    for u in 0..units.len() {
        let picks: Vec<String> = (0..5).filter(|&m| chosen[u][m] > 0).map(|m| format!("{} {}/{}", mechanisms::Mech::ALL[m].name(), chosen[u][m], seeds)).collect();
        let best = (0..5).map(|m| evidence[u][m] / n).fold(0.0, f64::max);
        println!(
            "  {:<9} {:>8.0}% {:>6.0}% {:>7.0}% {:>7.0}% {:>7.0}%   {}{}",
            units[u], 100.0 * evidence[u][0] / n, 100.0 * evidence[u][1] / n, 100.0 * evidence[u][2] / n, 100.0 * evidence[u][3] / n, 100.0 * evidence[u][4] / n, picks.join(", "),
            if best < 0.8 { "   <- nothing here generalises; it can only memorise this" } else { "" }
        );
    }
    println!("\nExpressions the search built (seed 1):");
    for u in 0..units.len() {
        match &programs[u] {
            Some(p) => println!("  {:<9} {}", units[u], p),
            None => println!("  {:<9} (found none)", units[u]),
        }
    }
    println!("\nThen answering, right %, each unit counting equally:");
    println!("  {:<26} {:>10} {:>13} {:>18}", "system", "reworded", "new items", "beyond what it was taught");
    for (si, (name, _)) in systems.iter().enumerate() {
        let avg = |k: usize| {
            let us: Vec<usize> = (0..units.len()).filter(|&u| has[k][u]).collect();
            100.0 * us.iter().map(|&u| acc[si][k][u] / n).sum::<f64>() / us.len().max(1) as f64
        };
        println!("  {:<26} {:>9.0}% {:>12.0}% {:>17.0}%", name, avg(0), avg(1), avg(2));
    }
    println!("\nThe baby's own choices, unit by unit (reworded / new items / beyond):");
    for u in 0..units.len() {
        let cell = |k: usize| if has[k][u] { format!("{:>4.0}%", 100.0 * acc[0][k][u] / n) } else { "    -".to_string() };
        println!("  {:<9} {} {} {}", units[u], cell(0), cell(1), cell(2));
    }
    println!("\nWhich unit a question belongs to was guessed right {:.1}% of the time.", 100.0 * right_unit as f64 / all_unit.max(1) as f64);
}

fn cmd_learn(a: &Args) {
    let args = &a.positional[1.min(a.positional.len())..];
    if args.len() < 2 {
        eprintln!("usage: baby learn \"what is two multiplied by three\" six [--unit times]   (the last word is the answer)");
        std::process::exit(2);
    }
    let answer = &args[args.len() - 1];
    let query = args[..args.len() - 1].join(" ");
    let path = a.string("school", DEFAULT_SCHOOL);
    let mut school = school::School::load(&path).unwrap_or_else(|e| {
        eprintln!("{e}\n(teach one first: baby school)");
        std::process::exit(1);
    });
    let unit = a.opts.get("unit").cloned();
    if unit.is_none() {
        match school.learn_anywhere(&query, answer) {
            Ok(school::Outcome::Penned { pen, report }) => {
                if let Err(e) = school.save(&path) {
                    eprintln!("could not save {path}: {e}");
                    std::process::exit(1);
                }
                for (name, n) in &report.opened {
                    println!("a new kind of lesson gathered in the pen ({n} lessons): opened unit '{name}'");
                }
                for (name, n) in &report.joined {
                    println!("{n} waiting lessons built exactly the expression unit '{name}' builds: they joined it");
                }
                println!("learned: \"{query}\" -> {answer}   (this is not a kind of lesson I know yet: set aside with {} other{} waiting)", pen.saturating_sub(1), if pen == 2 { "" } else { "s" });
                for s in &report.suggestions {
                    println!("note: {s}");
                }
                if report.left == 0 {
                    println!("the pen is empty now");
                }
                return;
            }
            Ok(school::Outcome::Placed(r)) if !r.surprised => {
                println!("nothing to learn: unit '{}' already works out \"{query}\" -> {answer} (not stored)", r.unit);
                return;
            }
            Ok(school::Outcome::Placed(r)) => {
                let _ = school.save(&path);
                println!("learned: \"{query}\" -> {answer}   (added to unit '{}'; it now has {} facts, learns by {} at {:.0}% held back)", r.unit, r.facts, r.now.name(), 100.0 * r.held_back);
                return;
            }
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(1);
            }
        }
    }
    let learned = if a.flag("wondered") && unit.is_some() { school.learn_wondered(&query, answer, unit.as_deref().unwrap()) } else { school.learn(&query, answer, unit.as_deref()) };
    match learned {
        Ok(r) => {
            if let Err(e) = school.save(&path) {
                eprintln!("could not save {path}: {e}");
                std::process::exit(1);
            }
            if !r.surprised {
                println!("nothing to learn: unit '{}' already works out \"{query}\" -> {answer} (not stored)", r.unit);
                return;
            }
            let verb = if r.opened { format!("opened a new unit '{}'", r.unit) } else if r.corrected { format!("corrected a lesson in unit '{}'", r.unit) } else { format!("added to unit '{}'", r.unit) };
            println!("learned: \"{query}\" -> {answer}   ({verb}; it now has {} fact{})", r.facts, if r.facts == 1 { "" } else { "s" });
            let by = |m: mechanisms::Mech| m.name();
            match r.was {
                Some(w) if w != r.now => println!("it used to learn this unit by {}; with the new lesson it now learns it by {} ({:.0}% on facts held back)", by(w), by(r.now), 100.0 * r.held_back),
                _ if r.now == mechanisms::Mech::Remember && r.facts < 5 => println!("it only remembers this unit for now: with fewer than five facts there is not enough evidence to trust anything else"),
                _ => println!("it learns this unit by {} ({:.0}% on facts held back)", by(r.now), 100.0 * r.held_back),
            }
        }
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }
}

fn cmd_effort(a: &Args) {
    use ledger::{fmt_joules, EnergyModel, Ledger};
    let seed: u64 = a.get("seed", 1);
    let world = kinds::world(seed);
    let em = EnergyModel::default();

    println!("Teaching the baby twice, to see what the thrifty way saves in training:");
    let t = std::time::Instant::now();
    let school = school::School::train(&world, seed, true);
    let thrifty_secs = t.elapsed().as_secs_f32();
    let t = std::time::Instant::now();
    let _full = school::School::train(&world, seed, false);
    let full_secs = t.elapsed().as_secs_f32();
    println!("  thrifty (a network is only tried where cheaper ways did not already generalise): {thrifty_secs:.1}s");
    println!("  full (every mechanism cross-validated for every unit):                             {full_secs:.1}s\n");

    let sets: [(&str, &Vec<kinds::Item>); 4] =
        [("taught", &world.train), ("reworded", &world.reworded), ("never taught", &world.new_items), ("beyond the taught range", &world.beyond)];
    let all: Vec<&kinds::Item> = sets.iter().flat_map(|(_, s)| s.iter()).collect();
    let naive = school.naive_router();

    // (label, naive?)
    let mut rows = Vec::new();
    for (label, use_naive) in [("the slow way", true), ("what the baby does", false)] {
        let mut total = Ledger::default();
        let (mut right, mut wrong, mut refused) = (0usize, 0usize, 0usize);
        for it in &all {
            match school.ask_with(&it.query, &mut total, if use_naive { Some(&naive) } else { None }).answer {
                Some(x) if x == it.answer => right += 1,
                Some(_) => wrong += 1,
                None => refused += 1,
            }
        }
        // The cost of just finding the unit.
        let mut finding = Ledger::default();
        for it in &all {
            let s = mechanisms::shape(&it.query, &kinds_line());
            if use_naive {
                naive_route(&naive, &s, &mut finding);
            } else {
                school.route(&it.query, &mut finding);
            }
        }
        rows.push((label, total, finding, right, wrong, refused));
    }
    let n = all.len() as f64;
    let ops = |l: &Ledger| (l.macs + l.int_ops + l.other_ops) as f64 / n;
    println!("Answering {} questions (taught, reworded, never taught, beyond the range). Per question, counted then priced (45 nm figures, an ESTIMATE):\n", all.len());
    println!("  {:<24} {:>16} {:>12} {:>12}   {:>6} {:>8} {:>6}", "", "operations", "words read", "energy", "right", "refused", "WRONG");
    for (label, total, _finding, right, wrong, refused) in &rows {
        println!(
            "  {:<24} {:>16.0} {:>12.0} {:>12}   {:>5.1}% {:>7.1}% {:>5.1}%",
            label, ops(total), total.words_read as f64 / n, fmt_joules(em.joules(total) / n),
            100.0 * *right as f64 / n, 100.0 * *refused as f64 / n, 100.0 * *wrong as f64 / n
        );
    }
    // One small network answering everything, for scale.
    let words: usize = 120;
    let classes: usize = {
        let mut a: Vec<&str> = all.iter().map(|i| i.answer.as_str()).collect();
        a.sort();
        a.dedup();
        a.len()
    };
    let cfg = net::Config { vocab: words, dim: 16, len: data::LEN, hidden: 64, classes, reader: Reader::Bag, buckets: 0 };
    let net_ops = (cfg.hidden * cfg.input() + cfg.classes * cfg.hidden) as f64;
    let mut net_l = Ledger::default();
    net_l.macs = net_ops as u64;
    net_l.words_read = net_ops as u64;
    net_l.ws_bytes = (cfg.n_params() * 4) as u64;
    println!("  {:<24} {:>16.0} {:>12.0} {:>12}   (answers only what it memorised, and can't work out a new sum)", "one small network", net_ops, net_ops, fmt_joules(em.joules(&net_l)));
    // Fitting a rule: nudging it 800 times against solving it exactly (a different way of computing the same thing).
    {
        let sums: Vec<data::Example> = world
            .train
            .iter()
            .filter(|i| i.unit == 1)
            .map(|i| data::Example { family: data::Family::Add, template: i.template, query: i.query.clone(), answer: i.answer.clone() })
            .collect();
        let line = || quantity::NumberLine::from_counting(&quantity::counting_lessons());
        let t = std::time::Instant::now();
        for _ in 0..20 {
            std::hint::black_box(quantity::Quantity::fit(&sums, line(), 800, 0.05));
        }
        let nudge_us = t.elapsed().as_secs_f64() / 20.0 * 1e6;
        let t = std::time::Instant::now();
        let mut ops = 0;
        for _ in 0..20 {
            ops = std::hint::black_box(quantity::Quantity::fit_ridge(&sums, line(), &|_| 0.0)).1;
        }
        let exact_us = t.elapsed().as_secs_f64() / 20.0 * 1e6;
        let nudge_ops = quantity::Quantity::iterative_fit_ops(&sums, &line(), 800);
        println!(
            "\nFitting the rule for sums ({} lessons): nudging it 800 times = ~{} operations, {:.0} us; solving it exactly = {} operations, {:.0} us ({:.0}x fewer operations, {:.0}x faster).",
            sums.len(), nudge_ops, nudge_us, ops, exact_us, nudge_ops as f64 / ops as f64, nudge_us / exact_us
        );
    }
    println!("\n  of which, finding the unit:");
    for (label, _total, finding, ..) in &rows {
        println!("  {:<24} {:>16.0} {:>12.0} {:>12}", label, ops(finding), finding.words_read as f64 / n, fmt_joules(em.joules(finding) / n));
    }
    let (slow, fast) = (&rows[0].1, &rows[1].1);
    println!(
        "\nThe baby's way does {:.0}x fewer operations and moves {:.0}x fewer words than the slow way, answers {} the same questions rightly, and answers none wrongly.",
        ops(slow) / ops(fast), slow.words_read as f64 / fast.words_read as f64,
        if rows[1].3 >= rows[0].3 { "at least" } else { "fewer of" }
    );
}

fn kinds_line() -> quantity::NumberLine {
    quantity::NumberLine::from_counting(&quantity::counting_lessons())
}

fn naive_route(r: &school::NaiveRouter, shape: &[String], ledger: &mut ledger::Ledger) {
    r.route_public(shape, ledger);
}

fn cmd_discover(a: &Args) {
    let seed: u64 = a.get("seed", 1);
    let world = kinds::world(seed);
    // The teacher hands over a heap: questions and answers, shuffled, with no grouping at all.
    let mut order: Vec<usize> = (0..world.train.len()).collect();
    rng::Rng::new(seed.wrapping_add(5)).shuffle(&mut order);
    let lessons: Vec<(String, String)> = order.iter().map(|&i| (world.train[i].query.clone(), world.train[i].answer.clone())).collect();
    let truth: Vec<usize> = order.iter().map(|&i| world.train[i].unit).collect();
    println!("{} lessons with no grouping and no labels. Finding the units...\n", lessons.len());
    let started = std::time::Instant::now();
    let d = discover::discover(&lessons, seed);
    let found = started.elapsed().as_secs_f32();

    println!("  {:<22} {:>7}   made of (the units the teacher would have named)", "found unit", "lessons");
    for (u, name) in d.names.iter().enumerate() {
        let mut count: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
        let mut total = 0;
        for (it, o) in d.items.iter().zip(&d.origin) {
            if it.unit == u {
                *count.entry(kinds::UNITS[truth[*o]]).or_default() += 1;
                total += 1;
            }
        }
        let parts: Vec<String> = count.iter().map(|(k, v)| format!("{k} {v}")).collect();
        println!("  {:<22} {:>7}   {}", name, total, parts.join(", "));
    }
    println!("\n{} units found for {} the teacher would have named, in {:.1}s.\n", d.names.len(), kinds::UNITS.len(), found);

    let school = school::School::from_items(&d.items, d.names.clone(), None, seed, true);
    println!("  {:<22} {:<9} {:>9}   built", "found unit", "learns by", "held back");
    for u in &school.units {
        println!("  {:<22} {:<9} {:>8.0}%   {}", u.name, u.chosen.name(), 100.0 * u.best_evidence(), u.program().unwrap_or_default());
    }
    let labelled = school::School::train(&world, seed, true);
    let sets: [(&str, &Vec<kinds::Item>); 4] =
        [("taught", &world.train), ("reworded", &world.reworded), ("never taught", &world.new_items), ("beyond the taught range", &world.beyond)];
    println!("\nThe same questions put to the baby that found its own units, and to the one the teacher organised:\n");
    println!("  {:<24} {:>22} {:>22}", "", "found its own units", "organised by the teacher");
    println!("  {:<24} {:>7} {:>7} {:>6} {:>7} {:>7} {:>6}", "questions", "right", "refused", "WRONG", "right", "refused", "WRONG");
    for (name, set) in sets {
        let mut cells = Vec::new();
        for s in [&school, &labelled] {
            let (mut right, mut refused, mut wrong) = (0usize, 0usize, 0usize);
            for it in set.iter() {
                match s.ask(&it.query).answer {
                    Some(x) if x == it.answer => right += 1,
                    Some(_) => wrong += 1,
                    None => refused += 1,
                }
            }
            let t = set.len().max(1) as f64;
            cells.push((100.0 * right as f64 / t, 100.0 * refused as f64 / t, 100.0 * wrong as f64 / t));
        }
        println!(
            "  {:<24} {:>6.1}% {:>6.1}% {:>5.1}% {:>6.1}% {:>6.1}% {:>5.1}%",
            name, cells[0].0, cells[0].1, cells[0].2, cells[1].0, cells[1].1, cells[1].2
        );
    }
    if let Some(out) = a.opts.get("out") {
        match school.save(out) {
            Ok(()) => println!("\nsaved {out}"),
            Err(e) => eprintln!("could not save {out}: {e}"),
        }
    }
}

fn cmd_storage(a: &Args) {
    let path = a.string("school", DEFAULT_SCHOOL);
    let school = school::School::load(&path).unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(1);
    });
    let rows = school.storage();
    println!("{:<22} {:<9} {:>8} {:>14} {:>16}", "unit", "learns by", "lessons", "lessons bytes", "what it learned");
    let (mut l, mut b, mut s) = (0, 0, 0);
    for (name, m, lessons, lbytes, sbytes) in &rows {
        println!("{:<22} {:<9} {:>8} {:>14} {:>16}", name, m.name(), lessons, lbytes, sbytes);
        l += lessons;
        b += lbytes;
        s += sbytes;
    }
    println!("{:<22} {:<9} {:>8} {:>14} {:>16}", "ALL", "", l, b, s);
    let file = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
    println!("\nthe whole school file is {file} bytes; the lessons are {:.0}% of it. {} lesson(s) wait in the pen.", 100.0 * b as f64 / file.max(1) as f64, school.pen.len());
}

fn cmd_sleep(a: &Args) {
    let path = a.string("school", DEFAULT_SCHOOL);
    let mut school = school::School::load(&path).unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(1);
    });
    let report = school.process_pen();
    for (name, n) in &report.opened {
        println!("opened unit '{name}' from {n} waiting lessons");
    }
    for (name, n) in &report.joined {
        println!("{n} waiting lessons joined unit '{name}' (same expression)");
    }
    for s in &report.suggestions {
        println!("note: {s}");
    }
    println!("{} lesson(s) still waiting in the pen", report.left);
    let reorg = school.reorganise();
    for (from, to, n) in &reorg.split {
        println!("took {n} lessons out of unit '{from}' (they stopped following its rule) into unit '{to}'");
    }
    for (kept, gone) in &reorg.merged {
        println!("unit '{gone}' turned out to follow the same rule as '{kept}': merged");
    }
    if reorg.split.is_empty() && reorg.merged.is_empty() {
        println!("the units are grouped the way the lessons say they should be");
    }
    if a.flag("compact") {
        let before = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        let c = school.compact();
        println!("forgot what is already explained: {} lessons kept of {}", c.after, c.before);
        school.save(&path).unwrap_or_else(|e| eprintln!("could not save: {e}"));
        let after = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        println!("the school file went from {before} to {after} bytes ({:.0}% of what it was)", 100.0 * after as f64 / before.max(1) as f64);
    } else if let Err(e) = school.save(&path) {
        eprintln!("could not save: {e}");
    }
}

fn peak_memory_kb() -> u64 {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|t| t.lines().find(|l| l.starts_with("VmHWM")).and_then(|l| l.split_whitespace().nth(1).and_then(|n| n.parse().ok())))
        .unwrap_or(0)
}

/// `baby scale`: the world gets bigger; does learning stay light and the baby stay honest? One size per process (so peak memory is honest);
/// `--sweep` runs several sizes by starting this program once per size and prints the table at the end.
fn cmd_scale(a: &Args) {
    let seed: u64 = a.get("seed", 1);
    let noise: f32 = a.get("noise", 0.0);
    let typos: f32 = a.get("typos", 0.05);
    let facts: usize = a.get("facts", 12);
    let cap: usize = a.get("grade", 2500);
    if let Some(sweep) = a.opts.get("sweep") {
        let exe = std::env::current_exe().expect("own path");
        let mut rows: Vec<Vec<(String, f64)>> = Vec::new();
        for u in sweep.split(',').filter_map(|s| s.trim().parse::<usize>().ok()) {
            let entities = a.get("entities", (u * 20).clamp(1000, 10_000));
            let mut cmd = std::process::Command::new(&exe);
            cmd.args(["scale", "--units", &u.to_string(), "--entities", &entities.to_string(), "--facts", &facts.to_string(), "--noise", &noise.to_string(), "--typos", &typos.to_string(), "--seed", &seed.to_string(), "--grade", &cap.to_string(), "--row"]);
            if a.flag("no-reorg") {
                cmd.arg("--no-reorg");
            }
            if a.flag("digits") {
                cmd.arg("--digits");
            }
            let out = cmd.output().expect("run a size");
            let text = String::from_utf8_lossy(&out.stdout);
            let mut row = Vec::new();
            for line in text.lines() {
                match line.strip_prefix("ROW ") {
                    Some(r) => row = r.split_whitespace().filter_map(|kv| kv.split_once('=')).map(|(k, v)| (k.to_string(), v.parse().unwrap_or(0.0))).collect(),
                    None => println!("{line}"),
                }
            }
            if !out.status.success() {
                println!("  (size {u} failed: {})", String::from_utf8_lossy(&out.stderr).lines().last().unwrap_or(""));
            }
            rows.push(row);
        }
        let get = |r: &Vec<(String, f64)>, k: &str| r.iter().find(|(n, _)| n == k).map(|(_, v)| *v);
        let cols: [(&str, &str, usize); 17] = [
            ("units", "units", 6), ("lessons", "lessons", 8), ("teach_s", "teach s", 8), ("peak_mb", "peak MB", 8), ("known_ms", "known ms", 9), ("newfact_ms", "newfact ms", 11),
            ("retaught_ms", "retaught ms", 12), ("ask_us", "ask us", 8), ("ask_uj", "ask uJ", 8), ("reorg_s", "reorg s", 8), ("file_kb", "file KB", 9), ("agree", "mech %", 7),
            ("reworded_ok", "reword %", 9), ("never_ok", "never %", 8), ("route", "route %", 8), ("typo_wrong", "typo W%", 8), ("traps_answered", "trapsAns", 9),
        ];
        println!("\nTABLE");
        println!("{}", cols.iter().map(|(_, h, w)| format!("{:>w$}", h, w = *w)).collect::<Vec<_>>().join(" "));
        for r in &rows {
            println!("{}", cols.iter().map(|(k, _, w)| format!("{:>w$}", get(r, k).map_or("-".to_string(), |v| if v.abs() >= 100.0 { format!("{v:.0}") } else { format!("{v:.2}") }), w = *w)).collect::<Vec<_>>().join(" "));
        }
        println!("wrong columns: taught/reworded/never/beyond (WRONG %, must be 0 without noise):");
        for r in &rows {
            println!("  {:>5} units: {:.2} / {:.2} / {:.2} / {:.2}  after reorganising {:.2}", get(r, "units").unwrap_or(0.0), get(r, "taught_wrong").unwrap_or(0.0), get(r, "reworded_wrong").unwrap_or(0.0), get(r, "never_wrong").unwrap_or(0.0), get(r, "beyond_wrong").unwrap_or(0.0), get(r, "reorg_wrong").unwrap_or(f64::NAN));
        }
        return;
    }
    let c = scale::Config { units: a.get("units", 20), entities: a.get("entities", 1000), facts_per_unit: facts, noise, typos, digits: a.flag("digits"), seed };
    let row = scale::run(&c, !a.flag("no-reorg"), cap);
    if a.flag("row") {
        println!("ROW {}", row.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join(" "));
    }
}

/// `baby wonder`: where its lessons leave several explanations equally good, say which number it would like to be told about.
fn cmd_wonder(a: &Args) {
    let path = a.string("school", DEFAULT_SCHOOL);
    let school = school::School::load(&path).unwrap_or_else(|e| {
        eprintln!("{e}\n(teach one first: baby school)");
        std::process::exit(1);
    });
    let wonders = if a.flag("check") { school.check_myself(a.get("limit", 2)) } else { school.wonder() };
    if wonders.is_empty() {
        println!("Nothing I want to ask: where I can explain a unit there is one explanation, and where I cannot there is nothing to choose between.");
        return;
    }
    println!("I would like to be told:\n");
    for w in &wonders {
        println!("  unit '{}': {}", w.unit, w.question);
        println!("      why: {}", w.why);
    }
    println!("\nAnswer one with: baby learn \"<question>\" <answer> --unit <unit> --wondered");
}

fn cmd_cost(a: &Args) {
    let seed: u64 = a.get("seed", 1);
    let world = kinds::world(seed);
    let baseline = peak_memory_kb();
    println!("What does it cost to teach the baby? Measured on this laptop, one core, no GPU. (The CPU's energy counter is off limits here, so: time and memory.)\n");

    let t = std::time::Instant::now();
    let mut school = school::School::train(&world, seed, true);
    let teach = t.elapsed().as_secs_f64();
    let after_teach = peak_memory_kb();
    println!(
        "  teaching it from nothing: {} lessons in {} units, {:.2} s, peak memory {:.1} MB (the program alone: {:.1} MB)",
        world.train.len(), school.units.len(), teach, after_teach as f64 / 1024.0, baseline as f64 / 1024.0
    );

    // Lessons it can already work out: the surprise gate.
    let rule_units = [1usize, 2, 3, 4, 5, 6, 7, 12];
    let known: Vec<&kinds::Item> = world.new_items.iter().filter(|i| rule_units.contains(&i.unit)).take(60).collect();
    let before: usize = school.units.iter().map(|u| u.lessons()).sum();
    let t = std::time::Instant::now();
    let mut skipped = 0;
    for it in &known {
        if let Ok(r) = school.learn(&it.query, &it.answer, Some(kinds::UNITS[it.unit])) {
            skipped += (!r.surprised) as usize;
        }
    }
    let secs = t.elapsed().as_secs_f64();
    let kept: usize = school.units.iter().map(|u| u.lessons()).sum::<usize>() - before;
    println!(
        "  {} more lessons of kinds it already understands: {:.1} ms in all ({:.2} ms each); {} needed no learning at all, {} lessons kept",
        known.len(), secs * 1000.0, secs * 1000.0 / known.len().max(1) as f64, skipped, kept
    );

    // Surprising but consistent: a new way of saying something it already knows how to do.
    let t = std::time::Instant::now();
    let mut n_phr = 0;
    for (x, y) in [(0usize, 3usize), (1, 2), (2, 4), (3, 1), (1, 5), (4, 4), (2, 2), (3, 3)] {
        if school.learn(&format!("what is {} added to {}", data::NUM[x], data::NUM[y]), data::NUM[x + y], Some("sum")).is_ok() {
            n_phr += 1;
        }
    }
    let secs = t.elapsed().as_secs_f64();
    println!(
        "  {} lessons that say sums a new way (\"added to\"): {:.1} ms in all ({:.2} ms each); it now answers \"what is five added to three\" with {:?}",
        n_phr, secs * 1000.0, secs * 1000.0 / n_phr.max(1) as f64, school.ask("what is five added to three").answer
    );

    // A new kind of lesson, nobody says where it belongs.
    let nums = data::NUM;
    let t = std::time::Instant::now();
    let mut n_lessons = 0;
    for n in 0..=6usize {
        let _ = school.learn_anywhere(&format!("what is {} less than ten", nums[n]), nums[10 - n]);
        n_lessons += 1;
    }
    let secs = t.elapsed().as_secs_f64();
    let opened = school.units.last().map(|u| (u.name.clone(), u.chosen.name(), u.best_evidence()));
    println!(
        "  {} lessons of a kind it has never seen: {:.0} ms in all ({:.0} ms each); it opened unit {:?}",
        n_lessons, secs * 1000.0, secs * 1000.0 / n_lessons as f64, opened.map(|o| format!("{} (learns by {}, {:.0}% held back)", o.0, o.1, 100.0 * o.2))
    );

    let t = std::time::Instant::now();
    let r = school.reorganise();
    let rs = t.elapsed().as_secs_f64();
    let t = std::time::Instant::now();
    let c = school.compact();
    println!(
        "  going back over everything: reorganising ({} merges, {} splits) {:.2} s; forgetting what is explained ({} of {} lessons kept) {:.2} s",
        r.merged.len(), r.split.len(), rs, c.after, c.before, t.elapsed().as_secs_f64()
    );
    println!("\n  peak memory over all of it: {:.1} MB.", peak_memory_kb() as f64 / 1024.0);
    println!("\nA person could run this on a phone. What it does not tell you: how this grows when the lessons are millions of sentences instead of 800 tidy ones.");
}

fn main() {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let args = Args::parse(&raw);
    if let Some(bar) = args.opts.get("bar").and_then(|b| b.parse::<f32>().ok()) {
        mechanisms::fix_reliable(bar as f64);
    }
    match args.positional.first().map(String::as_str) {
        Some("train") => cmd_train(&args),
        Some("ask") => cmd_ask(&args),
        Some("teach") => cmd_teach(&args),
        Some("know") => cmd_know(&args),
        Some("kinds") => cmd_kinds(&args),
        Some("school") => cmd_school(&args),
        Some("learn") => cmd_learn(&args),
        Some("cost") => cmd_cost(&args),
        Some("wonder") => cmd_wonder(&args),
        Some("scale") => cmd_scale(&args),
        Some("invent") => {
            if args.flag("calibrate") {
                let seeds: u64 = args.get("seeds", 1);
                let runs: Vec<_> = (1..=seeds).map(invent::run_calibrate).collect();
                let (facts, refused, withheld): (usize, usize, usize) = runs.iter().fold((0, 0, 0), |a, r| (a.0 + r.facts, a.1 + r.refused, a.2 + r.withheld));
                let wrong_facts: usize = runs.iter().map(|r| r.wrong_facts).sum();
                let n: usize = runs.iter().map(|r| r.rows.len()).sum();
                println!("guess, be told, update: {seeds} seeds, {n} guesses made; besides: {facts} stated as fact (of which WRONG: {wrong_facts}), {refused} refused with no guess, {withheld} withheld because its guesses this far out had been wrong more often than right");
                println!("{:<14} {:>8} {:>16} {:>16}   {:>34}", "stretch", "guesses", "rate it stated", "actually right", "(stated, over the first 20 | the rest)");
                let names = ["under 0.25", "0.25 to 0.5", "0.5 to 1", "1 to 2", "2 and over"];
                for (bucket, name) in names.iter().enumerate() {
                    let rows: Vec<_> = runs.iter().flat_map(|r| r.rows.iter()).filter(|r| r.bucket == bucket).collect();
                    if rows.is_empty() {
                        println!("{name:<14} {:>8}", 0);
                        continue;
                    }
                    let stated = rows.iter().map(|r| r.stated as f64).sum::<f64>() / rows.len() as f64;
                    let right = rows.iter().filter(|r| r.right).count() as f64 / rows.len() as f64;
                    let early: Vec<_> = rows.iter().filter(|r| r.tried < 20).collect();
                    let early_stated = early.iter().map(|r| r.stated as f64).sum::<f64>() / early.len().max(1) as f64;
                    let early_right = early.iter().filter(|r| r.right).count() as f64 / early.len().max(1) as f64;
                    println!(
                        "{name:<14} {:>8} {:>15.1}% {:>15.1}%   early: said {:>5.1}% was right {:>5.1}% (n={})",
                        rows.len(), 100.0 * stated, 100.0 * right, 100.0 * early_stated, 100.0 * early_right, early.len()
                    );
                }
                return;
            }
            if args.flag("corrections") {
                let seeds: u64 = args.get("seeds", 1);
                let runs: Vec<_> = (1..=seeds).map(invent::run_corrections).collect();
                for (label, deceptive) in [("deceptive kinds (a rule that changes regime)", true), ("genuine kinds", false)] {
                    let mut t = invent::CorrectionTally::default();
                    for run in &runs {
                        let r = if deceptive { run.0 } else { run.1 };
                        t.kinds += r.kinds; t.wrong_before += r.wrong_before; t.still_wrong_guess += r.still_wrong_guess; t.still_wrong_fact += r.still_wrong_fact;
                        t.now_refused += r.now_refused; t.now_right_guess += r.now_right_guess; t.now_right_fact += r.now_right_fact; t.right_before += r.right_before;
                        t.kept_right_guess += r.kept_right_guess; t.kept_right_fact += r.kept_right_fact; t.lost_to_refusal += r.lost_to_refusal; t.broke += r.broke;
                    }
                    println!("\n{label}: {} kinds with a wrong guess, over {seeds} seeds. One correction each (the wrong guess nearest the taught range).", t.kinds);
                    println!("  the other {} wrong guesses of those kinds are now: still a wrong guess {}, still WRONG as a fact {}, refused {}, right as a guess {}, right as a fact {}",
                        t.wrong_before, t.still_wrong_guess, t.still_wrong_fact, t.now_refused, t.now_right_guess, t.now_right_fact);
                    println!("  the {} guesses that were right: still right as a guess {}, now right as a fact {}, now refused {}, BROKEN (wrong) {}",
                        t.right_before, t.kept_right_guess, t.kept_right_fact, t.lost_to_refusal, t.broke);
                }
                return;
            }
            if args.flag("guesses") {
                let seeds: u64 = args.get("seeds", 1);
                let mut rows = Vec::new();
                for seed in 1..=seeds {
                    rows.extend(invent::run_guesses(seed));
                }
                println!("labelled guesses from the careful baby on unseen questions, {seeds} seeds ({} guesses). stretch = how far past the taught range, in widths of that range", rows.len());
                println!("{:<14} {:>22} {:>22}", "stretch", "genuine kinds", "deceptive kinds");
                println!("{:<14} {:>22} {:>22}", "", "guesses  right", "guesses  right");
                let edges = [0.0f32, 0.25, 0.5, 1.0, 2.0, f32::MAX];
                for w in edges.windows(2) {
                    let cell = |dec: bool| {
                        let sel: Vec<_> = rows.iter().filter(|r| r.deceptive == dec && r.stretch >= w[0] && r.stretch < w[1]).collect();
                        let ok = sel.iter().filter(|r| r.right).count();
                        if sel.is_empty() { format!("{:>7} {:>9}", 0, "-") } else { format!("{:>7} {:>8.1}%", sel.len(), 100.0 * ok as f64 / sel.len() as f64) }
                    };
                    let label = if w[1] == f32::MAX { format!("{:.2} and over", w[0]) } else { format!("{:.2} to {:.2}", w[0], w[1]) };
                    println!("{label:<14} {:>22} {:>22}", cell(false), cell(true));
                }
                return;
            }
            if args.flag("careful") {
                let seeds: u64 = args.get("seeds", 1);
                let rounds: usize = args.get("rounds", 3);
                let per_unit: usize = args.get("limit", 2);
                let mut dec: Vec<invent::Tally3> = vec![Default::default(); rounds + 2];
                let mut gen: Vec<invent::Tally3> = vec![Default::default(); rounds + 2];
                let mut asked = 0;
                for seed in 1..=seeds {
                    let r = invent::run_careful(seed, rounds, per_unit);
                    for i in 0..rounds + 2 {
                        dec[i].right += r.deceptive[i].right; dec[i].refused += r.deceptive[i].refused; dec[i].wrong += r.deceptive[i].wrong;
                        gen[i].right += r.genuine[i].right; gen[i].refused += r.genuine[i].refused; gen[i].wrong += r.genuine[i].wrong;
                    }
                    asked += r.asked;
                }
                println!("unseen questions (never taught + beyond the taught numbers), {seeds} seeds; {asked} edge questions asked over {rounds} rounds ({per_unit} per unit per round)");
                println!("{:<28} {:>22} {:>22}", "policy", "deceptive kinds", "genuine kinds");
                println!("{:<28} {:>22} {:>22}", "", "right / refused / WRONG", "right / refused / WRONG");
                for i in 0..rounds + 2 {
                    let label = match i { 0 => "permissive (as before)".to_string(), 1 => "careful, nothing asked".to_string(), k => format!("careful, after {} round(s)", k - 1) };
                    let fmt = |t: &invent::Tally3| format!("{:>6} / {:>5} / {:>5}", t.right, t.refused, t.wrong);
                    println!("{label:<28} {:>22} {:>22}", fmt(&dec[i]), fmt(&gen[i]));
                }
                return;
            }
            if args.flag("check") {
                let seeds: u64 = args.get("seeds", 1);
                let limit: usize = args.get("limit", 2);
                let mut table: std::collections::BTreeMap<&'static str, (usize, usize, usize, f32, f32, usize, usize, f32, usize)> = std::collections::BTreeMap::new();
                for seed in args.get::<u64>("from", 1)..=seeds {
                    for r in invent::run_check(seed, limit) {
                        let e = table.entry(r.kind).or_default();
                        e.0 += 1;
                        e.1 += r.asked;
                        e.2 += r.refuted;
                        e.3 += r.before.0;
                        e.4 += r.after.0;
                        e.5 += r.before.1;
                        e.6 += r.after.1;
                        e.7 += r.settled.0;
                        e.8 += r.settled.1;
                    }
                }
                println!("each unit that holds ONE program asks up to {limit} questions about the numbers it was told least about; accuracy on unseen questions before and after");
                println!("{:<14} {:>6} {:>7} {:>9} {:>12} {:>12} {:>14} {:>13}", "kind", "units", "asked", "unconfirmed", "right before", "right after", "after wonder", "WRONG b -> a");
                let (mut tb, mut ta, mut ts, mut wb, mut wa, mut ws, mut n, mut asked, mut refuted) = (0.0, 0.0, 0.0, 0, 0, 0, 0, 0, 0);
                for (name, (units, a, r, b, af, wbk, wak, st, wst)) in &table {
                    println!("{name:<14} {units:>6} {a:>7} {r:>11} {:>11.1}% {:>11.1}% {:>13.1}% {:>5} -> {wak} -> {wst}", b / *units as f32, af / *units as f32, st / *units as f32, wbk);
                    tb += b; ta += af; ts += st; wb += wbk; wa += wak; ws += wst; n += units; asked += a; refuted += r;
                }
                println!("all: {n} units, {asked} questions, {refuted} not confirmed; right {:.1}% -> {:.1}% -> {:.1}% (after following up); WRONG answers {wb} -> {wa} -> {ws}", tb / n.max(1) as f32, ta / n.max(1) as f32, ts / n.max(1) as f32);
                return;
            }
            if args.flag("active") {
                let seeds: u64 = args.get("seeds", 1);
                let limit: usize = args.get("limit", 4);
                // per kind: seeds where it had a question (asking curve, random curve for the same seeds), seeds where it had none, wrong
                struct Row { role: invent::Role, asked: Vec<(Vec<f32>, Vec<f32>)>, silent: Vec<(Vec<f32>, Vec<f32>)>, wrong: usize }
                let mut table: std::collections::BTreeMap<&'static str, Row> = std::collections::BTreeMap::new();
                for seed in 1..=seeds {
                    for r in invent::run_active(seed, limit) {
                        let e = table.entry(r.kind).or_insert(Row { role: r.role.clone(), asked: Vec::new(), silent: Vec::new(), wrong: 0 });
                        if r.questions > 0 { e.asked.push((r.asking, r.random)) } else { e.silent.push((r.asking, r.random)) }
                        e.wrong += r.wrong;
                    }
                }
                let mean = |v: &Vec<(Vec<f32>, Vec<f32>)>, which: usize, k: usize| v.iter().map(|x| { let c = if which == 0 { &x.0 } else { &x.1 }; c[k.min(c.len() - 1)] }).sum::<f32>() / v.len().max(1) as f32;
                let row = |v: &Vec<(Vec<f32>, Vec<f32>)>, which: usize| (0..=limit).map(|k| format!("{:>5.1}", mean(v, which, k))).collect::<Vec<_>>().join(" ");
                println!("percent of unseen questions answered rightly after k questions; {limit} questions at most. A kind appears if it was not already perfect.");
                println!("{:<14} {:<13} {:<28} {:<28} {:<28} {:>5}", "kind", "role", "baby asks (k=0..)", "random numbers (same seeds)", "no question to ask (k=0..)", "WRONG");
                for (name, r) in &table {
                    let asked = if r.asked.is_empty() { "-".to_string() } else { format!("[{}] {}", r.asked.len(), row(&r.asked, 0)) };
                    let rand = if r.asked.is_empty() { "-".to_string() } else { format!("[{}] {}", r.asked.len(), row(&r.asked, 1)) };
                    let silent = if r.silent.is_empty() { "-".to_string() } else { format!("[{}] {}", r.silent.len(), row(&r.silent, 0)) };
                    println!("{name:<14} {:<13} {asked:<28} {rand:<28} {silent:<28} {:>5}", format!("{:?}", r.role).to_lowercase(), r.wrong);
                }
                return;
            }
            invent::STAGE2_FIRST.store(args.flag("stage2-first"), std::sync::atomic::Ordering::Relaxed);
            invent::NOISE_PERMILLE.store((args.get("noise", 0.0f32) * 1000.0).round() as usize, std::sync::atomic::Ordering::Relaxed);
            let seeds: u64 = args.get("seeds", 1);
            if seeds <= 1 {
                invent::run(args.get("seed", 1), false);
            } else {
                // many seeds: new random codes and a new split every time; how often does it fall short, or explain what has no explanation?
                let (mut solved, mut total, mut invented, mut unexpl, mut wrong, mut s2, mut s2t, mut dwrong) = (0, 0, 0, 0, 0, 0, 0, 0);
                let mut by_kind: std::collections::BTreeMap<&'static str, (invent::Role, usize)> = std::collections::BTreeMap::new();
                for seed in 1..=seeds {
                    let o = invent::run(seed, true);
                    solved += o.hidden_solved;
                    total += o.hidden_total;
                    for (name, role, ok) in &o.per_kind {
                        let e = by_kind.entry(name).or_insert((role.clone(), 0));
                        e.1 += *ok as usize;
                    }
                    s2 += o.stage2_solved;
                    s2t += o.stage2_total;
                    invented += o.invented_for_unexplainable;
                    unexpl += o.unexplainable_total;
                    wrong += o.wrong_total;
                    dwrong += o.deceptive_wrong;
                }
                for (name, (role, n)) in &by_kind {
                    if *role != invent::Role::Base {
                        println!("  {name:<14} {:<13} {n:>3} of {seeds}", format!("{role:?}").to_lowercase());
                    }
                }
                println!("{seeds} seeds: hidden kinds worked out {solved} of {total}; stage-2 kinds {s2} of {s2t}; unexplainable kinds that got a mechanism {invented} of {unexpl}; WRONG answers {wrong} (and {dwrong} on the deceptive kinds, which are answered permissively here)");
            }
        }
        Some("grade") => cmd_grade(&args),
        Some("storage") => cmd_storage(&args),
        Some("sleep") => cmd_sleep(&args),
        Some("discover") => cmd_discover(&args),
        Some("effort") => cmd_effort(&args),
        Some("rules") => cmd_rules(&args),
        Some("curious") => cmd_curious(&args),
        Some("eval") => cmd_eval(&args),
        Some("data") => cmd_data(&args),
        Some("life") => cmd_life(&args),
        Some("energy") => cmd_energy(&args),
        _ => {
            eprintln!("usage: baby <school|ask|learn|wonder|sleep|storage|cost|scale|invent|grade|discover|teach|effort|data|train|know|curious|rules|kinds|eval|life|energy> ...  (see the top of src/main.rs)");
            std::process::exit(2);
        }
    }
}
