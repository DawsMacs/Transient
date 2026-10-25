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

/// `2^(-c / 256)` as a fixed-point number with 32 fractional bits (`1 << 32` is one), zero once it falls under one part in 2^32. The inverse of
/// [`log2`]: an explanation that costs `c` more bits than another is `exp2_neg(c)` times as believable. Table values are 2^(-f/256) * 2^32, rounded.
pub fn exp2_neg(c: Cost) -> u64 {
    const FRACTION: [u64; 256] = [
    4294967296, 4283353945, 4271771996, 4260221365, 4248701965, 4237213713, 4225756525, 4214330316,
    4202935003, 4191570503, 4180236731, 4168933606, 4157661043, 4146418961, 4135207276, 4124025908,
    4112874773, 4101753790, 4090662878, 4079601955, 4068570940, 4057569753, 4046598312, 4035656537,
    4024744348, 4013861665, 4003008408, 3992184498, 3981389855, 3970624400, 3959888054, 3949180739,
    3938502376, 3927852886, 3917232192, 3906640216, 3896076880, 3885542107, 3875035819, 3864557939,
    3854108391, 3843687099, 3833293984, 3822928972, 3812591987, 3802282952, 3792001792, 3781748432,
    3771522796, 3761324810, 3751154399, 3741011488, 3730896002, 3720807869, 3710747013, 3700713361,
    3690706840, 3680727376, 3670774895, 3660849326, 3650950594, 3641078629, 3631233356, 3621414705,
    3611622603, 3601856978, 3592117759, 3582404874, 3572718252, 3563057822, 3553423514, 3543815256,
    3534232978, 3524676610, 3515146082, 3505641324, 3496162267, 3486708840, 3477280975, 3467878602,
    3458501653, 3449150059, 3439823750, 3430522660, 3421246719, 3411995860, 3402770014, 3393569115,
    3384393094, 3375241885, 3366115421, 3357013633, 3347936457, 3338883824, 3329855670, 3320851927,
    3311872529, 3302917412, 3293986508, 3285079754, 3276197082, 3267338429, 3258503729, 3249692918,
    3240905930, 3232142703, 3223403170, 3214687269, 3205994934, 3197326104, 3188680714, 3180058700,
    3171459999, 3162884549, 3154332287, 3145803149, 3137297074, 3128813999, 3120353861, 3111916599,
    3103502151, 3095110456, 3086741451, 3078395075, 3070071267, 3061769967, 3053491112, 3045234644,
    3037000500, 3028788621, 3020598946, 3012431416, 3004285971, 2996162550, 2988061094, 2979981545,
    2971923842, 2963887926, 2955873740, 2947881223, 2939910317, 2931960965, 2924033107, 2916126685,
    2908241642, 2900377920, 2892535461, 2884714207, 2876914102, 2869135087, 2861377107, 2853640104,
    2845924021, 2838228802, 2830554390, 2822900730, 2815267765, 2807655439, 2800063696, 2792492481,
    2784941738, 2777411411, 2769901447, 2762411789, 2754942382, 2747493172, 2740064105, 2732655125,
    2725266179, 2717897212, 2710548171, 2703219000, 2695909648, 2688620059, 2681350181, 2674099961,
    2666869345, 2659658279, 2652466713, 2645294591, 2638141863, 2631008476, 2623894376, 2616799513,
    2609723834, 2602667287, 2595629821, 2588611383, 2581611923, 2574631389, 2567669730, 2560726895,
    2553802834, 2546897494, 2540010826, 2533142779, 2526293303, 2519462348, 2512649863, 2505855799,
    2499080105, 2492322733, 2485583632, 2478862753, 2472160047, 2465475465, 2458808958, 2452160476,
    2445529972, 2438917396, 2432322700, 2425745836, 2419186755, 2412645410, 2406121752, 2399615734,
    2393127307, 2386656425, 2380203040, 2373767105, 2367348571, 2360947394, 2354563524, 2348196917,
    2341847524, 2335515300, 2329200197, 2322902170, 2316621173, 2310357160, 2304110083, 2297879899,
    2291666561, 2285470023, 2279290240, 2273127167, 2266980759, 2260850970, 2254737756, 2248641071,
    2242560872, 2236497113, 2230449750, 2224418739, 2218404036, 2212405596, 2206423375, 2200457330,
    2194507417, 2188573592, 2182655811, 2176754033, 2170868212, 2164998306, 2159144272, 2153306067,
    ];
    let whole = c / BIT;
    if whole >= 32 {
        return 0;
    }
    FRACTION[(c % BIT) as usize] >> whole
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
    fn belief_falls_by_half_for_every_bit() {
        assert_eq!(exp2_neg(0), 1 << 32);
        assert_eq!(exp2_neg(BIT), 1 << 31);
        assert_eq!(exp2_neg(10 * BIT), 1 << 22);
        assert_eq!(exp2_neg(32 * BIT), 0);
        for c in (0..20 * BIT).step_by(7) {
            let real = (-(c as f64) / 256.0).exp2() * 4294967296.0;
            assert!((exp2_neg(c) as f64 - real).abs() <= 1.0 + real / 1e8, "c={c}");
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
