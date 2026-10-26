// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DawsMacs
//! Does learning get cheaper as more is known? Teach the kinds of an adaptive exam in batches, in the order of their roles (base, hidden, stage 2,
//! deceptive, no rule), and measure what each batch costs once; then teach each batch to a system that knows nothing, as the control.
//!
//!     cargo run --release --offline --example curriculum -- EXAM_DIR [REPEATS] [KINDS_PER_BATCH]
//!
//! EXAM_DIR is a directory written by `baby exam adapt-write`. The exam's unit labels are used only to cut the lessons into batches: they are
//! never given to Novice. Work is counted in evaluations; energy is the CPU package's counter above the idle draw (Linux with a readable counter),
//! the median of REPEATS runs. Evaluations are exact and repeatable; joules are an upper bound.

use novice::learner::Novice;
use novice::protocol::System;
use std::collections::BTreeMap;
use std::time::Instant;

const ENERGY: &str = "/sys/class/powercap/intel-rapl:0/energy_uj";

fn energy_uj() -> Option<u64> {
    std::fs::read_to_string(ENERGY).ok()?.trim().parse().ok()
}

/// Watts the machine draws doing nothing, measured over a second.
fn idle_watts() -> Option<f64> {
    let (e0, t0) = (energy_uj()?, Instant::now());
    std::thread::sleep(std::time::Duration::from_millis(1000));
    let (e1, t1) = (energy_uj()?, Instant::now());
    Some((e1.checked_sub(e0)? as f64 / 1e6) / (t1 - t0).as_secs_f64())
}

struct Batch {
    roles: String,
    kinds: usize,
    lessons: Vec<(String, String)>,
}

#[derive(Clone, Copy, Default)]
struct Cost {
    evaluations: u64,
    seconds: f64,
    joules: Option<f64>,
    explained: usize,
    found: usize,
}

fn teach(n: &mut Novice, batch: &Batch, idle: Option<f64>) -> Cost {
    let (kinds_before, explained_before) = n.kinds();
    let before = n.stats.evaluations;
    let (e0, t0) = (energy_uj(), Instant::now());
    for (q, a) in &batch.lessons {
        n.teach(q, a);
    }
    n.ready();
    let (e1, t1) = (energy_uj(), Instant::now());
    let seconds = (t1 - t0).as_secs_f64();
    let joules = match (e0, e1, idle) {
        (Some(a), Some(b), Some(w)) if b >= a => Some(((b - a) as f64 / 1e6 - w * seconds).max(0.0)),
        _ => None,
    };
    let (kinds, explained) = n.kinds();
    Cost { evaluations: n.stats.evaluations - before, seconds, joules, explained: explained - explained_before, found: kinds - kinds_before }
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let dir = args.first().expect("usage: curriculum EXAM_DIR [REPEATS] [KINDS_PER_BATCH]");
    let repeats: usize = args.get(1).and_then(|a| a.parse().ok()).unwrap_or(5);
    let per_batch: usize = args.get(2).and_then(|a| a.parse().ok()).unwrap_or(5);

    // The lessons by unit, and each unit's role from the stream.
    let mut role_of: BTreeMap<String, String> = BTreeMap::new();
    for line in std::fs::read_to_string(format!("{dir}/stream.tsv")).expect("stream.tsv").lines() {
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() >= 3 {
            role_of.entry(f[2].to_string()).or_insert_with(|| f[1].to_string());
        }
    }
    let mut units: Vec<(String, Vec<(String, String)>)> = Vec::new();
    for line in std::fs::read_to_string(format!("{dir}/lessons.tsv")).expect("lessons.tsv").lines() {
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() < 4 {
            continue;
        }
        match units.iter_mut().find(|u| u.0 == f[3]) {
            Some(u) => u.1.push((f[1].to_string(), f[2].to_string())),
            None => units.push((f[3].to_string(), vec![(f[1].to_string(), f[2].to_string())])),
        }
    }
    let order = ["base", "hidden", "stage2", "deceptive", "unexplainable"];
    units.sort_by_key(|u| order.iter().position(|r| *r == role_of.get(&u.0).map(String::as_str).unwrap_or("")).unwrap_or(order.len()));
    let batches: Vec<Batch> = units
        .chunks(per_batch)
        .map(|c| {
            let mut roles: Vec<&str> = c.iter().map(|u| role_of.get(&u.0).map(String::as_str).unwrap_or("?")).collect();
            roles.dedup();
            Batch { roles: roles.join("+"), kinds: c.len(), lessons: c.iter().flat_map(|u| u.1.clone()).collect() }
        })
        .collect();

    let idle = idle_watts();
    match idle {
        Some(w) => eprintln!("idle draw {w:.2} W; {} batches of up to {per_batch} kinds; median of {repeats} runs", batches.len()),
        None => eprintln!("no readable energy counter: evaluations and time only"),
    }

    // Curriculum: one system taught batch after batch. Control: a new system for every batch.
    let mut chain: Vec<Vec<Cost>> = vec![Vec::new(); batches.len()];
    let mut fresh: Vec<Vec<Cost>> = vec![Vec::new(); batches.len()];
    for _ in 0..repeats {
        let mut n = Novice::default();
        for (i, b) in batches.iter().enumerate() {
            chain[i].push(teach(&mut n, b, idle));
            fresh[i].push(teach(&mut Novice::default(), b, idle));
        }
    }

    println!("{:>5} {:<22} {:>5} | {:>9} {:>9} {:>7} | {:>9} {:>9} {:>7} | {:>8} {:>8}", "batch", "roles", "kinds", "evals", "ms", "mJ", "evals", "ms", "mJ", "explained", "explained");
    println!("{:>5} {:<22} {:>5} | {:^27} | {:^27} | {:>8} {:>8}", "", "", "", "after the batches before", "knowing nothing", "(after)", "(alone)");
    let (mut sum_chain, mut sum_fresh) = (0.0, 0.0);
    for i in 0..batches.len() {
        let col = |v: &Vec<Cost>, f: &dyn Fn(&Cost) -> f64| median(v.iter().map(f).collect());
        let (ce, cs) = (col(&chain[i], &|c| c.evaluations as f64), col(&chain[i], &|c| c.seconds * 1000.0));
        let (fe, fs) = (col(&fresh[i], &|c| c.evaluations as f64), col(&fresh[i], &|c| c.seconds * 1000.0));
        let cj = chain[i][0].joules.map(|_| col(&chain[i], &|c| c.joules.unwrap_or(0.0) * 1000.0));
        let fj = fresh[i][0].joules.map(|_| col(&fresh[i], &|c| c.joules.unwrap_or(0.0) * 1000.0));
        let show = |x: Option<f64>| x.map_or("n/a".to_string(), |v| format!("{v:.0}"));
        println!(
            "{:>5} {:<22} {:>5} | {:>9.0} {:>9.1} {:>7} | {:>9.0} {:>9.1} {:>7} | {:>4}/{:<3} {:>4}/{:<3}",
            i + 1,
            batches[i].roles,
            batches[i].kinds,
            ce,
            cs,
            show(cj),
            fe,
            fs,
            show(fj),
            chain[i][0].explained,
            chain[i][0].found,
            fresh[i][0].explained,
            fresh[i][0].found
        );
        sum_chain += ce;
        sum_fresh += fe;
    }
    println!("total evaluations: {sum_chain:.0} one system taught in order, {sum_fresh:.0} each batch taught to a system that knows nothing");
}
