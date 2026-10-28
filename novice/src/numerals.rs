// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DawsMacs
//! Words that are numbers, found from the lessons: nothing here knows that "two" is a number, in any language.
//!
//! Some words are the answer of lesson after lesson, and the same words are what other questions are about ("five bidevubi zero" is "five"). If a kind made of such words
//! behaves like addition under *some* assignment of whole numbers to the words, that assignment is what the words mean. It is found the way the counting
//! is: one word is nothing (adding it changes nothing), another is the unit, and every lesson of the kind ("one and two make three") then fixes more
//! words from the ones already fixed, until every lesson of the kind holds or one fails. The assignment is kept only if it is the same for every
//! kind that adds. Everything else (subtracting, multiplying, the biggest of two) is then learned by the learner as it learns on written numbers.
//!
//! Limits, plainly: addition is the structure looked for (it is the one that fixes a unit; a kind that is only an order, like the larger of two,
//! does not), and a number is a word only if some lesson says it.

use std::collections::{BTreeMap, BTreeSet};

/// A word is a candidate number only if it is the whole answer of at least this many lessons (one fact, in its three wordings).
const MIN_ANSWERS: u32 = 3;
/// Fewest lessons of one kind to look for addition in it.
const MIN_LESSONS: usize = 8;
/// Fewest words a vocabulary of numbers must have.
const MIN_WORDS: usize = 4;

/// How the words of a family make a number.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Grammar {
    /// Every word is a number of its own ("five").
    #[default]
    Single,
    /// A run of words is the sum of its words ("twenty three").
    Additive,
    /// A run of words is a number written in this base, the first word the most significant digit.
    Positional(i64),
    /// The same, the first word the least significant digit.
    LeastFirst(i64),
    /// Three words, the first the least significant digit, with a radix for each place.
    Mixed([i64; 3]),
    /// A number so far times the base plus the next word's number, whatever whole numbers the words stand for (digits from one to the base, as in a
    /// numeration with no zero digit, or from minus a few to a few), the first word the most significant digit unless `least_first`.
    Fold { base: i64, least_first: bool },
    /// A word for each digit, a word for the base and one for its square that multiply the digit word before them (none means one), the parts added
    /// ("two ten three" is 23); `minus_from` is the first last digit that is said as the next ten minus what is missing (0: no such word).
    Multiplier { base: i64, minus_from: i64 },
}

/// The words of one family of number words: what each means, and how a run of them makes a number.
#[derive(Clone, Debug, Default)]
pub struct Family {
    values: BTreeMap<String, i64>,
    grammar: Grammar,
    /// For single words: the word for each number.
    spoken: BTreeMap<i64, String>,
    /// The word for "minus", if the grammar has one.
    minus: Option<String>,
    /// How often each word is used in an answer: where two words mean the same, the answers use the more common.
    used: BTreeMap<String, usize>,
    /// For runs: the largest number the lessons showed, and whether saying numbers back reproduced every one the lessons said.
    max_spoken: i64,
    speaks_runs: bool,
    /// The most words in any numeral the lessons showed: a longer run is a structure nobody showed.
    max_len: usize,
}

fn pow(b: i64, e: usize) -> i64 {
    (0..e).fold(1i64, |acc, _| acc.saturating_mul(b))
}

/// The digits, least significant first, of `n` in `base` using only the `digits` (tried in the order given), at most `depth` of them.
fn fold_digits(n: i64, base: i64, digits: &[i64], depth: usize) -> Option<Vec<i64>> {
    if n == 0 {
        return Some(Vec::new());
    }
    if depth == 0 {
        return None;
    }
    for &d in digits {
        if (n - d).rem_euclid(base) == 0 {
            if let Some(mut tail) = fold_digits((n - d) / base, base, digits, depth - 1) {
                tail.insert(0, d);
                return Some(tail);
            }
        }
    }
    None
}

impl Family {
    fn get(&self, word: &str) -> Option<i64> {
        self.values.get(word).copied()
    }

    fn is_minus(&self, word: &str) -> bool {
        self.minus.as_deref() == Some(word)
    }

    /// The number a run of words makes under the grammar, if every word is in the family.
    pub fn value_of(&self, run: &[String]) -> Option<i64> {
        if run.is_empty() || (self.max_len > 0 && run.len() > self.max_len) {
            return None;
        }
        match self.grammar {
            Grammar::Single => (run.len() == 1).then(|| self.get(&run[0])).flatten(),
            Grammar::Additive => run.iter().map(|w| self.get(w)).sum::<Option<i64>>(),
            Grammar::Positional(_) if run.len() > 1 && self.get(&run[0]) == Some(0) => None,
            Grammar::LeastFirst(_) if run.len() > 1 && self.get(&run[run.len() - 1]) == Some(0) => None,
            Grammar::Positional(b) => run.iter().try_fold(0i64, |acc, w| {
                let d = self.get(w)?;
                (d < b).then(|| acc.checked_mul(b)?.checked_add(d)).flatten()
            }),
            Grammar::LeastFirst(b) => run.iter().enumerate().try_fold(0i64, |acc, (i, w)| {
                let d = self.get(w)?;
                (d < b).then(|| acc.checked_add(d.checked_mul(pow(b, i))?)).flatten()
            }),
            Grammar::Fold { base, least_first } => {
                let mut digits: Vec<i64> = run.iter().map(|w| self.get(w)).collect::<Option<_>>()?;
                if least_first {
                    digits.reverse();
                }
                // (no numeral begins with a digit that is not a positive number)
                if digits.len() > 1 && digits[0] <= 0 {
                    return None;
                }
                let v = digits.iter().try_fold(0i64, |acc, d| acc.checked_mul(base)?.checked_add(*d))?;
                (v >= 0).then_some(v)
            }
            Grammar::Mixed(r) => {
                if run.len() != 3 {
                    return None;
                }
                let d: Vec<i64> = run.iter().map(|w| self.get(w)).collect::<Option<_>>()?;
                (d[0] < r[0] && d[1] < r[1] && d[2] < r[2]).then(|| d[0] + r[0] * d[1] + r[0] * r[1] * d[2])
            }
            Grammar::Multiplier { base, .. } => match run.iter().position(|w| self.is_minus(w)) {
                Some(at) => {
                    let (left, right) = (self.multiplier_value(&run[..at], base)?, self.multiplier_value(&run[at + 1..], base)?);
                    (left > right).then_some(left - right)
                }
                None => self.multiplier_value(run, base),
            },
        }
    }

    fn multiplier_value(&self, run: &[String], base: i64) -> Option<i64> {
        if run.is_empty() {
            return None;
        }
        // The parts come from the largest unit down, each at most once, and a last digit by itself.
        let (mut total, mut pending, mut last_rank) = (0i64, None, 3);
        for w in run {
            let v = self.get(w)?;
            if v == 0 {
                return (run.len() == 1).then_some(0);
            } else if v == base || v == base * base {
                let rank = if v == base { 1 } else { 2 };
                if rank >= last_rank {
                    return None;
                }
                last_rank = rank;
                total += pending.take().unwrap_or(1) * v;
            } else if v >= 1 && v < base {
                if pending.is_some() {
                    return None;
                }
                pending = Some(v);
            } else {
                return None;
            }
        }
        Some(total + pending.unwrap_or(0))
    }

    /// The word the lessons use for a number (or a digit): of the words with that meaning, the one the answers use most.
    fn word_for(&self, v: i64) -> Option<String> {
        self.values.iter().filter(|(_, x)| **x == v).max_by_key(|(w, _)| (self.used.get(*w).copied().unwrap_or(0), std::cmp::Reverse((*w).clone()))).map(|(w, _)| w.clone())
    }

    fn compose(&self, n: i64) -> Option<String> {
        if n < 0 || n > self.max_spoken {
            return None;
        }
        let digits_msd = |b: i64| -> Vec<i64> {
            let (mut out, mut rest) = (Vec::new(), n);
            loop {
                out.push(rest % b);
                rest /= b;
                if rest == 0 {
                    break;
                }
            }
            out
        };
        let say = |digits: Vec<i64>| digits.iter().map(|&d| self.word_for(d)).collect::<Option<Vec<_>>>().map(|w| w.join(" "));
        match self.grammar {
            Grammar::Single => self.spoken.get(&n).cloned(),
            Grammar::Additive => {
                if n == 0 {
                    return self.word_for(0);
                }
                let mut rest = n;
                let mut out: Vec<String> = Vec::new();
                while rest > 0 {
                    let (w, v) = self.values.iter().filter(|(_, v)| **v > 0 && **v <= rest).max_by_key(|(w, v)| (**v, std::cmp::Reverse((*w).clone())))?;
                    out.push(w.clone());
                    rest -= v;
                }
                Some(out.join(" "))
            }
            Grammar::Positional(b) => say(digits_msd(b).into_iter().rev().collect()),
            Grammar::Fold { base, least_first } => {
                if n == 0 {
                    return self.word_for(0);
                }
                let mut digits: Vec<i64> = self.values.values().copied().collect::<BTreeSet<_>>().into_iter().collect();
                digits.sort_by_key(|d| (std::cmp::Reverse(d.abs()), std::cmp::Reverse(*d)));
                let mut least_first_digits = fold_digits(n, base, &digits, self.max_len + 1)?;
                if least_first_digits.last().is_none_or(|d| *d <= 0) {
                    return None;
                }
                if !least_first {
                    least_first_digits.reverse();
                }
                say(least_first_digits)
            }
            Grammar::LeastFirst(b) => say(digits_msd(b)),
            Grammar::Mixed(r) => {
                let (d0, rest) = (n % r[0], n / r[0]);
                let (d1, d2) = (rest % r[1], rest / r[1]);
                (d2 < r[2]).then(|| say(vec![d0, d1, d2])).flatten()
            }
            Grammar::Multiplier { base, minus_from } => self.say_multiplier(n, base, minus_from),
        }
    }

    fn say_multiplier(&self, n: i64, base: i64, minus_from: i64) -> Option<String> {
        if n == 0 {
            return self.word_for(0);
        }
        let digit = |d: i64| self.word_for(d);
        // The last digit is said as the next ten minus what is missing.
        if minus_from > 0 && n % base >= minus_from {
            let missing = base - n % base;
            let up = self.say_multiplier(n + missing, base, minus_from)?;
            return Some(format!("{up} {} {}", self.minus.clone()?, digit(missing)?));
        }
        let (squares, rest) = (n / (base * base), n % (base * base));
        let (tens, ones) = (rest / base, rest % base);
        let mut out: Vec<String> = Vec::new();
        for (count, unit) in [(squares, base * base), (tens, base)] {
            if count > 0 {
                if count > 1 {
                    out.push(digit(count)?);
                }
                out.push(self.word_for(unit)?);
            }
        }
        if ones > 0 {
            out.push(digit(ones)?);
        }
        Some(out.join(" "))
    }
}

/// The words that are numbers, in as many families as the lessons show: what each means, and how a run of them makes one number (and how a number
/// is said again). A word is in at most one family.
#[derive(Clone, Debug, Default)]
pub struct Lexicon {
    families: Vec<Family>,
}

impl Lexicon {
    /// Words that are one number each.
    pub fn single(values: BTreeMap<String, i64>) -> Lexicon {
        if values.is_empty() {
            return Lexicon::default();
        }
        let spoken = values.iter().map(|(w, v)| (*v, w.clone())).collect();
        Lexicon { families: vec![Family { values, grammar: Grammar::Single, spoken, ..Family::default() }] }
    }

    pub fn is_empty(&self) -> bool {
        self.families.is_empty()
    }

    pub fn families(&self) -> usize {
        self.families.len()
    }

    pub fn grammar(&self, family: usize) -> Grammar {
        self.families[family].grammar
    }

    pub fn len(&self) -> usize {
        self.families.iter().map(|f| f.values.len() + usize::from(f.minus.is_some())).sum()
    }

    /// The family a word is in and, for a family of single words, its number; for a family that makes numbers from runs, any number (its place in a run
    /// is what counts).
    pub fn owner(&self, word: &str) -> Option<(usize, i64)> {
        self.families.iter().enumerate().find_map(|(i, f)| f.get(word).or_else(|| f.is_minus(word).then_some(0)).map(|v| (i, v)))
    }

    pub fn get(&self, word: &str) -> Option<i64> {
        self.owner(word).map(|(_, v)| v)
    }

    /// The number a run of words of `family` makes.
    pub fn value_of_run(&self, family: usize, run: &[String]) -> Option<i64> {
        self.families.get(family)?.value_of(run)
    }

    /// The number a whole text is made of and the family it is in, if all of it is number words of one family.
    pub fn value_of_text(&self, text: &str) -> Option<(i64, usize)> {
        let words = tokens(text);
        let (family, _) = self.owner(words.first()?)?;
        if words.iter().any(|w| self.owner(w).map(|(i, _)| i) != Some(family)) {
            return None;
        }
        self.value_of_run(family, &words).map(|v| (v, family))
    }

    /// The words for a number in `family`: the word the lessons used for it, or for a run grammar the words that make it, within the range the lessons
    /// showed and only if saying back the numbers of the lessons gave exactly what the lessons said.
    pub fn speak(&self, family: usize, n: i64) -> Option<String> {
        let f = self.families.get(family)?;
        match f.grammar {
            Grammar::Single => f.spoken.get(&n).cloned(),
            _ if !f.speaks_runs => None,
            _ => f.compose(n),
        }
    }
}

fn tokens(text: &str) -> Vec<String> {
    text.split_whitespace().map(|t| t.trim_matches(|c: char| matches!(c, '?' | ',' | '.' | '!' | ';' | ':')).to_lowercase()).filter(|t| !t.is_empty()).collect()
}

/// `(a, b, c)`: the two words in the question and the word of the answer.
type Triple = (String, String, String);

/// Fix the words of `triples` as whole numbers, starting from `known`, so that `a + b = c` in every triple; none if that cannot be done
/// (a lesson that fails, two words with one number, a number below zero).
fn propagate(triples: &[Triple], known: &BTreeMap<String, i64>) -> Option<BTreeMap<String, i64>> {
    let mut sigma = known.clone();
    let set = |sigma: &mut BTreeMap<String, i64>, w: &str, v: i64| -> Option<bool> {
        if v < 0 {
            return None;
        }
        match sigma.get(w) {
            Some(&old) => (old == v).then_some(false),
            None => {
                if sigma.values().any(|&x| x == v) {
                    return None;
                }
                sigma.insert(w.to_string(), v);
                Some(true)
            }
        }
    };
    loop {
        let mut changed = false;
        for (a, b, c) in triples {
            let (sa, sb, sc) = (sigma.get(a).copied(), sigma.get(b).copied(), sigma.get(c).copied());
            match (sa, sb, sc) {
                (Some(x), Some(y), _) => changed |= set(&mut sigma, c, x + y)?,
                (Some(x), None, Some(z)) => changed |= set(&mut sigma, b, z - x)?,
                (None, Some(y), Some(z)) => changed |= set(&mut sigma, a, z - y)?,
                _ => {}
            }
        }
        if !changed {
            return Some(sigma);
        }
    }
}

/// Largest number a word may be given when a lesson does not fix it.
const MAX_VALUE: i64 = 40;
/// Most assignments looked at when a lesson set leaves words free, and most steps of that search.
const MAX_SOLUTIONS: usize = 40;
const MAX_STEPS: usize = 200_000;

/// Every way (up to a few) to give all the words of `triples` a number so that every lesson adds, starting from `known`. Words that are free are tried
/// from the smallest number up; the search stops at a limit, and the caller keeps only what every solution found agrees on.
fn solutions(triples: &[Triple], known: &BTreeMap<String, i64>) -> Vec<BTreeMap<String, i64>> {
    let mut words: BTreeMap<&str, usize> = BTreeMap::new();
    for (a, b, c) in triples {
        for w in [a, b, c] {
            *words.entry(w).or_insert(0) += 1;
        }
    }
    let mut out = Vec::new();
    let mut steps = 0usize;
    fn go(triples: &[Triple], sigma: BTreeMap<String, i64>, words: &BTreeMap<&str, usize>, out: &mut Vec<BTreeMap<String, i64>>, steps: &mut usize) {
        if out.len() >= MAX_SOLUTIONS || *steps > MAX_STEPS {
            return;
        }
        *steps += 1;
        let Some(sigma) = propagate(triples, &sigma) else { return };
        // The free word that is in most lessons.
        let free = words.iter().filter(|(w, _)| !sigma.contains_key(**w)).max_by_key(|(w, n)| (**n, std::cmp::Reverse(**w)));
        let Some((word, _)) = free else {
            out.push(sigma);
            return;
        };
        for v in 0..=MAX_VALUE {
            if sigma.values().any(|&x| x == v) {
                continue;
            }
            let mut next = sigma.clone();
            next.insert((*word).to_string(), v);
            go(triples, next, words, out, steps);
        }
    }
    go(triples, known.clone(), &words, &mut out, &mut steps);
    out
}

/// The numbers every solution agrees on.
fn consensus(solved: &[BTreeMap<String, i64>]) -> Option<BTreeMap<String, i64>> {
    let first = solved.first()?;
    Some(first.iter().filter(|(w, v)| solved.iter().all(|s| s.get(*w) == Some(*v))).map(|(w, v)| (w.clone(), *v)).collect())
}

/// The meaning of each number word the lessons give: word to whole number. Empty if no kind of the lessons adds.
pub fn learn(lessons: &[(String, String)]) -> BTreeMap<String, i64> {
    let mut answers: BTreeMap<String, u32> = BTreeMap::new();
    let parsed: Vec<(Vec<String>, String)> = lessons.iter().map(|(q, a)| (tokens(q), a.trim().to_lowercase())).collect();
    for (_, a) in &parsed {
        if !a.is_empty() && !a.contains(char::is_whitespace) {
            *answers.entry(a.clone()).or_insert(0) += 1;
        }
    }
    // (Written numbers are numbers already.)
    let candidates: BTreeSet<String> = answers.iter().filter(|(w, &n)| n >= MIN_ANSWERS && !w.bytes().all(|b| b.is_ascii_digit())).map(|(w, _)| w.clone()).collect();
    if candidates.len() < MIN_WORDS {
        return BTreeMap::new();
    }
    // The kinds made of these words: two of them in the question, one as the answer. Which word of the question names the kind is not known (the frame
    // words are in every lesson, the kind's own word in all of its lessons), so every word that is not a number is tried as the name: a group made of a
    // frame word mixes kinds and fails to add up; the group of the kind's own word is the whole kind.
    let mut kinds: BTreeMap<String, Vec<Triple>> = BTreeMap::new();
    for (q, a) in &parsed {
        let numbers: Vec<&String> = q.iter().filter(|w| candidates.contains(*w)).collect();
        if numbers.len() != 2 || !candidates.contains(a) {
            continue;
        }
        let names: BTreeSet<&String> = q.iter().filter(|w| !candidates.contains(*w)).collect();
        for kind in names {
            kinds.entry(kind.clone()).or_default().push((numbers[0].clone(), numbers[1].clone(), a.clone()));
        }
    }
    // Start from the kind and the two words that fix the most words.
    let mut best: Option<BTreeMap<String, i64>> = None;
    let better = |new: &BTreeMap<String, i64>, old: &Option<BTreeMap<String, i64>>| match old {
        None => true,
        Some(o) => (new.len(), std::cmp::Reverse(new.values().max())) > (o.len(), std::cmp::Reverse(o.values().max())),
    };
    for triples in kinds.values().filter(|t| t.len() >= MIN_LESSONS) {
        for zero in &candidates {
            // Adding the word changes nothing, in every lesson that has it.
            let with_zero: Vec<&Triple> = triples.iter().filter(|(a, b, _)| a == zero || b == zero).collect();
            if with_zero.len() < 2 || !with_zero.iter().all(|(a, b, c)| (a == zero && c == b) || (b == zero && c == a)) {
                continue;
            }
            for one in candidates.iter().filter(|w| *w != zero) {
                let start: BTreeMap<String, i64> = [(zero.clone(), 0), (one.clone(), 1)].into_iter().collect();
                if let Some(sigma) = consensus(&solutions(triples, &start)) {
                    if better(&sigma, &best) {
                        best = Some(sigma);
                    }
                }
            }
        }
    }
    let Some(mut sigma) = best else { return BTreeMap::new() };
    // Every other kind that adds under this assignment says more words: the union of their lessons is fixed together.
    let adds = |triples: &[Triple], sigma: &BTreeMap<String, i64>| {
        let known: Vec<&Triple> = triples.iter().filter(|(a, b, c)| sigma.contains_key(a) && sigma.contains_key(b) && sigma.contains_key(c)).collect();
        let distinct: BTreeSet<&Triple> = known.iter().copied().collect();
        distinct.len() >= 3 && known.iter().all(|(a, b, c)| sigma[a] + sigma[b] == sigma[c])
    };
    let mut union: Vec<Triple> = Vec::new();
    for triples in kinds.values().filter(|t| t.len() >= MIN_LESSONS) {
        if adds(triples, &sigma) {
            union.extend(triples.iter().cloned());
        }
    }
    if let Some(more) = consensus(&solutions(&union, &sigma)) {
        sigma = more;
    }
    if sigma.len() < MIN_WORDS {
        return BTreeMap::new();
    }
    sigma
}

/// Fewest lessons of one kind to look for a reading in.
const MIN_READING: usize = 20;
/// Largest base looked for.
const MAX_BASE: i64 = 24;

/// What a run of words in position `i` of `len` weighs in a number under `grammar`, and the number its meaning must stay below there (None: not a
/// grammar of weighted places).
fn places(grammar: Grammar, len: usize) -> Option<Vec<(i64, i64)>> {
    match grammar {
        Grammar::Additive => Some(vec![(1, i64::MAX); len]),
        Grammar::Positional(b) => Some((0..len).map(|i| (pow(b, len - 1 - i), b)).collect()),
        Grammar::LeastFirst(b) => Some((0..len).map(|i| (pow(b, i), b)).collect()),
        Grammar::Mixed(r) => (len == 3).then(|| vec![(1, r[0]), (r[0], r[1]), (r[0] * r[1], r[2])]),
        Grammar::Fold { base, least_first } => Some((0..len).map(|i| (pow(base, if least_first { i } else { len - 1 - i }), i64::MAX)).collect()),
        _ => None,
    }
}

/// Give the words of `equations` (a run of words, the number it makes) the numbers that make every equation hold under a grammar of weighted places,
/// by fixing a word wherever an equation has only one word left that is not fixed. None if an equation fails or a word would need a number that is not
/// whole, below zero or too large for its place.
fn fix_runs(equations: &[(Vec<String>, i64)], grammar: Grammar) -> Option<BTreeMap<String, i64>> {
    let signed = matches!(grammar, Grammar::Fold { .. });
    let mut known: BTreeMap<String, i64> = BTreeMap::new();
    loop {
        let mut changed = false;
        for (run, n) in equations {
            let w = places(grammar, run.len())?;
            let mut sum = 0i64;
            let mut open: Vec<usize> = Vec::new();
            for (i, t) in run.iter().enumerate() {
                match known.get(t) {
                    Some(v) if *v < w[i].1 => sum += w[i].0 * v,
                    Some(_) => return None,
                    None => open.push(i),
                }
            }
            if open.is_empty() {
                if sum != *n {
                    return None;
                }
                continue;
            }
            if open.iter().any(|&i| run[i] != run[open[0]]) {
                continue;
            }
            let weight: i64 = open.iter().map(|&i| w[i].0).sum();
            let rest = n - sum;
            if (rest < 0 && !signed) || rest % weight != 0 {
                return None;
            }
            let v = rest / weight;
            if open.iter().any(|&i| v >= w[i].1) {
                return None;
            }
            known.insert(run[open[0]].clone(), v);
            changed = true;
        }
        if !changed {
            return Some(known);
        }
    }
}

/// The families of words whose runs make numbers by a base word and a multiplier, for each base that could be. The one-word numbers fix the words said
/// alone; a word that is left is fixed by a run that has no other word not yet fixed, to a digit, the base or its square, if only one of them makes the
/// run hold; one word left over is "minus".
fn multiplier_family(equations: &[(Vec<String>, i64)]) -> Vec<Family> {
    let mut singles: BTreeMap<String, i64> = BTreeMap::new();
    for (run, n) in equations {
        if run.len() == 1 {
            match singles.get(&run[0]) {
                Some(v) if v != n => return Vec::new(),
                _ => {
                    singles.insert(run[0].clone(), *n);
                }
            }
        }
    }
    let mut out = Vec::new();
    for base in 2..=MAX_BASE {
        let mut values = singles.clone();
        if singles.values().any(|v| *v > 1 && *v != base && *v != base * base && *v >= base) {
            continue;
        }
        loop {
            let mut changed = false;
            for (run, n) in equations {
                let open: BTreeSet<&String> = run.iter().filter(|w| !values.contains_key(*w)).collect();
                if open.len() != 1 {
                    continue;
                }
                let u = (*open.iter().next().unwrap()).clone();
                let fits: Vec<i64> = (1..base).chain([base, base * base])
                    .filter(|&v| {
                        let mut trial = values.clone();
                        trial.insert(u.clone(), v);
                        let f = Family { values: trial, grammar: Grammar::Multiplier { base, minus_from: 0 }, ..Family::default() };
                        f.value_of(run) == Some(*n)
                    })
                    .collect();
                if fits.len() == 1 {
                    values.insert(u, fits[0]);
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
        let unknown: BTreeSet<&String> = equations.iter().flat_map(|(r, _)| r.iter()).filter(|w| !values.contains_key(*w)).collect();
        let minus = match unknown.len() {
            0 => None,
            1 => Some((*unknown.iter().next().unwrap()).clone()),
            _ => continue,
        };
        out.push(Family { values, grammar: Grammar::Multiplier { base, minus_from: 0 }, minus, ..Family::default() });
    }
    out
}

/// Number words found from a kind that reads them: its questions are made of number words and its answers are written numbers ("twenty three" is
/// 23). Which words of a question are the number is not known; they are what is left when the words that every lesson of the kind has (its name
/// and its frame) are taken away from what all the wordings of one fact have in common. The meaning of each word is then fixed the way the counting
/// is: a number of one word fixes it, a longer one fixes its last unknown word. Several ways of making a number from a run of words are tried (the
/// sum of its words, its words as the digits of a number in some base, most or least significant first, three words of a mixed radix, and words that
/// multiply a base word, with a word for "minus" or without); the first that holds in every lesson is kept. Each kind that reads gives a family;
/// families that share no word are all kept. It is then required that saying the numbers back reproduces what the lessons said (their answers where
/// they spell numbers), or the family only reads.
pub fn learn_reading(lessons: &[(String, String)]) -> Lexicon {
    let reading: Vec<(Vec<String>, i64)> = lessons
        .iter()
        .filter_map(|(q, a)| {
            let a = a.trim();
            let words = tokens(q);
            let written = !a.is_empty() && a.len() <= 6 && a.bytes().all(|b| b.is_ascii_digit());
            let digit_free = !words.iter().any(|w| w.bytes().any(|b| b.is_ascii_digit()));
            (written && digit_free && !words.is_empty()).then(|| (words, a.parse().unwrap_or(0)))
        })
        .collect();
    let mut used: BTreeMap<String, usize> = BTreeMap::new();
    for (_, a) in lessons {
        for w in tokens(a) {
            *used.entry(w).or_insert(0) += 1;
        }
    }
    let mut df: BTreeMap<&str, usize> = BTreeMap::new();
    for (words, _) in &reading {
        for w in words.iter().collect::<BTreeSet<_>>() {
            *df.entry(w.as_str()).or_insert(0) += 1;
        }
    }
    let names: Vec<&str> = df.iter().filter(|(_, n)| **n >= MIN_READING).map(|(w, _)| *w).collect();
    let mut found: Vec<Family> = Vec::new();
    for name in names {
        let kind: Vec<&(Vec<String>, i64)> = reading.iter().filter(|(w, _)| w.iter().any(|x| x == name)).collect();
        let common: BTreeSet<&String> = kind.iter().map(|(w, _)| w.iter().collect::<BTreeSet<_>>()).reduce(|a, b| a.intersection(&b).copied().collect()).unwrap_or_default();
        let mut by_value: BTreeMap<i64, Vec<&Vec<String>>> = BTreeMap::new();
        for (w, v) in &kind {
            by_value.entry(*v).or_default().push(w);
        }
        // One equation per fact asked in at least two wordings: the words all its wordings share, other than the words of the whole kind.
        let mut equations: Vec<(Vec<String>, i64)> = Vec::new();
        for (v, wordings) in &by_value {
            if wordings.len() < 2 {
                continue;
            }
            let shared: BTreeSet<&String> = wordings.iter().map(|w| w.iter().collect::<BTreeSet<_>>()).reduce(|a, b| a.intersection(&b).copied().collect()).unwrap_or_default();
            let run: Vec<String> = wordings[0].iter().filter(|w| shared.contains(w) && !common.contains(w)).cloned().collect();
            if !run.is_empty() {
                equations.push((run, *v));
            }
        }
        let distinct: BTreeSet<i64> = equations.iter().map(|(_, v)| *v).collect();
        if equations.len() < MIN_READING / 2 || distinct.len() < 10 {
            continue;
        }
        let mut candidates: Vec<Family> = Vec::new();
        let weighted = std::iter::once(Grammar::Additive)
            .chain((2..=MAX_BASE).map(Grammar::Positional))
            .chain((2..=MAX_BASE).map(Grammar::LeastFirst))
            .chain((2..=MAX_BASE).flat_map(|b| [Grammar::Fold { base: b, least_first: false }, Grammar::Fold { base: b, least_first: true }]))
            .chain(equations.iter().all(|(r, _)| r.len() == 3).then(|| (2..=MAX_BASE).flat_map(|a| (2..=MAX_BASE).flat_map(move |b| (2..=MAX_BASE).map(move |c| Grammar::Mixed([a, b, c]))))).into_iter().flatten());
        for grammar in weighted {
            if let Some(values) = fix_runs(&equations, grammar) {
                candidates.push(Family { values, grammar, ..Family::default() });
            }
            if candidates.len() >= 4 {
                break;
            }
        }
        candidates.extend(multiplier_family(&equations));
        for mut family in candidates {
            family.used = used.clone();
            family.max_len = equations.iter().map(|(r, _)| r.len()).max().unwrap_or(0);
            family.max_spoken = equations.iter().map(|(_, v)| *v).max().unwrap_or(0);
            let reads = |f: &Family| equations.iter().filter(|(run, _)| run.iter().all(|w| f.values.contains_key(w) || f.is_minus(w))).all(|(run, v)| f.value_of(run) == Some(*v));
            let covered = equations.iter().filter(|(run, _)| run.iter().all(|w| family.values.contains_key(w) || family.is_minus(w))).count();
            if family.values.len() < MIN_WORDS || covered * 10 < equations.len() * 9 || !reads(&family) {
                continue;
            }
            // What the lessons say in this family's words, for checking that the numbers come back as they were said.
            let said: Vec<(String, i64)> = lessons
                .iter()
                .map(|(_, a)| a.trim().to_lowercase())
                .filter_map(|a| {
                    let words = tokens(&a);
                    (!words.is_empty() && words.iter().all(|w| family.values.contains_key(w) || family.is_minus(w))).then(|| family.value_of(&words).map(|v| (a, v))).flatten()
                })
                .collect();
            family.max_len = said.iter().map(|(t, _)| tokens(t).len()).chain(std::iter::once(family.max_len)).max().unwrap_or(0);
            family.max_spoken = said.iter().map(|(_, v)| *v).chain(std::iter::once(family.max_spoken)).max().unwrap_or(0);
            let to_check: Vec<(String, i64)> = if said.is_empty() { equations.iter().map(|(r, v)| (r.join(" "), *v)).collect() } else { said };
            let thresholds: Vec<i64> = match family.grammar {
                Grammar::Multiplier { .. } if family.minus.is_some() => (6..=9).collect(),
                _ => vec![0],
            };
            let mut chosen = None;
            for t in thresholds {
                let mut trial = family.clone();
                if let Grammar::Multiplier { base, .. } = trial.grammar {
                    trial.grammar = Grammar::Multiplier { base, minus_from: if trial.minus.is_some() { t } else { 0 } };
                }
                trial.speaks_runs = true;
                if to_check.iter().all(|(text, v)| trial.compose(*v).as_deref() == Some(text.as_str())) {
                    chosen = Some(trial);
                    break;
                }
            }
            match chosen {
                Some(f) => found.push(f),
                None => {
                    family.speaks_runs = false;
                    found.push(family);
                }
            }
            break;
        }
    }
    // Largest families first; a family that shares a word with a larger one is dropped.
    found.sort_by(|a, b| b.values.len().cmp(&a.values.len()));
    let mut kept: Vec<Family> = Vec::new();
    for f in found {
        if kept.iter().all(|k| f.values.keys().all(|w| !k.values.contains_key(w))) {
            kept.push(f);
        }
    }
    Lexicon { families: kept }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WORDS: [&str; 11] = ["zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten"];

    /// Lessons of a kind `name` over `pairs`, each in the three wordings the exams use, answering with `rule`.
    fn lessons_of(name: &str, pairs: &[(usize, usize)], rule: impl Fn(usize, usize) -> Option<usize>) -> Vec<(String, String)> {
        let mut out = Vec::new();
        for &(a, b) in pairs {
            if let Some(c) = rule(a, b) {
                for template in ["what is {a} {k} {b}", "{a} {k} {b}", "{a} {k} {b} equals"] {
                    out.push((template.replace("{a}", WORDS[a]).replace("{k}", name).replace("{b}", WORDS[b]), WORDS[c].to_string()));
                }
            }
        }
        out
    }

    fn pairs_up_to(n: usize) -> Vec<(usize, usize)> {
        (0..=n).flat_map(|a| (0..=n).map(move |b| (a, b))).filter(|&(a, b)| (a, b) == (0, 0) || (a * 3 + b * 5) % 4 != 0).collect()
    }

    #[test]
    fn the_words_of_a_kind_that_adds_are_fixed_as_numbers() {
        let mut lessons = lessons_of("bidevubi", &pairs_up_to(5), |a, b| Some(a + b));
        // Kinds about things: an invented answer that repeats, and words that are only questions.
        for (i, w) in ["bafoba", "tadaba", "podoba", "mifuba", "ligaba", "nufeba", "pefoba"].iter().enumerate() {
            for template in ["how is {w} sedavubi", "{w} is sedavubi how", "sedavubi {w}"] {
                lessons.push((template.replace("{w}", w), ["vetupa", "bevapa", "zovapa", "vetupa", "bevapa", "zovapa", "vetupa"][i].to_string()));
            }
        }
        let sigma = learn(&lessons);
        for (i, w) in WORDS.iter().enumerate().take(11) {
            if let Some(&v) = sigma.get(*w) {
                assert_eq!(v, i as i64, "{w}");
            }
        }
        assert!(sigma.len() >= 8, "zero to at least seven: {sigma:?}");
        assert_eq!(sigma["zero"], 0);
        assert_eq!(sigma["six"], 6, "six is only ever an answer, and still fixed");
        assert!(!sigma.contains_key("vetupa") && !sigma.contains_key("sedavubi"), "{sigma:?}");
    }

    #[test]
    fn a_second_kind_that_adds_with_other_pairs_fixes_words_the_first_leaves_out() {
        let mut lessons = lessons_of("nifavubi", &pairs_up_to(3), |a, b| Some(a + b));
        lessons.extend(lessons_of("pudivubi", &[(5, 4), (4, 4), (5, 3), (4, 3), (5, 5), (1, 0), (0, 2), (0, 0), (0, 4)], |a, b| Some(a + b)));
        let sigma = learn(&lessons);
        assert_eq!(sigma.get("ten"), Some(&10));
        assert_eq!(sigma.get("nine"), Some(&9));
    }

    #[test]
    fn kinds_that_are_not_addition_fix_nothing() {
        // Only the larger of two and the absolute difference: an order, and a distance, but no unit and no zero that changes nothing.
        let mut lessons = lessons_of("dudovubi", &pairs_up_to(5), |a, b| Some(a.max(b)));
        lessons.extend(lessons_of("dufavubi", &pairs_up_to(5), |a, b| Some(a.abs_diff(b))));
        assert!(learn(&lessons).is_empty());
    }

    #[test]
    fn numbers_written_with_digits_are_not_words_to_learn() {
        let mut lessons = Vec::new();
        for a in 0..8usize {
            for b in 0..8usize {
                for template in ["what is {a} fegisido {b}", "{a} fegisido {b}", "{a} fegisido {b} equals"] {
                    lessons.push((template.replace("{a}", &a.to_string()).replace("{b}", &b.to_string()), (a + b).to_string()));
                }
            }
        }
        assert!(learn(&lessons).is_empty());
    }

    #[test]
    fn no_numbers_without_lessons_that_say_so() {
        assert!(learn(&[]).is_empty());
        let few = lessons_of("bidevubi", &[(1, 2), (2, 2)], |a, b| Some(a + b));
        assert!(learn(&few).is_empty());
    }

    const ONES: [&str; 20] = ["zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten", "eleven", "twelve", "thirteen", "fourteen", "fifteen", "sixteen", "seventeen", "eighteen", "nineteen"];
    const TENS: [&str; 10] = ["", "", "twenty", "thirty", "forty", "fifty", "sixty", "seventy", "eighty", "ninety"];

    /// English words for 0 to 99, as lessons spell them.
    fn english(n: usize) -> String {
        if n < 20 {
            ONES[n].to_string()
        } else if n % 10 == 0 {
            TENS[n / 10].to_string()
        } else {
            format!("{} {}", TENS[n / 10], ONES[n % 10])
        }
    }

    /// Digit words of an invented positional counting in base 6, most significant first.
    fn invented(n: usize) -> String {
        let digit = ["dag", "sim", "bat", "gar", "sed", "dib"];
        let mut digits = Vec::new();
        let mut rest = n;
        loop {
            digits.push(digit[rest % 6]);
            rest /= 6;
            if rest == 0 {
                break;
            }
        }
        digits.reverse();
        digits.join(" ")
    }

    /// A kind that reads a spelled number: three wordings of each fact, the answer written in digits.
    fn reading(name: &str, spell: impl Fn(usize) -> String, upto: usize) -> Vec<(String, String)> {
        let mut out = Vec::new();
        for n in 0..=upto {
            for template in ["what is {s} {k}", "{k} of {s}", "please {k} {s}"] {
                out.push((template.replace("{s}", &spell(n)).replace("{k}", name), n.to_string()));
            }
        }
        out
    }

    #[test]
    fn words_that_add_up_are_found_from_a_kind_that_reads_them() {
        let lexicon = learn_reading(&reading("tehayarn", english, 99));
        assert_eq!(lexicon.grammar(0), Grammar::Additive);
        assert_eq!(lexicon.get("twenty"), Some(20));
        assert_eq!(lexicon.get("seventeen"), Some(17));
        assert_eq!(lexicon.get("zero"), Some(0));
        assert_eq!(lexicon.value_of_text("sixty three"), Some((63, 0)));
        // Nothing is a number that no lesson says is one, and the name of the kind is not.
        assert_eq!(lexicon.get("tehayarn"), None);
        assert_eq!(lexicon.get("what"), None);
        // Saying numbers back, within what the lessons showed.
        assert_eq!(lexicon.speak(0, 55).as_deref(), Some("fifty five"));
        assert_eq!(lexicon.speak(0, 0).as_deref(), Some("zero"));
        assert_eq!(lexicon.speak(0, 19).as_deref(), Some("nineteen"));
        assert_eq!(lexicon.speak(0, 100), None);
    }

    #[test]
    fn digit_words_with_place_value_are_found_in_their_base() {
        let lexicon = learn_reading(&reading("henuyarn", invented, 99));
        assert_eq!(lexicon.grammar(0), Grammar::Positional(6));
        assert_eq!(lexicon.value_of_text("sim dag"), Some((6, 0)));
        assert_eq!(lexicon.value_of_text("bat bat gar"), Some((2 * 36 + 2 * 6 + 3, 0)));
        assert_eq!(lexicon.speak(0, 77).as_deref(), Some(invented(77).as_str()));
    }

    #[test]
    fn a_kind_that_is_not_a_reading_gives_no_number_words() {
        // Answers in digits, but the questions are not numbers: things and their sizes.
        let mut lessons = Vec::new();
        for (i, w) in ["bafoba", "tadaba", "podoba", "mifuba", "ligaba", "nufeba", "pefoba", "dageba", "kuvaba", "retiba", "gobaba", "zinuba"].iter().enumerate() {
            for template in ["what is {w} sedavubi", "sedavubi of {w}", "please sedavubi {w}"] {
                lessons.push((template.replace("{w}", w), ((i * 7) % 5).to_string()));
            }
        }
        assert!(learn_reading(&lessons).is_empty());
        // And a kind whose questions carry digits already is read by the digits.
        let mut digits = Vec::new();
        for a in 0..30 {
            digits.push((format!("what is {a} fegisido"), (a + 1).to_string()));
            digits.push((format!("fegisido of {a}"), (a + 1).to_string()));
        }
        assert!(learn_reading(&digits).is_empty());
    }

    /// Words that multiply: a word per digit, a word for the base (10) and one for its square, a missing digit word is one; with `minus_from` > 0, a last
    /// digit of that size or more is the next ten minus what is missing.
    fn multiplier(n: usize, minus_from: usize) -> String {
        let digit = ["nolu", "ana", "beru", "cile", "dovi", "eka", "fumi", "gato", "hely", "ibo"];
        let say = |d: usize, unit: &str| -> Vec<String> {
            let mut out = Vec::new();
            if d > 1 {
                out.push(digit[d].to_string());
            }
            out.push(unit.to_string());
            out
        };
        fn up(n: usize, digit: &[&str; 10], say: &dyn Fn(usize, &str) -> Vec<String>, minus_from: usize) -> Vec<String> {
            if n == 0 {
                return vec![digit[0].to_string()];
            }
            if minus_from > 0 && n % 10 >= minus_from {
                let missing = 10 - n % 10;
                let mut out = up(n + missing, digit, say, minus_from);
                out.push("nopa".to_string());
                out.push(digit[missing].to_string());
                return out;
            }
            let (h, t, o) = (n / 100, n % 100 / 10, n % 10);
            let mut out = Vec::new();
            if h > 0 {
                out.extend(say(h, "kora"));
            }
            if t > 0 {
                out.extend(say(t, "zaru"));
            }
            if o > 0 {
                out.push(digit[o].to_string());
            }
            out
        }
        up(n, &digit, &say, minus_from).join(" ")
    }

    /// Base 7, the least significant digit first.
    fn least_first(n: usize) -> String {
        let digit = ["moa", "pel", "rik", "sav", "tuo", "vex", "wyn"];
        let (mut rest, mut out) = (n, Vec::new());
        loop {
            out.push(digit[rest % 7]);
            rest /= 7;
            if rest == 0 {
                break;
            }
        }
        out.join(" ")
    }

    #[test]
    fn words_that_multiply_a_base_word_are_found_with_and_without_a_word_for_minus() {
        for minus_from in [0usize, 8] {
            let lexicon = learn_reading(&reading("hexayarn", |n| multiplier(n, minus_from), 199));
            assert_eq!(lexicon.families(), 1, "{minus_from}");
            assert_eq!(lexicon.grammar(0), Grammar::Multiplier { base: 10, minus_from: minus_from as i64 }, "{minus_from}");
            assert_eq!(lexicon.value_of_text(&multiplier(137, minus_from)), Some((137, 0)));
            assert_eq!(lexicon.speak(0, 168).as_deref(), Some(multiplier(168, minus_from).as_str()));
            assert_eq!(lexicon.speak(0, 98).as_deref(), Some(multiplier(98, minus_from).as_str()));
        }
    }

    #[test]
    fn digits_with_the_least_significant_first_are_found_in_their_base() {
        let lexicon = learn_reading(&reading("tamayarn", least_first, 300));
        assert_eq!(lexicon.grammar(0), Grammar::LeastFirst(7));
        assert_eq!(lexicon.value_of_text("pel pel"), Some((8, 0)));
        assert_eq!(lexicon.speak(0, 250).as_deref(), Some(least_first(250).as_str()));
    }

    #[test]
    fn a_world_with_two_families_of_number_words_reads_both() {
        let mut lessons = reading("tehayarn", english, 99);
        lessons.extend(reading("hexayarn", |n| multiplier(n, 0), 199));
        let lexicon = learn_reading(&lessons);
        assert_eq!(lexicon.families(), 2);
        assert_eq!(lexicon.value_of_text("sixty three").map(|x| x.0), Some(63));
        assert_eq!(lexicon.value_of_text(&multiplier(137, 0)).map(|x| x.0), Some(137));
        let (a, b) = (lexicon.value_of_text("sixty three").unwrap().1, lexicon.value_of_text(&multiplier(137, 0)).unwrap().1);
        assert_ne!(a, b);
        // A run that mixes the two families is no number.
        assert_eq!(lexicon.value_of_text("sixty kora"), None);
    }

    #[test]
    fn a_run_that_the_grammar_does_not_allow_is_no_number() {
        // Multiplier words: the parts come from the largest down, each once.
        let lexicon = learn_reading(&reading("hexayarn", |n| multiplier(n, 0), 199));
        assert_eq!(lexicon.value_of_text(&multiplier(137, 0)).map(|x| x.0), Some(137));
        assert_eq!(lexicon.value_of_text("beru zaru cile kora"), None, "tens before hundreds");
        assert_eq!(lexicon.value_of_text("ana kora ana kora"), None, "hundreds twice");
        // Positional digits: a number does not begin with a zero digit (here the last word is the most significant).
        let lexicon = learn_reading(&reading("tamayarn", least_first, 300));
        assert_eq!(lexicon.value_of_text("moa pel"), Some((7, 0)));
        assert_eq!(lexicon.value_of_text("pel moa"), None, "ends in the zero digit");
        let msd = learn_reading(&reading("henuyarn", invented, 99));
        assert_eq!(msd.value_of_text("dag sim"), None, "begins with the zero digit");
    }

    #[test]
    fn a_run_longer_than_any_numeral_the_lessons_showed_is_no_number() {
        let lexicon = learn_reading(&reading("tamayarn", least_first, 300));
        // 300 is three digits in base 7; four words were never shown.
        assert_eq!(lexicon.value_of_text("pel pel pel"), Some((1 + 7 + 49, 0)));
        assert_eq!(lexicon.value_of_text("pel pel pel pel"), None);
    }

    /// Bijective base 5, the most significant digit first: digits one to five (no zero digit), and a word of its own for zero.
    fn bijective5(n: usize) -> String {
        let digit = ["vada", "pisi", "nugi", "metu", "mobo", "hoko"];
        if n == 0 {
            return digit[0].to_string();
        }
        let (mut rest, mut out) = (n, Vec::new());
        while rest > 0 {
            let d = (rest - 1) % 5 + 1;
            out.push(digit[d]);
            rest = (rest - d) / 5;
        }
        out.reverse();
        out.join(" ")
    }

    /// Balanced base 5, the most significant digit first: digits from minus two to two.
    fn balanced5(n: usize) -> String {
        let digit = ["lazu", "guse", "voko", "nona", "tera"];
        if n == 0 {
            return digit[2].to_string();
        }
        let (mut rest, mut out) = (n as i64, Vec::new());
        while rest != 0 {
            let r = rest.rem_euclid(5);
            let d = if r > 2 { r - 5 } else { r };
            out.push(digit[(d + 2) as usize]);
            rest = (rest - d) / 5;
        }
        out.reverse();
        out.join(" ")
    }

    #[test]
    fn digits_with_no_zero_or_with_negative_ones_are_found_as_steps_of_a_number_so_far() {
        for (name, spell, fold) in [("bijective", bijective5 as fn(usize) -> String, "bijective"), ("balanced", balanced5, "balanced")] {
            let lexicon = learn_reading(&reading("unspellyarn", spell, 520));
            assert_eq!(lexicon.families(), 1, "{fold}");
            assert_eq!(lexicon.grammar(0), Grammar::Fold { base: 5, least_first: false }, "{name}");
            for k in [0usize, 1, 5, 6, 17, 124, 427, 519] {
                assert_eq!(lexicon.value_of_text(&spell(k)), Some((k as i64, 0)), "{name} {k} = {}", spell(k));
            }
            // Said back, and not past the range the lessons showed.
            for k in [0usize, 3, 26, 301, 519] {
                assert_eq!(lexicon.speak(0, k as i64).as_deref(), Some(spell(k).as_str()), "{name} {k}");
            }
            assert_eq!(lexicon.speak(0, 521), None);
        }
    }

    #[test]
    fn a_numeral_that_begins_with_a_zero_digit_or_comes_out_below_zero_is_no_number() {
        let lexicon = learn_reading(&reading("unspellyarn", balanced5, 520));
        // "voko nona" is a zero digit followed by a one; "lazu" alone would be minus two, which is not a number the lessons can mean.
        assert_eq!(lexicon.value_of_text("voko nona"), None);
        assert_eq!(lexicon.value_of_text("lazu"), None);
    }
}
