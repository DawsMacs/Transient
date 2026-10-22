// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DawsMacs
//! The controlled comparison of Vision Entry 2: the baby against a matched attention transformer, on the same lessons, graded on the same
//! questions, priced in joules (counted, and measured when the kernel lets this user read the CPU's energy counter).
//!
//! What "matched" means here, stated plainly so the comparison cannot be quietly tilted: both read the same word lists from the same training
//! lessons; both are graded on the same four question sets (taught, reworded, never taught, beyond the taught numbers); the transformer's
//! vocabulary and answer words come only from the training lessons (it can say nothing it was never taught); every size is trained until it
//! answers every training lesson, or until the epoch budget runs out, with the same optimiser and no tuning per size. The baby keeps its own
//! way of saying "I don't know"; the transformer is graded twice, always answering and answering only when its probability is at least 0.9.

use crate::data::tokenize;
use crate::kinds::{Item, World};
use crate::ledger::{fmt_joules, EnergyModel, Ledger, Rapl};
use crate::school::School;
use crate::transformer::{TConfig, Transformer};
use std::collections::HashMap;
use std::time::Instant;

/// The set names, in the order of the baby's scorecard.
pub const SETS: [&str; 4] = ["taught", "reworded", "never taught", "beyond the taught range"];

#[derive(Clone, Copy, Default, Debug)]
pub struct Grade {
    pub right: usize,
    pub refused: usize,
    pub wrong: usize,
    /// of the refused, how many had a labelled guess that was right (the baby only)
    pub guess_right: usize,
    pub n: usize,
}

impl Grade {
    fn pct(&self, x: usize) -> f64 {
        100.0 * x as f64 / self.n.max(1) as f64
    }
}

/// Real energy, if the counter is readable: joules above idle for a closure, and its wall time. The counter is the whole CPU package, so
/// anything else running on the machine is counted too: the energy above idle is therefore SHARED BY CPU TIME (this process's CPU seconds over
/// the machine's busy CPU seconds during the run, which is about 1 on a quiet machine), and `run` also notes how much CPU time OTHER processes used meanwhile
/// (from /proc/stat against this process's own CPU time) and `worst_other_share` reports the worst case, so a contaminated reading can be seen.
pub struct Meter {
    rapl: Option<Rapl>,
    idle_w: f64,
    worst_other: std::cell::Cell<f64>,
}

/// (busy CPU seconds of the whole machine, this process's CPU seconds); a jiffy is 1/100 s.
fn cpu_seconds() -> Option<(f64, f64)> {
    let stat = std::fs::read_to_string("/proc/stat").ok()?;
    let nums: Vec<f64> = stat.lines().next()?.split_whitespace().skip(1).filter_map(|x| x.parse().ok()).collect();
    if nums.len() < 8 {
        return None;
    }
    let busy = nums.iter().sum::<f64>() - nums[3] - nums[4]; // all but idle and iowait
    let me = std::fs::read_to_string("/proc/self/stat").ok()?;
    let after = me.rsplit(')').next()?;
    let f: Vec<&str> = after.split_whitespace().collect();
    let (ut, st): (f64, f64) = (f.get(11)?.parse().ok()?, f.get(12)?.parse().ok()?);
    Some((busy / 100.0, (ut + st) / 100.0))
}

impl Meter {
    pub fn new() -> Meter {
        let rapl = Rapl::open();
        let idle_w = match &rapl {
            Some(r) => {
                // five 1-second readings, the lowest: the idle baseline is what the machine draws when nothing else is going on
                let mut best = f64::MAX;
                for _ in 0..5 {
                    let i0 = r.read_uj().unwrap_or(0);
                    std::thread::sleep(std::time::Duration::from_secs(1));
                    best = best.min(r.delta_j(i0, r.read_uj().unwrap_or(0)));
                }
                best
            }
            None => 0.0,
        };
        Meter { rapl, idle_w, worst_other: std::cell::Cell::new(0.0) }
    }

    pub fn available(&self) -> bool {
        self.rapl.is_some()
    }

    /// The largest share of the machine's busy CPU time that went to OTHER processes during any measurement so far (0 to 1).
    pub fn worst_other_share(&self) -> f64 {
        self.worst_other.get()
    }

    /// Run `f`; return its value, the seconds it took, and the joules above idle (None if the counter cannot be read).
    pub fn run<R>(&self, f: impl FnOnce() -> R) -> (R, f64, Option<f64>) {
        let e0 = self.rapl.as_ref().and_then(|r| r.read_uj());
        let c0 = cpu_seconds();
        let t = Instant::now();
        let out = f();
        let secs = t.elapsed().as_secs_f64();
        let e1 = self.rapl.as_ref().and_then(|r| r.read_uj());
        if let (Some((b0, m0)), Some((b1, m1))) = (c0, cpu_seconds()) {
            let busy = b1 - b0;
            if busy > 0.05 {
                let other = ((busy - (m1 - m0)) / busy).clamp(0.0, 1.0);
                if secs > 0.1 && other > self.worst_other.get() {
                    self.worst_other.set(other);
                }
            }
        }
        // the package counter includes whatever else was running: share the energy above idle by CPU time (this process's share of the busy time)
        let share = match (c0, cpu_seconds()) {
            (Some((b0, m0)), Some((b1, m1))) if b1 - b0 > 0.05 => ((m1 - m0) / (b1 - b0)).clamp(0.0, 1.0),
            _ => 1.0,
        };
        let above = match (&self.rapl, e0, e1) {
            (Some(r), Some(a), Some(b)) => Some((r.delta_j(a, b) - self.idle_w * secs).max(0.0) * share),
            _ => None,
        };
        (out, secs, above)
    }

    /// A line to print at the end of an experiment if other work on the machine may have inflated the energy readings.
    pub fn warning(&self) -> Option<String> {
        let w = self.worst_other_share();
        (self.available() && w > 0.15).then(|| format!("WARNING: other processes used up to {:.0}% of the busy CPU time during some measurements, so those energies are attributed by CPU-time share (an approximation); for exact figures rerun on a quiet machine.", 100.0 * w))
    }
}

/// Words and answers a transformer can use: only those in the training lessons, plus spare slots reserved for what it will be taught later
/// (a network of fixed size cannot even represent a word it has no row for; the baby has no such limit, so the reserve is generous to the
/// network, not to the baby).
#[derive(Clone)]
pub struct Lexicon {
    words: HashMap<String, usize>,
    pub classes: Vec<String>,
    pub longest: usize,
    max_vocab: usize,
    max_classes: usize,
}

impl Lexicon {
    pub fn from(train: &[Item]) -> Lexicon {
        Lexicon::with_reserve(train, 0, 0)
    }

    pub fn with_reserve(train: &[Item], spare_words: usize, spare_classes: usize) -> Lexicon {
        let mut lex = Lexicon { words: HashMap::new(), classes: Vec::new(), longest: 1, max_vocab: usize::MAX, max_classes: usize::MAX };
        lex.extend(train);
        lex.max_vocab = lex.words.len() + 1 + spare_words;
        lex.max_classes = lex.classes.len() + spare_classes;
        lex
    }

    /// Give every new word and answer of these lessons a row (panics if the reserve is used up: a bug in the experiment, not a result).
    pub fn extend(&mut self, items: &[Item]) {
        for it in items {
            let toks = tokenize(&it.query);
            self.longest = self.longest.max(toks.len());
            for w in toks {
                let next = self.words.len() + 1;
                self.words.entry(w).or_insert(next);
            }
            if !self.classes.contains(&it.answer) {
                self.classes.push(it.answer.clone());
            }
        }
        assert!(self.words.len() + 1 <= self.max_vocab && self.classes.len() <= self.max_classes, "the reserve of spare words or answers is used up");
    }

    pub fn vocab(&self) -> usize {
        self.max_vocab
    }

    pub fn classes_capacity(&self) -> usize {
        self.max_classes
    }

    pub fn encode(&self, q: &str) -> Vec<usize> {
        tokenize(q).iter().map(|w| self.words.get(w).copied().unwrap_or(0)).collect()
    }

    pub fn class(&self, a: &str) -> Option<usize> {
        self.classes.iter().position(|c| c == a)
    }
}

pub fn lessons(lex: &Lexicon, items: &[Item]) -> Vec<(Vec<usize>, usize)> {
    items.iter().filter_map(|i| lex.class(&i.answer).map(|c| (lex.encode(&i.query), c))).collect()
}

/// Grade a transformer: with no threshold it always answers; with one it refuses below that probability.
pub fn grade_transformer(t: &Transformer, lex: &Lexicon, items: &[Item], threshold: f32) -> Grade {
    let mut g = Grade { n: items.len(), ..Grade::default() };
    let mut scratch = Ledger::default();
    for it in items {
        let (best, p) = t.ask(&lex.encode(&it.query), &mut scratch);
        if p < threshold {
            g.refused += 1;
        } else if lex.classes.get(best).map(|c| c.as_str()) == Some(it.answer.as_str()) {
            g.right += 1;
        } else {
            g.wrong += 1;
        }
    }
    g
}

pub fn grade_baby(school: &School, items: &[Item]) -> Grade {
    let mut g = Grade { n: items.len(), ..Grade::default() };
    for it in items {
        let r = school.ask(&it.query);
        match (&r.answer, &r.guess) {
            (Some(a), _) if *a == it.answer => g.right += 1,
            (Some(_), _) => g.wrong += 1,
            (None, guess) => {
                g.refused += 1;
                if guess.as_ref().map_or(false, |x| x.answer == it.answer) {
                    g.guess_right += 1;
                }
            }
        }
    }
    g
}

pub fn sets(w: &World) -> [&Vec<Item>; 4] {
    [&w.train, &w.reworded, &w.new_items, &w.beyond]
}

pub struct Trained {
    pub name: String,
    pub params: usize,
    pub epochs: usize,
    pub secs: f64,
    pub ledger: Ledger,
    pub measured_j: Option<f64>,
    pub model: Transformer,
}

/// Train one transformer size on the training lessons.
pub fn train_transformer(lex: &Lexicon, train: &[Item], d: usize, layers: usize, epochs: usize, seed: u64, meter: &Meter) -> Trained {
    let cfg = TConfig { vocab: lex.vocab(), classes: lex.classes_capacity(), d, heads: if d >= 64 { 4 } else { 2 }, layers, ff: 2 * d, len: lex.longest };
    let mut model = Transformer::new(cfg, seed);
    let data = lessons(lex, train);
    let mut ledger = Ledger::default();
    let (used, secs, joules) = meter.run(|| model.train(&data, epochs, 16, 0.003, seed, &mut ledger));
    Trained { name: format!("transformer d={d} x{layers}"), params: cfg.n_params(), epochs: used, secs, ledger, measured_j: joules, model }
}

fn fmt_opt_j(j: Option<f64>) -> String {
    j.map_or("n/a".to_string(), fmt_joules)
}

pub fn run(seed: u64, sizes: &[usize], layers: usize, epochs: usize) {
    let world = crate::kinds::world(seed);
    let meter = Meter::new();
    let em = EnergyModel::default();
    println!(
        "Same lessons for everyone: {} taught, graded on {} reworded, {} never taught, {} beyond the taught numbers.",
        world.train.len(),
        world.reworded.len(),
        world.new_items.len(),
        world.beyond.len()
    );
    if meter.available() {
        println!("Energy is MEASURED (CPU package counter, above idle) and also counted and priced with 45 nm figures (an estimate).");
    } else {
        println!("Energy is only counted and priced with 45 nm figures (an estimate): the CPU counter is not readable by this user.");
    }
    println!();

    // the baby
    let (school, secs, joules) = meter.run(|| School::train(&world, seed, true));
    let file_kb = {
        let tmp = std::env::temp_dir().join(format!("compare_{}.school", std::process::id()));
        let path = tmp.to_string_lossy().to_string();
        let kb = school.save(&path).ok().and_then(|_| std::fs::metadata(&path).ok()).map_or(0, |m| m.len() / 1024);
        std::fs::remove_file(&path).ok();
        kb
    };
    let baby: Vec<Grade> = sets(&world).iter().map(|s| grade_baby(&school, s)).collect();

    // the transformers
    let lex = Lexicon::from(&world.train);
    let trained: Vec<Trained> = sizes.iter().map(|&d| train_transformer(&lex, &world.train, d, layers, epochs, seed, &meter)).collect();

    println!("{:<22} {:>9} {:>8} {:>7} {:>14} {:>12} {:>12}", "", "size", "epochs", "time", "counted MACs", "est. energy", "MEASURED");
    println!(
        "{:<22} {:>9} {:>8} {:>6.2}s {:>14} {:>12} {:>12}",
        "baby (school)",
        format!("{file_kb} KB file"),
        "-",
        secs,
        "(not counted)",
        "-",
        fmt_opt_j(joules)
    );
    for t in &trained {
        println!(
            "{:<22} {:>9} {:>8} {:>6.2}s {:>14} {:>12} {:>12}",
            t.name,
            format!("{} params", t.params),
            t.epochs,
            t.secs,
            t.ledger.macs,
            fmt_joules(em.joules(&t.ledger)),
            fmt_opt_j(t.measured_j)
        );
    }

    println!("\nGraded on the same questions: right / refused / WRONG (percent of the set)\n");
    print!("{:<34}", "");
    for s in SETS {
        print!(" {:>24}", s);
    }
    println!();
    let row = |name: &str, g: &[Grade]| {
        print!("{name:<34}");
        for x in g {
            print!(" {:>24}", format!("{:.1} / {:.1} / {:.1}", x.pct(x.right), x.pct(x.refused), x.pct(x.wrong)));
        }
        println!();
    };
    row("baby", &baby);
    let guess_row: Vec<String> = baby.iter().map(|g| format!("{:.1}", g.pct(g.guess_right))).collect();
    println!("{:<34} {}", "  (of its refusals, a right guess)", guess_row.iter().map(|g| format!("{:>24}", g)).collect::<Vec<_>>().join(" "));
    for t in &trained {
        let always: Vec<Grade> = sets(&world).iter().map(|s| grade_transformer(&t.model, &lex, s, 0.0)).collect();
        row(&format!("{} (always answers)", t.name), &always);
        let careful: Vec<Grade> = sets(&world).iter().map(|s| grade_transformer(&t.model, &lex, s, 0.9)).collect();
        row(&format!("{} (answers if p >= 0.9)", t.name), &careful);
    }
    if let Some(w) = meter.warning() {
        println!("\n{w}");
    }
}

fn fmt_row_grades(label: &str, new: Option<&Grade>, old: &[Grade]) {
    let cell = |g: &Grade| format!("{:.1} / {:.1} / {:.1}", g.pct(g.right), g.pct(g.refused), g.pct(g.wrong));
    print!("{label:<40}");
    match new {
        Some(g) => print!(" {:>20}", cell(g)),
        None => print!(" {:>20}", "-"),
    }
    for g in old {
        print!(" {:>20}", cell(g));
    }
    println!();
}

/// Learning something new: `n_new` invented capitals are taught (three phrasings each) and asked in a fourth. What does it cost, and what
/// happens to what was already known?
pub fn run_adapt(seed: u64, sizes: &[usize], layers: usize, epochs: usize, n_new: usize) {
    let world = crate::kinds::world(seed);
    let meter = Meter::new();
    let em = EnergyModel::default();
    let (teach_new, ask_new) = crate::kinds::new_capital_facts(seed, n_new);
    println!(
        "Learning something NEW: {} invented capitals ({} lessons, three phrasings each), asked in a fourth phrasing; then the old knowledge is re-graded.",
        n_new,
        teach_new.len()
    );
    println!("{}\n", if meter.available() { "Energy is MEASURED (CPU package, above idle) and counted (45 nm estimate)." } else { "Energy is counted only (the CPU counter is not readable)." });

    // the baby
    let mut school = School::train(&world, seed, true);
    let old_sets = [&world.train, &world.reworded, &world.new_items];
    let before: Vec<Grade> = old_sets.iter().map(|s| grade_baby(&school, s)).collect();
    let ((), secs, joules) = meter.run(|| {
        for it in &teach_new {
            let _ = school.learn(&it.query, &it.answer, Some("capital"));
        }
    });
    let after: Vec<Grade> = old_sets.iter().map(|s| grade_baby(&school, s)).collect();
    let new_grade = grade_baby(&school, &ask_new);

    println!("{:<34} {:>7} {:>9} {:>12} {:>12} {:>12}", "", "steps", "time", "counted MACs", "est. energy", "MEASURED");
    println!("{:<34} {:>7} {:>8.3}s {:>12} {:>12} {:>12}", "baby: learn the new lessons", teach_new.len(), secs, "(not counted)", "-", fmt_opt_j(joules));

    let lex0 = Lexicon::with_reserve(&world.train, 2 * n_new, n_new);
    let mut rows: Vec<(String, Grade, Vec<Grade>)> = Vec::new();
    let mut lines: Vec<String> = Vec::new();
    for &d in sizes {
        let base = train_transformer(&lex0, &world.train, d, layers, epochs, seed, &meter);
        let mut lex = lex0.clone();
        lex.extend(&teach_new);
        let grade_all = |m: &Transformer| -> (Grade, Vec<Grade>) {
            (grade_transformer(m, &lex, &ask_new, 0.0), old_sets.iter().map(|s| grade_transformer(m, &lex, s, 0.0)).collect())
        };
        let (g_before, o_before) = grade_all(&base.model);
        rows.push((format!("{} before (never saw them)", base.name), g_before, o_before));
        for (label, share) in [("fine-tune on the new only", 0.0f32), ("fine-tune on new + 10% of the old", 0.1), ("fine-tune on new + all old", 1.0)] {
            let mut m = base.model.clone();
            let mut data = lessons(&lex, &teach_new);
            if share > 0.0 {
                let mut old = lessons(&lex, &world.train);
                crate::rng::Rng::new(seed ^ 0x5EED).shuffle(&mut old);
                old.truncate(((old.len() as f32) * share).round() as usize);
                data.extend(old);
            }
            let mut ledger = Ledger::default();
            let (used, secs, joules) = meter.run(|| m.train(&data, epochs, 16, 0.003, seed, &mut ledger));
            lines.push(format!(
                "{:<34} {:>7} {:>8.3}s {:>12} {:>12} {:>12}",
                format!("{} d={d}: {label}", "transformer"),
                format!("{used} ep"),
                secs,
                ledger.macs,
                fmt_joules(em.joules(&ledger)),
                fmt_opt_j(joules)
            ));
            let (gn, go) = grade_all(&m);
            rows.push((format!("transformer d={d}: {label}"), gn, go));
        }
    }
    for l in &lines {
        println!("{l}");
    }
    println!("\nright / refused / WRONG, percent of each set (transformers always answer)\n");
    print!("{:<40} {:>20}", "", "the new capitals");
    for s in ["taught", "reworded", "never taught"] {
        print!(" {:>20}", s);
    }
    println!();
    if let Some(w) = meter.warning() {
        println!("{w}\n");
    }
    fmt_row_grades("baby before", None, &before);
    fmt_row_grades("baby after learning", Some(&new_grade), &after);
    for (label, gn, go) in &rows {
        fmt_row_grades(label, Some(gn), go);
    }
}

/// How do the two grow? The same comparison on worlds of more and more units (the scale generator), trained from scratch at each size.
pub fn run_scale(seed: u64, counts: &[usize], sizes: &[usize], layers: usize, epochs: usize) {
    use crate::scale::{generate, Config};
    let meter = Meter::new();
    let em = EnergyModel::default();
    println!("{}\n", if meter.available() { "Energy is MEASURED (CPU package, above idle) and counted (45 nm estimate)." } else { "Energy is counted only." });
    println!("right / refused / WRONG, percent of each set; for the baby, in brackets, the share of its refusals that came with a right labelled guess\n");
    println!(
        "{:>6} {:>8}  {:<27} {:>8} {:>6} {:>11} {:>10}   {:>18} {:>19} {:>27}",
        "units", "lessons", "learner", "time", "epochs", "est. energy", "MEASURED", "reworded", "never taught", "beyond the taught numbers"
    );
    for &units in counts {
        let sw = generate(&Config { units, entities: (units * 20).clamp(1000, 10_000), facts_per_unit: 12, noise: 0.0, typos: 0.0, digits: false, seed });
        let w = &sw.world;
        let (school, secs, joules) = meter.run(|| School::from_items(&w.train, sw.names.clone(), Some(&w.reworded), seed, true));
        let (gr, gn, gb) = (grade_baby(&school, &w.reworded), grade_baby(&school, &w.new_items), grade_baby(&school, &w.beyond));
        let cell = |g: &Grade| format!("{:.1} / {:.1} / {:.1}", g.pct(g.right), g.pct(g.refused), g.pct(g.wrong));
        let cell_baby = |g: &Grade| format!("{} ({:.0})", cell(g), g.pct(g.guess_right) * 100.0 / g.pct(g.refused).max(0.0001));
        println!(
            "{:>6} {:>8}  {:<27} {:>7.2}s {:>6} {:>11} {:>10}   {:>18} {:>19} {:>27}",
            units, w.train.len(), "baby", secs, "-", "-", fmt_opt_j(joules), cell(&gr), cell(&gn), cell_baby(&gb)
        );
        let lex = Lexicon::from(&w.train);
        for &d in sizes {
            let t = train_transformer(&lex, &w.train, d, layers, epochs, seed, &meter);
            let g = |items: &[Item]| grade_transformer(&t.model, &lex, items, 0.0);
            println!(
                "{:>6} {:>8}  {:<27} {:>7.2}s {:>6} {:>11} {:>10}   {:>18} {:>19} {:>27}",
                "", "", format!("{} ({}k params)", t.name, t.params / 1000), t.secs, t.epochs, fmt_joules(em.joules(&t.ledger)), fmt_opt_j(t.measured_j),
                cell(&g(&w.reworded)), cell(&g(&w.new_items)), cell(&g(&w.beyond))
            );
        }
    }
}

/// Write the four sets of a world as TSV (query, answer, unit, fact) so another implementation can be trained and graded on exactly the same
/// lessons and questions: `train.tsv`, `reworded.tsv`, `never.tsv`, `beyond.tsv`. With `units > 0` the world is the scale generator's.
pub fn export(seed: u64, units: usize, dir: &str) -> std::io::Result<()> {
    let world = if units > 0 {
        let c = crate::scale::Config { units, entities: (units * 20).clamp(1000, 10_000), facts_per_unit: 12, noise: 0.0, typos: 0.0, digits: false, seed };
        crate::scale::generate(&c).world
    } else {
        crate::kinds::world(seed)
    };
    std::fs::create_dir_all(dir)?;
    for (name, items) in [("train", &world.train), ("reworded", &world.reworded), ("never", &world.new_items), ("beyond", &world.beyond)] {
        let mut out = String::new();
        for it in items.iter() {
            out.push_str(&format!("{}\t{}\t{}\t{}\n", it.query, it.answer, it.unit, it.fact));
        }
        std::fs::write(format!("{dir}/{name}.tsv"), out)?;
    }
    println!("wrote {} / {} / {} / {} lessons and questions to {dir}/{{train,reworded,never,beyond}}.tsv", world.train.len(), world.reworded.len(), world.new_items.len(), world.beyond.len());
    Ok(())
}
