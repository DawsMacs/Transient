// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DawsMacs
//! The baby network, written by hand: no framework, no autodiff.
//!
//! A query is a fixed number of word slots. A *reader* turns the slots into one feature vector, one tanh hidden layer follows, and
//! a softmax picks one answer word:
//!
//!   ids[len] -> reader -> x -> a1 = tanh(W1 x + b1) -> p = softmax(W2 a1 + b2)
//!
//! Three readers (the part that decides what the baby can notice):
//!   concat   each slot's embedding laid side by side. Order matters, and so does *position*: a word is tied to its slot.
//!   bag      the average of the words' embeddings. Position-free, order-blind.
//!   bigram   the bag, plus the average of hashed *pairs* of neighboring words (the fastText recipe). Position-free but it
//!            keeps local order ("three minus two" differs from "two minus three").
//!   letters  no word list at all. The query is turned into hashed pieces of its own spelling (whole words, 3- and 4-letter
//!            groups, word pairs) and their embeddings are averaged. Any word, even one never seen, has a reading, and a typo
//!            shares most of its pieces with the right spelling. The "ids" this reader takes are those hashed piece numbers.
//!
//! All parameters live in one flat `theta` vector (embeddings, pair embeddings, W1, b1, W2, b2), which keeps the optimizer, the
//! file format and the gradient check trivial.

use crate::ledger::Ledger;
use crate::rng::Rng;
use std::ops::Range;

const PAD: usize = 0;

/// Dot product with eight independent running sums. A plain `for` loop that adds into one `f32` cannot be vectorized, because
/// floating-point addition is not associative and the compiler must keep the written order; splitting the work into lanes
/// lets it use SIMD without any unsafe code or special flags.
#[inline]
fn dot(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len());
    let mut acc = [0.0f32; 8];
    let (ca, cb) = (a.chunks_exact(8), b.chunks_exact(8));
    let (ra, rb) = (ca.remainder(), cb.remainder());
    for (x, y) in ca.zip(cb) {
        for l in 0..8 {
            acc[l] += x[l] * y[l];
        }
    }
    let mut sum = acc.iter().sum::<f32>();
    for (x, y) in ra.iter().zip(rb) {
        sum += x * y;
    }
    sum
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reader {
    Concat,
    Bag,
    Bigram,
    Letters,
}

impl Reader {
    pub fn name(self) -> &'static str {
        match self {
            Reader::Concat => "concat",
            Reader::Bag => "bag",
            Reader::Bigram => "bigram",
            Reader::Letters => "letters",
        }
    }

    pub fn parse(s: &str) -> Option<Reader> {
        [Reader::Concat, Reader::Bag, Reader::Bigram, Reader::Letters].into_iter().find(|r| r.name() == s)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Config {
    /// Words the network can read, including `<pad>` and `<unk>`.
    pub vocab: usize,
    /// Numbers per word embedding.
    pub dim: usize,
    /// Word slots per query.
    pub len: usize,
    pub hidden: usize,
    /// Answer words it can say.
    pub classes: usize,
    pub reader: Reader,
    /// Hash buckets for neighboring word pairs (bigram reader only).
    pub buckets: usize,
}

/// Which pair-embedding row a pair of neighboring words lands in.
fn bucket(a: usize, b: usize, buckets: usize) -> usize {
    let h = (a as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (b as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    ((h >> 17) % buckets as u64) as usize
}

impl Config {
    pub fn input(&self) -> usize {
        match self.reader {
            Reader::Concat => self.len * self.dim,
            Reader::Bag | Reader::Letters => self.dim,
            Reader::Bigram => 2 * self.dim,
        }
    }

    fn pair_rows(&self) -> usize {
        if matches!(self.reader, Reader::Bigram | Reader::Letters) { self.buckets } else { 0 }
    }

    /// Rows in the word-embedding table (the letters reader has none: it reads spelling, not a word list).
    fn word_rows(&self) -> usize {
        if self.reader == Reader::Letters { 0 } else { self.vocab }
    }

    /// (word embeddings, pair embeddings, W1, b1, W2, b2) as ranges into `theta`.
    fn ranges(&self) -> [Range<usize>; 6] {
        let sizes = [
            self.word_rows() * self.dim,
            self.pair_rows() * self.dim,
            self.hidden * self.input(),
            self.hidden,
            self.classes * self.hidden,
            self.classes,
        ];
        let mut out: [Range<usize>; 6] = [0..0, 0..0, 0..0, 0..0, 0..0, 0..0];
        let mut at = 0;
        for (i, s) in sizes.iter().enumerate() {
            out[i] = at..at + s;
            at += s;
        }
        out
    }

    pub fn n_params(&self) -> usize {
        self.ranges()[5].end
    }
}

pub struct Net {
    pub cfg: Config,
    pub theta: Vec<f32>,
}

/// Reusable work buffers, so a forward/backward pass allocates nothing big.
pub struct Scratch {
    x: Vec<f32>,
    a1: Vec<f32>,
    p: Vec<f32>,
    dz1: Vec<f32>,
    dx: Vec<f32>,
    /// Embedding numbers read by the last forward pass (what the backward pass will touch).
    emb_words: usize,
    /// Everything computed through this scratch is tallied here (see `ledger.rs`).
    pub ledger: Ledger,
}

impl Scratch {
    pub fn new(cfg: &Config) -> Scratch {
        Scratch {
            x: vec![0.0; cfg.input()],
            a1: vec![0.0; cfg.hidden],
            p: vec![0.0; cfg.classes],
            dz1: vec![0.0; cfg.hidden],
            dx: vec![0.0; cfg.input()],
            emb_words: 0,
            ledger: Ledger::default(),
        }
    }
}

impl Net {
    pub fn new(cfg: Config, rng: &mut Rng) -> Net {
        let [e, pr, w1, _b1, w2, _b2] = cfg.ranges();
        let mut theta = vec![0.0; cfg.n_params()];
        for t in &mut theta[e] {
            *t = rng.range(-0.3, 0.3);
        }
        for t in &mut theta[pr] {
            *t = rng.range(-0.3, 0.3);
        }
        let lim1 = (6.0 / (cfg.input() + cfg.hidden) as f32).sqrt();
        for t in &mut theta[w1] {
            *t = rng.range(-lim1, lim1);
        }
        let lim2 = (6.0 / (cfg.hidden + cfg.classes) as f32).sqrt();
        for t in &mut theta[w2] {
            *t = rng.range(-lim2, lim2);
        }
        // Biases start at zero.
        Net { cfg, theta }
    }

    pub fn from_theta(cfg: Config, theta: Vec<f32>) -> Net {
        assert_eq!(theta.len(), cfg.n_params());
        Net { cfg, theta }
    }

    /// Forward pass. Leaves the answer probabilities in `s.p`.
    fn forward(&self, ids: &[usize], s: &mut Scratch) {
        let c = &self.cfg;
        if c.reader != Reader::Letters {
            assert_eq!(ids.len(), c.len, "a query must be exactly `len` word slots");
        }
        let [e, pr, w1, b1, w2, b2] = c.ranges();
        let (d, inp, h) = (c.dim, c.input(), c.hidden);
        let th = &self.theta;

        // ---- reader: ids -> x ----
        let words: Vec<usize> = if c.reader == Reader::Letters { Vec::new() } else { ids.iter().copied().filter(|&i| i != PAD).collect() };
        let embed_reads;
        match c.reader {
            Reader::Letters => {
                s.x.iter_mut().for_each(|v| *v = 0.0);
                let n = ids.len().max(1) as f32;
                for &b in ids {
                    let src = pr.start + b * d;
                    for k in 0..d {
                        s.x[k] += th[src + k] / n;
                    }
                }
                embed_reads = ids.len() * d;
            }
            Reader::Concat => {
                for (pos, &id) in ids.iter().enumerate() {
                    let src = e.start + id * d;
                    s.x[pos * d..(pos + 1) * d].copy_from_slice(&th[src..src + d]);
                }
                embed_reads = c.len * d;
            }
            Reader::Bag | Reader::Bigram => {
                s.x.iter_mut().for_each(|v| *v = 0.0);
                let n = words.len().max(1) as f32;
                for &id in &words {
                    let src = e.start + id * d;
                    for k in 0..d {
                        s.x[k] += th[src + k] / n;
                    }
                }
                let mut reads = words.len() * d;
                if c.reader == Reader::Bigram && words.len() >= 2 {
                    let nb = (words.len() - 1) as f32;
                    for pair in words.windows(2) {
                        let src = pr.start + bucket(pair[0], pair[1], c.buckets) * d;
                        for k in 0..d {
                            s.x[d + k] += th[src + k] / nb;
                        }
                    }
                    reads += (words.len() - 1) * d;
                }
                embed_reads = reads;
            }
        }

        s.emb_words = embed_reads;
        s.ledger.touch(4 * c.n_params());
        s.ledger.macs += (h * inp + c.classes * h) as u64;
        s.ledger.other_ops += (h + 2 * c.classes) as u64;
        s.ledger.words_read += (h * inp + h + c.classes * h + c.classes + embed_reads) as u64;

        for j in 0..h {
            let row = &th[w1.start + j * inp..w1.start + (j + 1) * inp];
            s.a1[j] = (th[b1.start + j] + dot(row, &s.x)).tanh();
        }
        let mut max = f32::NEG_INFINITY;
        for k in 0..c.classes {
            let row = &th[w2.start + k * h..w2.start + (k + 1) * h];
            let z = th[b2.start + k] + dot(row, &s.a1);
            s.p[k] = z;
            max = max.max(z);
        }
        let mut sum = 0.0;
        for p in s.p.iter_mut() {
            *p = (*p - max).exp();
            sum += *p;
        }
        for p in s.p.iter_mut() {
            *p /= sum;
        }
    }

    /// Probabilities over the answer words for one query.
    pub fn probs(&self, ids: &[usize], s: &mut Scratch) -> Vec<f32> {
        self.forward(ids, s);
        s.p.clone()
    }

    /// Cross-entropy loss for one example, and its gradient *added into* `grad` (same layout as `theta`).
    pub fn loss_and_grad(&self, ids: &[usize], target: usize, grad: &mut [f32], s: &mut Scratch) -> f32 {
        self.forward(ids, s);
        let c = &self.cfg;
        let [e, pr, w1, b1, w2, b2] = c.ranges();
        let (d, inp, h) = (c.dim, c.input(), c.hidden);
        let th = &self.theta;

        // Backward costs about twice the forward arithmetic, and reads then writes every gradient slot it touches.
        let grad_words = (c.classes * h + h * inp + h + c.classes + s.emb_words) as u64;
        s.ledger.macs += (2 * c.classes * h + 2 * h * inp) as u64;
        s.ledger.other_ops += 1;
        s.ledger.words_read += (c.classes * h + h * inp) as u64 + grad_words;
        s.ledger.words_written += grad_words;

        let loss = -(s.p[target].max(1e-12)).ln();
        // Softmax + cross-entropy: d(loss)/d(logit) = p - onehot.
        s.p[target] -= 1.0;

        s.dz1.iter_mut().for_each(|v| *v = 0.0);
        for k in 0..c.classes {
            let g = s.p[k];
            grad[b2.start + k] += g;
            let off = w2.start + k * h;
            for j in 0..h {
                grad[off + j] += g * s.a1[j];
                s.dz1[j] += th[off + j] * g;
            }
        }
        for j in 0..h {
            s.dz1[j] *= 1.0 - s.a1[j] * s.a1[j]; // tanh'
        }

        s.dx.iter_mut().for_each(|v| *v = 0.0);
        for j in 0..h {
            let g = s.dz1[j];
            grad[b1.start + j] += g;
            let off = w1.start + j * inp;
            for i in 0..inp {
                grad[off + i] += g * s.x[i];
                s.dx[i] += th[off + i] * g;
            }
        }

        // ---- reader backward: dx -> embeddings ----
        match c.reader {
            Reader::Letters => {
                let n = ids.len().max(1) as f32;
                for &b in ids {
                    let dst = pr.start + b * d;
                    for k in 0..d {
                        grad[dst + k] += s.dx[k] / n;
                    }
                }
            }
            Reader::Concat => {
                for (pos, &id) in ids.iter().enumerate() {
                    let dst = e.start + id * d;
                    for k in 0..d {
                        grad[dst + k] += s.dx[pos * d + k];
                    }
                }
            }
            Reader::Bag | Reader::Bigram => {
                let words: Vec<usize> = ids.iter().copied().filter(|&i| i != PAD).collect();
                let n = words.len().max(1) as f32;
                for &id in &words {
                    let dst = e.start + id * d;
                    for k in 0..d {
                        grad[dst + k] += s.dx[k] / n;
                    }
                }
                if c.reader == Reader::Bigram && words.len() >= 2 {
                    let nb = (words.len() - 1) as f32;
                    for pair in words.windows(2) {
                        let dst = pr.start + bucket(pair[0], pair[1], c.buckets) * d;
                        for k in 0..d {
                            grad[dst + k] += s.dx[d + k] / nb;
                        }
                    }
                }
            }
        }
        loss
    }
}

/// Adam with decoupled weight decay (AdamW), or plain gradient descent.
pub struct Adam {
    m: Vec<f32>,
    v: Vec<f32>,
    t: i32,
    pub lr: f32,
    pub weight_decay: f32,
    /// Plain gradient descent instead of Adam: no moment vectors to keep or move.
    pub plain: bool,
}

impl Adam {
    pub fn new(n: usize, lr: f32, weight_decay: f32) -> Adam {
        Adam { m: vec![0.0; n], v: vec![0.0; n], t: 0, lr, weight_decay, plain: false }
    }

    pub fn plain_sgd(_n: usize, lr: f32) -> Adam {
        Adam { m: Vec::new(), v: Vec::new(), t: 0, lr, weight_decay: 0.0, plain: true }
    }

    pub fn step(&mut self, theta: &mut [f32], grad: &[f32], ledger: &mut Ledger) {
        let n = theta.len() as u64;
        if self.plain {
            ledger.touch(8 * theta.len()); // theta and grad
            ledger.macs += n;
            ledger.words_read += 2 * n;
            ledger.words_written += n;
            for i in 0..theta.len() {
                theta[i] -= self.lr * grad[i];
            }
            return;
        }
        ledger.touch(16 * theta.len()); // theta, grad and both moment vectors
        ledger.macs += 6 * n;
        ledger.other_ops += 2 * n; // sqrt and divide
        ledger.words_read += 4 * n;
        ledger.words_written += 3 * n;
        const B1: f32 = 0.9;
        const B2: f32 = 0.999;
        const EPS: f32 = 1e-8;
        self.t += 1;
        let c1 = 1.0 - B1.powi(self.t);
        let c2 = 1.0 - B2.powi(self.t);
        for i in 0..theta.len() {
            self.m[i] = B1 * self.m[i] + (1.0 - B1) * grad[i];
            self.v[i] = B2 * self.v[i] + (1.0 - B2) * grad[i] * grad[i];
            let update = (self.m[i] / c1) / ((self.v[i] / c2).sqrt() + EPS);
            theta[i] -= self.lr * (update + self.weight_decay * theta[i]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tiny(reader: Reader) -> (Net, Vec<usize>) {
        let cfg = Config { vocab: 8, dim: 3, len: 5, hidden: 5, classes: 4, reader, buckets: 7 };
        let mut rng = Rng::new(3);
        // For the letters reader the ids are hashed-piece numbers (any count, repeats allowed).
        let ids = if reader == Reader::Letters { vec![2, 6, 5, 3, 0, 5, 1] } else { vec![2, 6, 5, 3, 0] };
        (Net::new(cfg, &mut rng), ids)
    }

    #[test]
    fn probabilities_sum_to_one() {
        for r in [Reader::Concat, Reader::Bag, Reader::Bigram, Reader::Letters] {
            let (net, ids) = tiny(r);
            let mut s = Scratch::new(&net.cfg);
            let p = net.probs(&ids, &mut s);
            assert!((p.iter().sum::<f32>() - 1.0).abs() < 1e-5);
            assert!(p.iter().all(|&x| x > 0.0));
        }
    }

    /// The hand-written backward pass must match a numerical derivative, for every reader. This is the test that matters most in
    /// a from-scratch network: a wrong gradient still "trains", just badly.
    #[test]
    fn backward_matches_finite_differences() {
        for r in [Reader::Concat, Reader::Bag, Reader::Bigram, Reader::Letters] {
            let (mut net, ids) = tiny(r);
            let target = 2;
            let mut s = Scratch::new(&net.cfg);
            let mut grad = vec![0.0; net.cfg.n_params()];
            net.loss_and_grad(&ids, target, &mut grad, &mut s);

            let eps = 1e-3f32;
            let mut checked = 0;
            for i in (0..net.theta.len()).step_by(2) {
                let orig = net.theta[i];
                net.theta[i] = orig + eps;
                let up = net.loss_and_grad(&ids, target, &mut vec![0.0; grad.len()], &mut s);
                net.theta[i] = orig - eps;
                let down = net.loss_and_grad(&ids, target, &mut vec![0.0; grad.len()], &mut s);
                net.theta[i] = orig;
                let numeric = (up - down) / (2.0 * eps);
                let err = (numeric - grad[i]).abs();
                assert!(err < 2e-3 + 2e-2 * grad[i].abs(), "{r:?} param {i}: analytic {} vs numeric {numeric}", grad[i]);
                checked += 1;
            }
            assert!(checked > 20);
        }
    }

    #[test]
    fn bag_ignores_position_but_concat_does_not() {
        let mut rng = Rng::new(9);
        let cfg = |reader| Config { vocab: 8, dim: 4, len: 5, hidden: 6, classes: 3, reader, buckets: 7 };
        let (bag, concat) = (Net::new(cfg(Reader::Bag), &mut rng), Net::new(cfg(Reader::Concat), &mut rng));
        let (a, b) = (vec![2, 3, 4, 0, 0], vec![0, 4, 2, 3, 0]);
        let (mut s1, mut s2) = (Scratch::new(&bag.cfg), Scratch::new(&concat.cfg));
        assert_eq!(bag.probs(&a, &mut s1), bag.probs(&b, &mut s1));
        assert_ne!(concat.probs(&a, &mut s2), concat.probs(&b, &mut s2));
    }

    #[test]
    fn bigram_sees_local_order_but_not_position() {
        let mut rng = Rng::new(9);
        let cfg = Config { vocab: 8, dim: 4, len: 6, hidden: 6, classes: 3, reader: Reader::Bigram, buckets: 31 };
        let net = Net::new(cfg, &mut rng);
        let mut s = Scratch::new(&cfg);
        let shifted = (net.probs(&[2, 3, 4, 0, 0, 0], &mut s), net.probs(&[0, 2, 3, 4, 0, 0], &mut s));
        assert_eq!(shifted.0, shifted.1, "the same phrase in a different place reads the same");
        assert_ne!(net.probs(&[2, 3, 4, 0, 0, 0], &mut s), net.probs(&[4, 3, 2, 0, 0, 0], &mut s), "reversed order reads differently");
    }

    #[test]
    fn can_memorize_a_handful_of_examples() {
        let cfg = Config { vocab: 8, dim: 4, len: 3, hidden: 12, classes: 3, reader: Reader::Concat, buckets: 0 };
        let mut rng = Rng::new(1);
        let mut net = Net::new(cfg, &mut rng);
        let data: Vec<(Vec<usize>, usize)> = vec![
            (vec![2, 3, 0], 0),
            (vec![4, 3, 0], 1),
            (vec![5, 6, 0], 2),
            (vec![2, 7, 0], 1),
        ];
        let mut s = Scratch::new(&cfg);
        let mut adam = Adam::new(cfg.n_params(), 0.05, 0.0);
        let mut grad = vec![0.0; cfg.n_params()];
        for _ in 0..300 {
            grad.iter_mut().for_each(|g| *g = 0.0);
            for (ids, y) in &data {
                net.loss_and_grad(ids, *y, &mut grad, &mut s);
            }
            grad.iter_mut().for_each(|g| *g /= data.len() as f32);
            adam.step(&mut net.theta, &grad, &mut s.ledger);
        }
        for (ids, y) in &data {
            let p = net.probs(ids, &mut s);
            let best = p.iter().enumerate().max_by(|a, b| a.1.partial_cmp(b.1).unwrap()).unwrap().0;
            assert_eq!(best, *y);
        }
    }
}
