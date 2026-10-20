// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DawsMacs
//! Thinking in joules, not just parameters.
//!
//! Two ways to know what a computation costs:
//!
//! 1. **Counted** (portable): every multiply-add and every 4-byte memory access is tallied in a `Ledger`, then priced by an
//!    `EnergyModel`. The prices are the widely cited 45 nm figures from Horowitz (ISSCC 2014): an fp32 multiply 3.7 pJ, an add
//!    0.9 pJ, a 32-bit DRAM read 640 pJ. Cache prices (L1 ~5 pJ, L2 ~50 pJ per 32-bit word) are my recollection of the same
//!    talk's SRAM numbers, not re-checked, and a modern chip is several times cheaper in absolute terms. The *ratios* (memory
//!    costs far more than arithmetic, and farther memory costs far more than near memory) are what to trust. It is a model, not
//!    a measurement, and is labelled as an estimate wherever it is printed.
//! 2. **Measured** (this machine only): the CPU's own energy counter (Intel RAPL), when the OS lets us read it. It includes
//!    everything the package does, so we subtract idle power. Unavailable unless the user grants read access.

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Ledger {
    /// fp32 multiply-accumulates.
    pub macs: u64,
    /// Transcendentals, divisions, square roots.
    pub other_ops: u64,
    /// Integer comparisons and adds (the fact store).
    pub int_ops: u64,
    /// 4-byte words loaded / stored.
    pub words_read: u64,
    pub words_written: u64,
    /// Largest working set touched, in bytes: decides which memory level the traffic is priced at.
    pub ws_bytes: u64,
}

impl Ledger {
    pub fn touch(&mut self, ws_bytes: usize) {
        self.ws_bytes = self.ws_bytes.max(ws_bytes as u64);
    }

    pub fn since(&self, earlier: &Ledger) -> Ledger {
        Ledger {
            macs: self.macs - earlier.macs,
            other_ops: self.other_ops - earlier.other_ops,
            int_ops: self.int_ops - earlier.int_ops,
            words_read: self.words_read - earlier.words_read,
            words_written: self.words_written - earlier.words_written,
            ws_bytes: self.ws_bytes,
        }
    }

    pub fn add(&mut self, o: &Ledger) {
        self.macs += o.macs;
        self.other_ops += o.other_ops;
        self.int_ops += o.int_ops;
        self.words_read += o.words_read;
        self.words_written += o.words_written;
        self.ws_bytes = self.ws_bytes.max(o.ws_bytes);
    }
}

#[derive(Clone, Copy, Debug)]
pub struct EnergyModel {
    pub pj_mac: f64,
    pub pj_other: f64,
    pub pj_int: f64,
    pub pj_word_l1: f64,
    pub pj_word_l2: f64,
    pub pj_word_dram: f64,
    pub l1_bytes: u64,
    pub l2_bytes: u64,
}

impl Default for EnergyModel {
    fn default() -> Self {
        EnergyModel {
            pj_mac: 3.7 + 0.9, // Horowitz 2014, 45 nm, fp32 multiply + add
            pj_other: 20.0,    // a few multiplies' worth for tanh/exp/div/sqrt (rough)
            pj_int: 0.2,       // 32-bit integer add (Horowitz: 0.1 pJ), rounded up for compares
            pj_word_l1: 5.0,   // small SRAM, per 32-bit word (recollection)
            pj_word_l2: 50.0,  // ~1 MB SRAM, per 32-bit word (recollection)
            pj_word_dram: 640.0, // Horowitz 2014, 32-bit DRAM read
            l1_bytes: 32 * 1024,
            l2_bytes: 1024 * 1024,
        }
    }
}

impl EnergyModel {
    pub fn pj_per_word(&self, ws_bytes: u64) -> f64 {
        if ws_bytes <= self.l1_bytes {
            self.pj_word_l1
        } else if ws_bytes <= self.l2_bytes {
            self.pj_word_l2
        } else {
            self.pj_word_dram
        }
    }

    /// Estimated picojoules, split as (arithmetic, memory).
    pub fn split_pj(&self, l: &Ledger) -> (f64, f64) {
        let arithmetic = l.macs as f64 * self.pj_mac + l.other_ops as f64 * self.pj_other + l.int_ops as f64 * self.pj_int;
        let memory = (l.words_read + l.words_written) as f64 * self.pj_per_word(l.ws_bytes);
        (arithmetic, memory)
    }

    pub fn joules(&self, l: &Ledger) -> f64 {
        let (a, m) = self.split_pj(l);
        (a + m) * 1e-12
    }
}

/// "12.3 uJ", "4.5 mJ"...
pub fn fmt_joules(j: f64) -> String {
    let a = j.abs();
    if a >= 1.0 {
        format!("{j:.2} J")
    } else if a >= 1e-3 {
        format!("{:.2} mJ", j * 1e3)
    } else if a >= 1e-6 {
        format!("{:.2} uJ", j * 1e6)
    } else if a >= 1e-9 {
        format!("{:.2} nJ", j * 1e9)
    } else {
        format!("{:.2} pJ", j * 1e12)
    }
}

/// Intel RAPL package energy, if the kernel lets this user read it.
pub struct Rapl {
    path: String,
    max_uj: u64,
}

pub const RAPL_PATH: &str = "/sys/class/powercap/intel-rapl:0";

impl Rapl {
    pub fn open() -> Option<Rapl> {
        let path = RAPL_PATH.to_string();
        std::fs::read_to_string(format!("{path}/energy_uj")).ok()?;
        let max_uj = std::fs::read_to_string(format!("{path}/max_energy_range_uj")).ok()?.trim().parse().ok()?;
        Some(Rapl { path, max_uj })
    }

    pub fn read_uj(&self) -> Option<u64> {
        std::fs::read_to_string(format!("{}/energy_uj", self.path)).ok()?.trim().parse().ok()
    }

    /// Joules used between two readings (handles the counter wrapping).
    pub fn delta_j(&self, before: u64, after: u64) -> f64 {
        let d = if after >= before { after - before } else { self.max_uj - before + after };
        d as f64 * 1e-6
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_far_away_costs_more() {
        let m = EnergyModel::default();
        let mut near = Ledger { words_read: 1000, ..Default::default() };
        near.ws_bytes = 8 * 1024;
        let mut far = near;
        far.ws_bytes = 64 * 1024 * 1024;
        assert!(m.joules(&far) > 100.0 * m.joules(&near));
    }

    #[test]
    fn one_mac_is_the_published_figure() {
        let m = EnergyModel::default();
        let l = Ledger { macs: 1, ..Default::default() };
        assert!((m.joules(&l) - 4.6e-12).abs() < 1e-15);
    }

    #[test]
    fn since_subtracts() {
        let a = Ledger { macs: 10, words_read: 4, ..Default::default() };
        let b = Ledger { macs: 25, words_read: 9, ..Default::default() };
        let d = b.since(&a);
        assert_eq!((d.macs, d.words_read), (15, 5));
    }

    #[test]
    fn formatting_picks_a_sensible_unit() {
        assert_eq!(fmt_joules(2.5e-6), "2.50 uJ");
        assert_eq!(fmt_joules(3.0e-3), "3.00 mJ");
        assert_eq!(fmt_joules(1.5), "1.50 J");
    }
}
