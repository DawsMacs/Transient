// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DawsMacs
//! The questions Transient asks the operating system, and what it says when the system cannot answer.
//!
//! Linux answers memory questions from `/proc/self/status`. macOS, Windows, most BSDs and sandboxes without `/proc` cannot, and for
//! them the answer is "unavailable": reports say `n/a` instead of printing `0 MB`, which would read as a measurement. (Energy and
//! CPU time have the same Linux-only sources, `ledger::Rapl` and `compare::cpu_seconds`; they already return nothing when their
//! files are missing, so they need no change.) Nothing here uses `unsafe` or a dependency.

/// One field of `/proc/self/status`, in megabytes: `VmRSS:` is the memory resident now, `VmHWM:` the peak. `NaN` when the system
/// does not say.
fn status_mb(key: &str) -> f64 {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|t| t.lines().find(|l| l.starts_with(key)).and_then(|l| l.split_whitespace().nth(1).and_then(|n| n.parse::<f64>().ok())))
        .map_or(f64::NAN, |kb| kb / 1024.0)
}

/// Memory resident in this process now, in MB (`NaN` if unavailable).
pub fn resident_mb() -> f64 {
    status_mb("VmRSS:")
}

/// Peak memory of this process so far, in MB (`NaN` if unavailable).
pub fn peak_mb() -> f64 {
    status_mb("VmHWM:")
}

/// A memory figure for a report: `12.3 MB`, or `n/a` when the system could not say.
pub fn fmt_mb(mb: f64, decimals: usize) -> String {
    if mb.is_nan() {
        "n/a".to_string()
    } else {
        format!("{mb:.decimals$} MB")
    }
}

/// What to tell someone whose machine gave no energy reading. If the RAPL counter exists but cannot be read, that is a permission
/// the owner can grant; if it does not exist at all (macOS, Windows, a VM, a container, hardware without RAPL), there is nothing to
/// grant and the advice would be wrong.
pub fn energy_unavailable(rapl_path: &str) -> String {
    if std::path::Path::new(rapl_path).exists() {
        format!("Real energy not measured: this user cannot read {rapl_path}/energy_uj. To allow it until reboot:\n  sudo chmod a+r {rapl_path}/energy_uj")
    } else {
        format!("Real energy not measured: this system has no RAPL energy counter ({rapl_path} does not exist). Real joules need Linux on hardware that has one, not a VM or container.")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_unavailable_figure_says_so_and_a_real_one_keeps_its_format() {
        assert_eq!(fmt_mb(f64::NAN, 1), "n/a");
        assert_eq!(fmt_mb(f64::NAN, 0), "n/a");
        assert_eq!(fmt_mb(12.34, 1), "12.3 MB");
        assert_eq!(fmt_mb(12.5, 0), "12 MB");
    }

    #[test]
    fn memory_is_either_a_real_figure_or_unavailable_never_zero() {
        // Linux has /proc/self/status; elsewhere the answer must be NaN, not 0.
        let (now, peak) = (resident_mb(), peak_mb());
        if std::path::Path::new("/proc/self/status").exists() {
            assert!(now > 0.0 && peak >= now, "resident {now} MB, peak {peak} MB");
        } else {
            assert!(now.is_nan() && peak.is_nan());
        }
    }

    #[test]
    fn the_energy_message_does_not_tell_anyone_to_change_permissions_on_a_file_that_is_not_there() {
        let m = energy_unavailable("/nonexistent/path/for/this/test");
        assert!(m.contains("no RAPL energy counter") && !m.contains("chmod"), "{m}");
        let here = std::env::temp_dir();
        let m = energy_unavailable(&here.to_string_lossy());
        assert!(m.contains("chmod"), "{m}");
    }
}
