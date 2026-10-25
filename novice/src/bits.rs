// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DawsMacs
//! Costs in bits, with integers only.
//!
//! Novice compares explanations by one currency: how many bits it takes to write the explanation down plus how many bits it takes to write down
//! what the explanation gets wrong. Every comparison that decides an answer goes through this module, so it must give the same number on every
//! platform. That rules out the math library (`f64::log2` differs in the last bit between systems); the logarithm below uses integer
//! multiplication and shifts only. Floating point is allowed in reports, never in a decision.
//!
//! A [`Cost`] counts 1/256 of a bit, so sums are exact and an error of less than one unit per logarithm is the only rounding there is.

/// A number of bits, in units of 1/256 bit.
pub type Cost = u64;

/// One bit.
pub const BIT: Cost = 256;

/// `log2(n)` in 1/256 bit, rounded down, for `n >= 1` (`0` for `n == 0`, which has no logarithm; callers pass counts that start at one).
pub fn log2(n: u64) -> Cost {
    if n == 0 {
        return 0;
    }
    let whole = 63 - u64::from(n.leading_zeros());
    // The mantissa in [1, 2) as a fixed-point number with 32 fractional bits: 2^32 <= m < 2^33.
    let mut m: u128 = if whole >= 32 { u128::from(n >> (whole - 32)) } else { u128::from(n) << (32 - whole) };
    // Each squaring doubles the logarithm; the bit that falls out past 2 is the next fractional digit.
    let mut frac: u64 = 0;
    for _ in 0..8 {
        m = (m * m) >> 32;
        frac <<= 1;
        if m >= 1 << 33 {
            m >>= 1;
            frac |= 1;
        }
    }
    whole * BIT + frac
}

/// Bits to write the non-negative integer `n` in a prefix-free code (Elias gamma of `n + 1`: `2 * floor(log2(n + 1)) + 1` bits).
/// A small number is cheap and a large one is not, which is the preference for short constants that an explanation should have.
pub fn uint(n: u64) -> Cost {
    let n1 = n.saturating_add(1);
    (2 * (63 - u64::from(n1.leading_zeros())) + 1) * BIT
}

/// Bits to point at one of `k` equally likely things.
pub fn pick(k: u64) -> Cost {
    log2(k)
}

/// Bits to say that an event of probability `num / den` happened: `-log2(num / den)`. Zero when `num >= den`.
pub fn surprise(num: u64, den: u64) -> Cost {
    log2(den.max(1)).saturating_sub(log2(num.max(1)))
}

/// Whole bits, for reports.
pub fn as_bits(c: Cost) -> f64 {
    c as f64 / BIT as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_on_powers_of_two() {
        for k in 0..63u32 {
            assert_eq!(log2(1u64 << k), u64::from(k) * BIT, "2^{k}");
        }
    }

    #[test]
    fn never_decreases_and_stays_close_to_the_real_logarithm() {
        let mut last = 0;
        let mut n = 1u64;
        while n < 5_000_000 {
            let c = log2(n);
            assert!(c >= last, "log2 went down at {n}");
            last = c;
            // Tests may use floating point; the code under test does not. Rounding down by at most 2/256 bit.
            let real = (n as f64).log2() * 256.0;
            assert!(real - c as f64 <= 2.0 && c as f64 <= real + 0.5, "n={n}: got {c}, real {real}");
            n += 1 + n / 97;
        }
    }

    #[test]
    fn large_numbers_work_too() {
        assert_eq!(log2(u64::MAX) / BIT, 63);
        assert!(log2(u64::MAX) > 63 * BIT + 250);
    }

    #[test]
    fn integer_code_lengths_are_the_elias_gamma_lengths() {
        let want = [(0u64, 1u64), (1, 3), (2, 3), (3, 5), (6, 5), (7, 7), (14, 7), (15, 9), (254, 15), (255, 17)];
        for (n, bits) in want {
            assert_eq!(uint(n), bits * BIT, "uint({n})");
        }
    }

    #[test]
    fn surprise_is_the_negative_log_of_the_probability() {
        assert_eq!(surprise(1, 2), BIT);
        assert_eq!(surprise(1, 8), 3 * BIT);
        assert_eq!(surprise(5, 5), 0);
        assert_eq!(surprise(9, 5), 0);
    }
}
