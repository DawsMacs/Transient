// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DawsMacs
//! A fingerprint of this platform's math library, on the inputs Transient actually uses.
//!
//! Rust's `exp`, `ln`, `log2`, `tanh` and `powi` are documented as having unspecified precision: the last bit may differ between
//! platforms, C libraries and Rust versions. `+ - * /` and `sqrt` are exact everywhere. This program evaluates the platform-
//! dependent functions over the ranges the code reaches and prints a 64-bit hash of every result's bits (plus one hash per block of
//! 262,144 inputs, to say where a difference starts). Two platforms whose output matches have the same last bits on all of those
//! inputs. It touches none of Transient's code; it only calls the same standard-library functions the same way.
//!
//! Usage:
//!   cargo run --release --offline --example mathprint                           print the fingerprint
//!   cargo run --release --offline --example mathprint -- dump NAME FROM COUNT   list inputs and result bits (to look at a block)
//!   cargo run --release --offline --example mathprint -- audit                  how far each result is from the correctly rounded one
//!
//! The audit compares each f32 result with the f64 result of the same function rounded to f32. The f64 result is accurate to far
//! below half an f32 step, so the rounded value is the correctly rounded f32 result except for inputs within about one part in a
//! billion of a rounding boundary. It needs no stored reference: it says how often, and by how many steps, THIS platform is off.
//! (`powi` is by design not correctly rounded: it multiplies repeatedly, so its error grows with the exponent.)
//!
//! Lines beginning with `#` describe the platform and are not part of the comparison.

use std::hint::black_box;

const BLOCK: u64 = 262_144;

struct Section {
    name: &'static str,
    note: &'static str,
    len: u64,
    f: fn(u64) -> (f32, f32),
    /// The same function in f64, applied to the input that `f` reports.
    reference: fn(f32) -> f64,
}

/// The inverse-document-frequency weight (`model.rs`, `mechanisms.rs`, `life.rs`, `curious.rs`): ln((1 + heard) / (1 + df)).
fn idf(i: u64) -> (f32, f32) {
    let (heard, df) = (black_box((i / 5001) as u32), black_box((i % 5001) as u32));
    let x = ((1 + heard) as f32) / ((1 + df) as f32);
    (x, x.ln())
}

/// As `idf`, for the larger worlds of the scale tests (tens of thousands of lessons): `heard` from 5,001 to about 50,000 in steps of 83, and for each
/// of them 1,000 values of `df` spread evenly from 0 to `heard`.
fn idf_wide(i: u64) -> (f32, f32) {
    let (a, b) = (i / 1000, i % 1000);
    let heard = black_box((5001 + 83 * a) as u32);
    let df = black_box(((heard as u64 * b) / 999) as u32);
    let x = ((1 + heard) as f32) / ((1 + df) as f32);
    (x, x.ln())
}

/// The routing gain (`school.rs`): ln(1 + 2 * count).
fn route_gain(i: u64) -> (f32, f32) {
    let x = 1.0 + 2.0 * black_box(i as u32) as f32;
    (x, x.ln())
}

/// The routing prior (`school.rs`): ln(n + half_vocabulary), half_vocabulary = 1 + rows / 100.
fn route_prior(i: u64) -> (f32, f32) {
    let (rows, n) = (black_box((i / 3201) as u32), black_box((i % 3201) as u32));
    let x = n as f32 + (1.0 + rows as f32 / 100.0);
    (x, x.ln())
}

/// The believability test (`program.rs`): log2 of counts and of the work done, up to the search budget (1,000,000) and a little over.
fn log2_int(i: u64) -> (f32, f32) {
    let x = black_box((i + 1) as u32) as f32;
    (x, x.log2())
}

/// The softmax numerator (`net.rs`, `transformer.rs`): exp(p - max), always at or below zero, down to where it underflows.
fn exp_neg(i: u64) -> (f32, f32) {
    let x = -(black_box(i as u32) as f32) / 1024.0;
    (x, x.exp())
}

/// The hidden layer (`net.rs`): tanh over the range where it is not already 1.
fn tanh_grid(i: u64) -> (f32, f32) {
    let x = (black_box(i as i32) - 12 * 1024) as f32 / 1024.0;
    (x, x.tanh())
}

/// The loss (`net.rs`, `transformer.rs`): ln of a probability, floored at 1e-12.
fn ln_prob(i: u64) -> (f32, f32) {
    let x = (black_box((i + 1) as u32) as f32 / 100_000.0).max(1e-12);
    (x, x.ln())
}

/// The optimizer's bias correction (`net.rs`): 0.9^t, for t steps.
fn powi_b1(i: u64) -> (f32, f32) {
    let t = black_box((i + 1) as i32);
    (t as f32, 0.9f32.powi(t))
}

/// As above, for 0.999^t.
fn powi_b2(i: u64) -> (f32, f32) {
    let t = black_box((i + 1) as i32);
    (t as f32, 0.999f32.powi(t))
}

fn ref_ln(x: f32) -> f64 {
    (x as f64).ln()
}
fn ref_log2(x: f32) -> f64 {
    (x as f64).log2()
}
fn ref_exp(x: f32) -> f64 {
    (x as f64).exp()
}
fn ref_tanh(x: f32) -> f64 {
    (x as f64).tanh()
}
/// For `powi` the input reported is the exponent; the reference uses `powf`, which does not multiply repeatedly.
fn ref_b1(t: f32) -> f64 {
    (0.9f32 as f64).powf(t as f64)
}
fn ref_b2(t: f32) -> f64 {
    (0.999f32 as f64).powf(t as f64)
}

const SECTIONS: &[Section] = &[
    Section { name: "idf", note: "ln((1+heard)/(1+df))", len: 5001 * 5001, f: idf, reference: ref_ln },
    Section { name: "idf_wide", note: "ln((1+heard)/(1+df)), heard to 50,000 (a grid)", len: 1000 * 542, f: idf_wide, reference: ref_ln },
    Section { name: "route_gain", note: "ln(1+2*count)", len: 200_001, f: route_gain, reference: ref_ln },
    Section { name: "route_prior", note: "ln(n+1+rows/100)", len: 10001 * 3201, f: route_prior, reference: ref_ln },
    Section { name: "log2_int", note: "log2(1..=2000000)", len: 2_000_000, f: log2_int, reference: ref_log2 },
    Section { name: "exp_neg", note: "exp(-k/1024), k to 90 * 1024", len: 90 * 1024 + 1, f: exp_neg, reference: ref_exp },
    Section { name: "tanh_grid", note: "tanh(k/1024), k in -12..=12", len: 24 * 1024 + 1, f: tanh_grid, reference: ref_tanh },
    Section { name: "ln_prob", note: "ln(k/100000)", len: 100_000, f: ln_prob, reference: ref_ln },
    Section { name: "powi_b1", note: "0.9^t, t to 100000", len: 100_000, f: powi_b1, reference: ref_b1 },
    Section { name: "powi_b2", note: "0.999^t, t to 100000", len: 100_000, f: powi_b2, reference: ref_b2 },
];

fn fnv(h: u64, bits: u32) -> u64 {
    let mut h = h;
    for b in bits.to_le_bytes() {
        h = (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

const BASIS: u64 = 0xcbf2_9ce4_8422_2325;

fn fingerprint() {
    println!("# os {} arch {} family {}", std::env::consts::OS, std::env::consts::ARCH, std::env::consts::FAMILY);
    println!(
        "# musl {} 64-bit {} big-endian {} fma-enabled {}",
        cfg!(target_env = "musl"),
        cfg!(target_pointer_width = "64"),
        cfg!(target_endian = "big"),
        cfg!(target_feature = "fma")
    );
    for s in SECTIONS {
        let (mut all, mut block, mut blocks) = (BASIS, BASIS, Vec::new());
        for i in 0..s.len {
            let bits = (s.f)(i).1.to_bits();
            all = fnv(all, bits);
            block = fnv(block, bits);
            if (i + 1) % BLOCK == 0 || i + 1 == s.len {
                blocks.push(block);
                block = BASIS;
            }
        }
        println!("# {}: {}", s.name, s.note);
        println!("fp {} n={} all={:016x}", s.name, s.len, all);
        for (k, b) in blocks.iter().enumerate() {
            println!("blk {} {} {:016x}", s.name, k, b);
        }
    }
    constants();
}

/// Calls whose argument is a literal may be folded by the compiler using the machine that BUILT the program, not the one that runs
/// it (`school.rs` has one: `3.0f32.ln()`). Print each both ways: folded where the compiler chooses to, and forced at run time.
fn constants() {
    macro_rules! both {
        ($name:expr, $lit:expr, $call:ident) => {{
            let folded = $lit.$call();
            let runtime = black_box($lit).$call();
            println!("const {} folded={:08x} runtime={:08x} same={}", $name, folded.to_bits(), runtime.to_bits(), folded.to_bits() == runtime.to_bits());
        }};
    }
    both!("ln(3)", 3.0f32, ln);
    both!("ln(2)", 2.0f32, ln);
    both!("ln(10)", 10.0f32, ln);
    both!("log2(3)", 3.0f32, log2);
    both!("log2(10)", 10.0f32, log2);
    both!("exp(1)", 1.0f32, exp);
    both!("exp(-1)", -1.0f32, exp);
    both!("tanh(0.5)", 0.5f32, tanh);
}

/// Position of a float on the number line, so that neighbors differ by one: the distance between two floats in steps of the last bit.
fn ordered(x: f32) -> i64 {
    let b = x.to_bits();
    if b >> 31 == 1 { -((b & 0x7fff_ffff) as i64) } else { b as i64 }
}

fn steps_apart(a: f32, b: f32) -> u64 {
    (ordered(a) - ordered(b)).unsigned_abs()
}

fn audit() {
    println!("# audit: each result against the f64 result rounded to f32 (see the file comment for what that means)");
    for s in SECTIONS {
        let (mut exact, mut off1, mut more, mut max, mut worst) = (0u64, 0u64, 0u64, 0u64, 0u32);
        for i in 0..s.len {
            let (x, y) = (s.f)(i);
            let right = ((s.reference)(x)) as f32;
            if y.is_nan() || right.is_nan() {
                continue;
            }
            match steps_apart(y, right) {
                0 => exact += 1,
                1 => off1 += 1,
                _ => more += 1,
            }
            let d = steps_apart(y, right);
            if d > max {
                max = d;
                worst = x.to_bits();
            }
        }
        println!(
            "audit {:<12} n={:<9} correctly-rounded={:<9} one-step-off={:<8} more-than-one={:<8} worst={} steps (input {:08x})",
            s.name, s.len, exact, off1, more, max, worst
        );
    }
    exact_cases();
}

/// Results that are exactly representable, where a comparison can come out as an exact tie: log2 of a power of two is a whole
/// number, ln(1) is 0, exp(0) is 1, tanh(0) is 0. A library that is off by a step on any of these would break a tie differently.
fn exact_cases() {
    let log2_ok = (0..=24).filter(|&k| black_box((1u32 << k) as f32).log2() == k as f32).count();
    let ln_ok = black_box(1.0f32).ln() == 0.0;
    let exp_ok = black_box(0.0f32).exp() == 1.0;
    let tanh_ok = black_box(0.0f32).tanh() == 0.0;
    println!("audit exact-cases log2(2^k)=k: {log2_ok}/25 ok; ln(1)=0: {ln_ok}; exp(0)=1: {exp_ok}; tanh(0)=0: {tanh_ok}");
}

fn dump(name: &str, from: u64, count: u64) {
    let Some(s) = SECTIONS.iter().find(|s| s.name == name) else {
        eprintln!("no section {name}; sections: {}", SECTIONS.iter().map(|s| s.name).collect::<Vec<_>>().join(" "));
        std::process::exit(2);
    };
    for i in from..(from + count).min(s.len) {
        let (x, y) = (s.f)(i);
        println!("{} {} input={:08x} ({}) result={:08x} ({})", s.name, i, x.to_bits(), x, y.to_bits(), y);
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        None => fingerprint(),
        Some("audit") if args.len() == 1 => audit(),
        Some("dump") if args.len() == 4 => match (args[2].parse(), args[3].parse()) {
            (Ok(from), Ok(count)) => dump(&args[1], from, count),
            _ => eprintln!("usage: mathprint dump NAME FROM COUNT"),
        },
        _ => eprintln!("usage: mathprint | mathprint audit | mathprint dump NAME FROM COUNT"),
    }
}
