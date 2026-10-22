// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DawsMacs
//! A small attention transformer, written from scratch (no dependencies, hand-written backward pass), as the *matched* counterpart for the
//! controlled comparison of Vision Entry 2: the thing the industry actually builds, at a size that fits the same lessons.
//!
//! Pre-LayerNorm encoder: token + position embeddings, `layers` blocks of (multi-head self-attention, then a ReLU feed-forward layer, each with a
//! residual), a final LayerNorm, mean-pool over the words, and a linear layer over the answer words. It reads the same word lists as the baby
//! and says one answer word, so the two are graded on the same questions. Every multiply-add and every word moved is tallied in a `Ledger`,
//! so training and answering can be priced in estimated joules and (where the kernel allows) measured.
//!
//! It is deliberately plain: no dropout, no learning-rate schedule, no tokenizer tricks. The comparison is only meaningful if nothing here
//! is tuned to lose; `check_gradients` (a test) pins the backward pass against finite differences.

use crate::ledger::Ledger;
use crate::net::Adam;
use crate::rng::Rng;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TConfig {
    /// Words it can read, including `<pad>`-free: id 0 is `<unk>`.
    pub vocab: usize,
    /// Answer words it can say.
    pub classes: usize,
    /// Model width.
    pub d: usize,
    pub heads: usize,
    pub layers: usize,
    /// Feed-forward width.
    pub ff: usize,
    /// Longest question (word positions).
    pub len: usize,
}

impl TConfig {
    pub fn n_params(&self) -> usize {
        let per_layer = 2 * self.d + 4 * self.d * self.d + 2 * self.d + self.d * self.ff + self.ff + self.ff * self.d + self.d;
        self.vocab * self.d + self.len * self.d + self.layers * per_layer + 2 * self.d + self.d * self.classes + self.classes
    }
}

/// Where each block of parameters starts in the flat parameter vector.
struct Layer {
    g1: usize,
    b1: usize,
    wq: usize,
    wk: usize,
    wv: usize,
    wo: usize,
    g2: usize,
    b2: usize,
    w1: usize,
    c1: usize,
    w2: usize,
    c2: usize,
}

struct Layout {
    emb: usize,
    pos: usize,
    layers: Vec<Layer>,
    gf: usize,
    bf: usize,
    wc: usize,
    bc: usize,
}

fn layout(c: &TConfig) -> Layout {
    let mut at = 0usize;
    let mut take = |n: usize| {
        let s = at;
        at += n;
        s
    };
    let emb = take(c.vocab * c.d);
    let pos = take(c.len * c.d);
    let layers = (0..c.layers)
        .map(|_| Layer {
            g1: take(c.d),
            b1: take(c.d),
            wq: take(c.d * c.d),
            wk: take(c.d * c.d),
            wv: take(c.d * c.d),
            wo: take(c.d * c.d),
            g2: take(c.d),
            b2: take(c.d),
            w1: take(c.d * c.ff),
            c1: take(c.ff),
            w2: take(c.ff * c.d),
            c2: take(c.d),
        })
        .collect();
    let gf = take(c.d);
    let bf = take(c.d);
    let wc = take(c.d * c.classes);
    let bc = take(c.classes);
    Layout { emb, pos, layers, gf, bf, wc, bc }
}

#[derive(Clone)]
pub struct Transformer {
    pub cfg: TConfig,
    pub theta: Vec<f32>,
}

/// Row-major (n x k) times (k x m).
fn mm(a: &[f32], b: &[f32], n: usize, k: usize, m: usize) -> Vec<f32> {
    let mut out = vec![0.0f32; n * m];
    for i in 0..n {
        for p in 0..k {
            let x = a[i * k + p];
            if x != 0.0 {
                let row = &b[p * m..(p + 1) * m];
                let o = &mut out[i * m..(i + 1) * m];
                for j in 0..m {
                    o[j] += x * row[j];
                }
            }
        }
    }
    out
}

/// Aᵀ (k x n) times G (n x m) = (k x m): the gradient of the weights.
fn mm_at_g(a: &[f32], g: &[f32], n: usize, k: usize, m: usize) -> Vec<f32> {
    let mut out = vec![0.0f32; k * m];
    for i in 0..n {
        for p in 0..k {
            let x = a[i * k + p];
            if x != 0.0 {
                let row = &g[i * m..(i + 1) * m];
                let o = &mut out[p * m..(p + 1) * m];
                for j in 0..m {
                    o[j] += x * row[j];
                }
            }
        }
    }
    out
}

/// G (n x m) times Bᵀ (m x k) = (n x k): the gradient flowing back into the input.
fn mm_g_bt(g: &[f32], b: &[f32], n: usize, k: usize, m: usize) -> Vec<f32> {
    let mut out = vec![0.0f32; n * k];
    for i in 0..n {
        for p in 0..k {
            let row = &b[p * m..(p + 1) * m];
            let gr = &g[i * m..(i + 1) * m];
            let mut s = 0.0f32;
            for j in 0..m {
                s += gr[j] * row[j];
            }
            out[i * k + p] = s;
        }
    }
    out
}

fn charge_mm(l: &mut Ledger, n: usize, k: usize, m: usize) {
    l.macs += (n * k * m) as u64;
    l.words_read += (n * k + k * m) as u64;
    l.words_written += (n * m) as u64;
}

const LN_EPS: f32 = 1e-5;

/// What the forward pass keeps for the backward pass.
struct LayerCache {
    x_in: Vec<f32>,
    xhat1: Vec<f32>,
    inv1: Vec<f32>,
    h1: Vec<f32>,
    q: Vec<f32>,
    k: Vec<f32>,
    v: Vec<f32>,
    att: Vec<f32>, // heads x n x n
    o: Vec<f32>,
    x_mid: Vec<f32>,
    xhat2: Vec<f32>,
    inv2: Vec<f32>,
    h2: Vec<f32>,
    m1: Vec<f32>,
    r: Vec<f32>,
}

struct Cache {
    n: usize,
    ids: Vec<usize>,
    layers: Vec<LayerCache>,
    x_out: Vec<f32>,
    xhatf: Vec<f32>,
    invf: Vec<f32>,
    hf: Vec<f32>,
    pooled: Vec<f32>,
    probs: Vec<f32>,
}

fn layer_norm(x: &[f32], g: &[f32], b: &[f32], n: usize, d: usize) -> (Vec<f32>, Vec<f32>, Vec<f32>) {
    let mut y = vec![0.0f32; n * d];
    let mut xhat = vec![0.0f32; n * d];
    let mut inv = vec![0.0f32; n];
    for i in 0..n {
        let row = &x[i * d..(i + 1) * d];
        let mu = row.iter().sum::<f32>() / d as f32;
        let var = row.iter().map(|v| (v - mu) * (v - mu)).sum::<f32>() / d as f32;
        let s = 1.0 / (var + LN_EPS).sqrt();
        inv[i] = s;
        for j in 0..d {
            let h = (row[j] - mu) * s;
            xhat[i * d + j] = h;
            y[i * d + j] = g[j] * h + b[j];
        }
    }
    (y, xhat, inv)
}

/// Backward of a LayerNorm: returns dx and adds into dg, db.
fn layer_norm_back(dy: &[f32], xhat: &[f32], inv: &[f32], g: &[f32], dg: &mut [f32], db: &mut [f32], n: usize, d: usize) -> Vec<f32> {
    let mut dx = vec![0.0f32; n * d];
    for i in 0..n {
        let mut sum_dxh = 0.0f32;
        let mut sum_dxh_xh = 0.0f32;
        for j in 0..d {
            let dyj = dy[i * d + j];
            dg[j] += dyj * xhat[i * d + j];
            db[j] += dyj;
            let dxh = dyj * g[j];
            sum_dxh += dxh;
            sum_dxh_xh += dxh * xhat[i * d + j];
        }
        for j in 0..d {
            let dxh = dy[i * d + j] * g[j];
            dx[i * d + j] = inv[i] * (dxh - sum_dxh / d as f32 - xhat[i * d + j] * sum_dxh_xh / d as f32);
        }
    }
    dx
}

impl Transformer {
    pub fn new(cfg: TConfig, seed: u64) -> Transformer {
        assert!(cfg.d % cfg.heads == 0, "width must divide into heads");
        let mut rng = Rng::new(seed);
        let lay = layout(&cfg);
        let mut theta = vec![0.0f32; cfg.n_params()];
        let mut fill = |from: usize, n: usize, scale: f32, rng: &mut Rng, theta: &mut Vec<f32>| {
            for i in 0..n {
                theta[from + i] = rng.range(-scale, scale);
            }
        };
        fill(lay.emb, cfg.vocab * cfg.d, 0.3, &mut rng, &mut theta);
        fill(lay.pos, cfg.len * cfg.d, 0.1, &mut rng, &mut theta);
        let sd = 1.0 / (cfg.d as f32).sqrt();
        let sf = 1.0 / (cfg.ff as f32).sqrt();
        for l in &lay.layers {
            for g in [l.g1, l.g2] {
                for j in 0..cfg.d {
                    theta[g + j] = 1.0;
                }
            }
            for w in [l.wq, l.wk, l.wv, l.wo] {
                fill(w, cfg.d * cfg.d, sd, &mut rng, &mut theta);
            }
            fill(l.w1, cfg.d * cfg.ff, sd, &mut rng, &mut theta);
            fill(l.w2, cfg.ff * cfg.d, sf, &mut rng, &mut theta);
        }
        for j in 0..cfg.d {
            theta[lay.gf + j] = 1.0;
        }
        fill(lay.wc, cfg.d * cfg.classes, sd, &mut rng, &mut theta);
        Transformer { cfg, theta }
    }

    fn ids(&self, tokens: &[usize]) -> Vec<usize> {
        tokens.iter().take(self.cfg.len).map(|&t| if t < self.cfg.vocab { t } else { 0 }).collect()
    }

    fn forward(&self, tokens: &[usize], ledger: &mut Ledger) -> Cache {
        let c = self.cfg;
        let lay = layout(&c);
        let th = &self.theta;
        let ids = self.ids(tokens);
        let n = ids.len().max(1);
        let d = c.d;
        let dh = d / c.heads;
        let scale = 1.0 / (dh as f32).sqrt();
        let mut x = vec![0.0f32; n * d];
        for (i, &id) in ids.iter().enumerate() {
            for j in 0..d {
                x[i * d + j] = th[lay.emb + id * d + j] + th[lay.pos + i * d + j];
            }
        }
        ledger.words_read += (2 * n * d) as u64;
        ledger.words_written += (n * d) as u64;
        let mut caches = Vec::new();
        for l in &lay.layers {
            let x_in = x.clone();
            let (h1, xhat1, inv1) = layer_norm(&x, &th[l.g1..l.g1 + d], &th[l.b1..l.b1 + d], n, d);
            let q = mm(&h1, &th[l.wq..l.wq + d * d], n, d, d);
            let k = mm(&h1, &th[l.wk..l.wk + d * d], n, d, d);
            let v = mm(&h1, &th[l.wv..l.wv + d * d], n, d, d);
            for _ in 0..3 {
                charge_mm(ledger, n, d, d);
            }
            let mut att = vec![0.0f32; c.heads * n * n];
            let mut o = vec![0.0f32; n * d];
            for h in 0..c.heads {
                for i in 0..n {
                    let mut mx = f32::NEG_INFINITY;
                    for j in 0..n {
                        let mut s = 0.0f32;
                        for p in 0..dh {
                            s += q[i * d + h * dh + p] * k[j * d + h * dh + p];
                        }
                        s *= scale;
                        att[(h * n + i) * n + j] = s;
                        mx = mx.max(s);
                    }
                    let mut z = 0.0f32;
                    for j in 0..n {
                        let e = (att[(h * n + i) * n + j] - mx).exp();
                        att[(h * n + i) * n + j] = e;
                        z += e;
                    }
                    for j in 0..n {
                        att[(h * n + i) * n + j] /= z;
                    }
                    for j in 0..n {
                        let a = att[(h * n + i) * n + j];
                        for p in 0..dh {
                            o[i * d + h * dh + p] += a * v[j * d + h * dh + p];
                        }
                    }
                }
            }
            ledger.macs += (2 * c.heads * n * n * dh) as u64;
            ledger.other_ops += (c.heads * n * n) as u64;
            ledger.words_read += (c.heads * n * n * 2) as u64;
            ledger.words_written += (c.heads * n * n) as u64;
            let ao = mm(&o, &th[l.wo..l.wo + d * d], n, d, d);
            charge_mm(ledger, n, d, d);
            for i in 0..n * d {
                x[i] += ao[i];
            }
            let x_mid = x.clone();
            let (h2, xhat2, inv2) = layer_norm(&x, &th[l.g2..l.g2 + d], &th[l.b2..l.b2 + d], n, d);
            let mut m1 = mm(&h2, &th[l.w1..l.w1 + d * c.ff], n, d, c.ff);
            charge_mm(ledger, n, d, c.ff);
            for i in 0..n {
                for j in 0..c.ff {
                    m1[i * c.ff + j] += th[l.c1 + j];
                }
            }
            let r: Vec<f32> = m1.iter().map(|&v| v.max(0.0)).collect();
            let mut m2 = mm(&r, &th[l.w2..l.w2 + c.ff * d], n, c.ff, d);
            charge_mm(ledger, n, c.ff, d);
            for i in 0..n {
                for j in 0..d {
                    m2[i * d + j] += th[l.c2 + j];
                    x[i * d + j] += m2[i * d + j];
                }
            }
            ledger.other_ops += (4 * n) as u64; // layer-norm reciprocal roots
            caches.push(LayerCache { x_in, xhat1, inv1, h1, q, k, v, att, o, x_mid, xhat2, inv2, h2, m1, r });
        }
        let x_out = x.clone();
        let (hf, xhatf, invf) = layer_norm(&x, &th[lay.gf..lay.gf + d], &th[lay.bf..lay.bf + d], n, d);
        let mut pooled = vec![0.0f32; d];
        for i in 0..n {
            for j in 0..d {
                pooled[j] += hf[i * d + j] / n as f32;
            }
        }
        let mut logits = vec![0.0f32; c.classes];
        for k in 0..c.classes {
            logits[k] = th[lay.bc + k];
            for j in 0..d {
                logits[k] += pooled[j] * th[lay.wc + j * c.classes + k];
            }
        }
        ledger.macs += (d * c.classes) as u64;
        ledger.words_read += (d * c.classes + c.classes) as u64;
        ledger.other_ops += c.classes as u64;
        let mx = logits.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let mut probs: Vec<f32> = logits.iter().map(|v| (v - mx).exp()).collect();
        let z: f32 = probs.iter().sum();
        for p in &mut probs {
            *p /= z;
        }
        ledger.touch(4 * (self.theta.len() + n * d * (6 + 3 * c.layers) + c.heads * n * n));
        Cache { n, ids, layers: caches, x_out, xhatf, invf, hf, pooled, probs }
    }

    /// The answer word with the highest probability, and that probability.
    pub fn ask(&self, tokens: &[usize], ledger: &mut Ledger) -> (usize, f32) {
        let cache = self.forward(tokens, ledger);
        let (best, p) = cache.probs.iter().enumerate().fold((0usize, 0.0f32), |b, (i, &p)| if p > b.1 { (i, p) } else { b });
        (best, p)
    }

    /// Cross-entropy loss of one lesson, and its gradient added into `grad`.
    pub fn loss_and_grad(&self, tokens: &[usize], target: usize, grad: &mut [f32], ledger: &mut Ledger) -> f32 {
        let c = self.cfg;
        let lay = layout(&c);
        let th = &self.theta;
        let cache = self.forward(tokens, ledger);
        let n = cache.n;
        let d = c.d;
        let dh = d / c.heads;
        let scale = 1.0 / (dh as f32).sqrt();
        let loss = -(cache.probs[target].max(1e-12)).ln();

        // classifier
        let mut dlogits = cache.probs.clone();
        dlogits[target] -= 1.0;
        let mut dpooled = vec![0.0f32; d];
        for k in 0..c.classes {
            grad[lay.bc + k] += dlogits[k];
            for j in 0..d {
                grad[lay.wc + j * c.classes + k] += cache.pooled[j] * dlogits[k];
                dpooled[j] += th[lay.wc + j * c.classes + k] * dlogits[k];
            }
        }
        ledger.macs += (2 * d * c.classes) as u64;
        ledger.words_read += (2 * d * c.classes) as u64;
        ledger.words_written += (d * c.classes) as u64;
        // pooling
        let mut dhf = vec![0.0f32; n * d];
        for i in 0..n {
            for j in 0..d {
                dhf[i * d + j] = dpooled[j] / n as f32;
            }
        }
        // final layer norm
        let (mut dgf, mut dbf) = (vec![0.0f32; d], vec![0.0f32; d]);
        let mut dx = layer_norm_back(&dhf, &cache.xhatf, &cache.invf, &th[lay.gf..lay.gf + d], &mut dgf, &mut dbf, n, d);
        for j in 0..d {
            grad[lay.gf + j] += dgf[j];
            grad[lay.bf + j] += dbf[j];
        }
        let _ = &cache.hf;
        let _ = &cache.x_out;

        for (li, l) in lay.layers.iter().enumerate().rev() {
            let lc = &cache.layers[li];
            // feed-forward block: x = x_mid + relu(h2 W1 + c1) W2 + c2
            let dm2 = dx.clone();
            for i in 0..n {
                for j in 0..d {
                    grad[l.c2 + j] += dm2[i * d + j];
                }
            }
            let dw2 = mm_at_g(&lc.r, &dm2, n, c.ff, d);
            for (i, g) in dw2.iter().enumerate() {
                grad[l.w2 + i] += g;
            }
            let mut dr = mm_g_bt(&dm2, &th[l.w2..l.w2 + c.ff * d], n, c.ff, d);
            charge_mm(ledger, n, c.ff, d);
            charge_mm(ledger, n, c.ff, d);
            for i in 0..n * c.ff {
                if lc.m1[i] <= 0.0 {
                    dr[i] = 0.0;
                }
            }
            for i in 0..n {
                for j in 0..c.ff {
                    grad[l.c1 + j] += dr[i * c.ff + j];
                }
            }
            let dw1 = mm_at_g(&lc.h2, &dr, n, d, c.ff);
            for (i, g) in dw1.iter().enumerate() {
                grad[l.w1 + i] += g;
            }
            let dh2 = mm_g_bt(&dr, &th[l.w1..l.w1 + d * c.ff], n, d, c.ff);
            charge_mm(ledger, n, d, c.ff);
            charge_mm(ledger, n, d, c.ff);
            let (mut dg2, mut db2) = (vec![0.0f32; d], vec![0.0f32; d]);
            let dxn = layer_norm_back(&dh2, &lc.xhat2, &lc.inv2, &th[l.g2..l.g2 + d], &mut dg2, &mut db2, n, d);
            for j in 0..d {
                grad[l.g2 + j] += dg2[j];
                grad[l.b2 + j] += db2[j];
            }
            for i in 0..n * d {
                dx[i] += dxn[i];
            }
            let _ = &lc.x_mid;
            // attention block: x = x_in + (concat heads) Wo
            let dao = dx.clone();
            let dwo = mm_at_g(&lc.o, &dao, n, d, d);
            for (i, g) in dwo.iter().enumerate() {
                grad[l.wo + i] += g;
            }
            let do_ = mm_g_bt(&dao, &th[l.wo..l.wo + d * d], n, d, d);
            charge_mm(ledger, n, d, d);
            charge_mm(ledger, n, d, d);
            let mut dq = vec![0.0f32; n * d];
            let mut dk = vec![0.0f32; n * d];
            let mut dv = vec![0.0f32; n * d];
            for h in 0..c.heads {
                for i in 0..n {
                    // d attention weights
                    let mut da = vec![0.0f32; n];
                    for j in 0..n {
                        let mut s = 0.0f32;
                        for p in 0..dh {
                            s += do_[i * d + h * dh + p] * lc.v[j * d + h * dh + p];
                        }
                        da[j] = s;
                        let a = lc.att[(h * n + i) * n + j];
                        for p in 0..dh {
                            dv[j * d + h * dh + p] += a * do_[i * d + h * dh + p];
                        }
                    }
                    let dot: f32 = (0..n).map(|j| lc.att[(h * n + i) * n + j] * da[j]).sum();
                    for j in 0..n {
                        let ds = lc.att[(h * n + i) * n + j] * (da[j] - dot) * scale;
                        for p in 0..dh {
                            dq[i * d + h * dh + p] += ds * lc.k[j * d + h * dh + p];
                            dk[j * d + h * dh + p] += ds * lc.q[i * d + h * dh + p];
                        }
                    }
                }
            }
            ledger.macs += (6 * c.heads * n * n * dh) as u64;
            ledger.other_ops += (c.heads * n * n) as u64;
            ledger.words_read += (c.heads * n * n * 4) as u64;
            let mut dh1 = vec![0.0f32; n * d];
            for (w, dmat) in [(l.wq, &dq), (l.wk, &dk), (l.wv, &dv)] {
                let dw = mm_at_g(&lc.h1, dmat, n, d, d);
                for (i, g) in dw.iter().enumerate() {
                    grad[w + i] += g;
                }
                let back = mm_g_bt(dmat, &th[w..w + d * d], n, d, d);
                for i in 0..n * d {
                    dh1[i] += back[i];
                }
                charge_mm(ledger, n, d, d);
                charge_mm(ledger, n, d, d);
            }
            let (mut dg1, mut db1) = (vec![0.0f32; d], vec![0.0f32; d]);
            let dxa = layer_norm_back(&dh1, &lc.xhat1, &lc.inv1, &th[l.g1..l.g1 + d], &mut dg1, &mut db1, n, d);
            for j in 0..d {
                grad[l.g1 + j] += dg1[j];
                grad[l.b1 + j] += db1[j];
            }
            for i in 0..n * d {
                dx[i] += dxa[i];
            }
            let _ = &lc.x_in;
        }
        // embeddings
        for (i, &id) in cache.ids.iter().enumerate() {
            for j in 0..d {
                grad[lay.emb + id * d + j] += dx[i * d + j];
                grad[lay.pos + i * d + j] += dx[i * d + j];
            }
        }
        ledger.words_written += (2 * n * d) as u64;
        loss
    }

    /// Train on lessons (token ids, answer id) for up to `epochs` passes in batches, stopping early once every lesson is answered right.
    /// Returns the epochs used.
    pub fn train(&mut self, lessons: &[(Vec<usize>, usize)], epochs: usize, batch: usize, lr: f32, seed: u64, ledger: &mut Ledger) -> usize {
        let mut adam = Adam::new(self.theta.len(), lr, 0.0);
        let mut rng = Rng::new(seed);
        let mut order: Vec<usize> = (0..lessons.len()).collect();
        let mut grad = vec![0.0f32; self.theta.len()];
        for epoch in 1..=epochs {
            rng.shuffle(&mut order);
            for chunk in order.chunks(batch) {
                grad.iter_mut().for_each(|g| *g = 0.0);
                for &i in chunk {
                    self.loss_and_grad(&lessons[i].0, lessons[i].1, &mut grad, ledger);
                }
                let scale = 1.0 / chunk.len() as f32;
                grad.iter_mut().for_each(|g| *g *= scale);
                adam.step(&mut self.theta, &grad, ledger);
            }
            if epoch % 5 == 0 || epoch == epochs {
                let mut scratch = Ledger::default();
                if lessons.iter().all(|(t, a)| self.ask(t, &mut scratch).0 == *a) {
                    return epoch;
                }
            }
        }
        epochs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tiny() -> TConfig {
        TConfig { vocab: 9, classes: 5, d: 8, heads: 2, layers: 2, ff: 12, len: 6 }
    }

    #[test]
    fn the_parameter_count_matches_the_layout() {
        let c = tiny();
        let t = Transformer::new(c, 1);
        assert_eq!(t.theta.len(), c.n_params());
        let lay = layout(&c);
        assert_eq!(lay.bc + c.classes, c.n_params());
    }

    #[test]
    fn check_gradients() {
        // the backward pass against finite differences, on a handful of parameters from every block
        let c = tiny();
        let mut t = Transformer::new(c, 7);
        // move LayerNorm gains and biases off their starting values so their gradients are exercised
        let lay = layout(&c);
        for l in &lay.layers {
            for j in 0..c.d {
                t.theta[l.g1 + j] = 0.8 + 0.05 * j as f32;
                t.theta[l.b1 + j] = 0.02 * j as f32;
                t.theta[l.b2 + j] = -0.01 * j as f32;
            }
        }
        let tokens = [3usize, 1, 4, 1, 5];
        let target = 2usize;
        let mut grad = vec![0.0f32; t.theta.len()];
        let mut ledger = Ledger::default();
        t.loss_and_grad(&tokens, target, &mut grad, &mut ledger);
        let (mut num_sq_err, mut ana_sq, mut checked) = (0.0f64, 0.0f64, 0usize);
        let mut big_worst = 0.0f32;
        for idx in (0..t.theta.len()).step_by(3) {
            let eps = 1e-3f32;
            let mut plus = t.clone();
            plus.theta[idx] += eps;
            let mut minus = t.clone();
            minus.theta[idx] -= eps;
            let mut scratch = vec![0.0f32; t.theta.len()];
            let lp = plus.loss_and_grad(&tokens, target, &mut scratch, &mut Ledger::default());
            let lm = minus.loss_and_grad(&tokens, target, &mut scratch, &mut Ledger::default());
            let numeric = (lp - lm) / (2.0 * eps);
            num_sq_err += ((numeric - grad[idx]) as f64).powi(2);
            ana_sq += (grad[idx] as f64).powi(2);
            // entries that matter (not lost in float32 round-off of the finite difference) must agree closely one by one
            if grad[idx].abs() > 0.01 {
                big_worst = big_worst.max((numeric - grad[idx]).abs() / grad[idx].abs());
            }
            checked += 1;
        }
        assert!(checked > 100);
        let rel_l2 = (num_sq_err / ana_sq).sqrt();
        assert!(rel_l2 < 0.03, "relative L2 error of the whole gradient against finite differences: {rel_l2}");
        assert!(big_worst < 0.03, "worst relative error among the gradient entries above 0.01: {big_worst}");
    }

    #[test]
    fn it_learns_a_small_lookup_and_counts_the_work() {
        // five questions of two words each, each with its own answer
        let c = TConfig { vocab: 12, classes: 5, d: 16, heads: 2, layers: 1, ff: 32, len: 4 };
        let mut t = Transformer::new(c, 3);
        let lessons: Vec<(Vec<usize>, usize)> = (0..5).map(|i| (vec![1 + i, 6 + i], i)).collect();
        let mut ledger = Ledger::default();
        let epochs = t.train(&lessons, 300, 5, 0.01, 1, &mut ledger);
        assert!(epochs < 300, "should have learned in {epochs}");
        let mut scratch = Ledger::default();
        for (q, a) in &lessons {
            assert_eq!(t.ask(q, &mut scratch).0, *a);
        }
        assert!(ledger.macs > 0 && ledger.words_read > 0);
    }
}
