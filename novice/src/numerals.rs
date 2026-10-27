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

/// How the words of a lexicon make a number.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Grammar {
    /// Every word is a number of its own ("five").
    #[default]
    Single,
    /// A run of words is the sum of its words ("twenty three").
    Additive,
    /// A run of words is a number written in this base, the first word the most significant digit.
    Positional(i64),
}

/// The words that are numbers, what they mean, and how a run of them makes one number (and how a number is said again).
#[derive(Clone, Debug, Default)]
pub struct Lexicon {
    values: BTreeMap<String, i64>,
    grammar: Grammar,
    /// For single words: the word for each number. For runs: unused.
    spoken: BTreeMap<i64, String>,
    /// For runs: the largest number the lessons showed, and whether saying numbers back reproduced every one of them.
    max_spoken: i64,
    speaks_runs: bool,
}

impl Lexicon {
    /// Words that are one number each.
    pub fn single(values: BTreeMap<String, i64>) -> Lexicon {
        let spoken = values.iter().map(|(w, v)| (*v, w.clone())).collect();
        Lexicon { values, grammar: Grammar::Single, spoken, max_spoken: 0, speaks_runs: false }
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn grammar(&self) -> Grammar {
        self.grammar
    }

    pub fn get(&self, word: &str) -> Option<i64> {
        self.values.get(word).copied()
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// The number a run of words makes under the grammar, if every word is in the lexicon.
    pub fn value_of(&self, run: &[String]) -> Option<i64> {
        if run.is_empty() {
            return None;
        }
        match self.grammar {
            Grammar::Single => (run.len() == 1).then(|| self.get(&run[0])).flatten(),
            Grammar::Additive => run.iter().map(|w| self.get(w)).sum::<Option<i64>>(),
            Grammar::Positional(b) => run.iter().try_fold(0i64, |acc, w| {
                let d = self.get(w)?;
                (d < b).then(|| acc.checked_mul(b)?.checked_add(d)).flatten()
            }),
        }
    }

    /// The number a whole text is made of, if all of it is number words.
    pub fn value_of_text(&self, text: &str) -> Option<i64> {
        let words = tokens(text);
        self.value_of(&words)
    }

    /// The words for a number: the word the lessons used for it, or for a run grammar the words that make it, within the range the lessons showed
    /// and only if saying back every number of the lessons gave exactly what the lessons said.
    pub fn speak(&self, n: i64) -> Option<String> {
        match self.grammar {
            Grammar::Single => self.spoken.get(&n).cloned(),
            _ if !self.speaks_runs => None,
            _ => self.compose(n),
        }
    }

    fn compose(&self, n: i64) -> Option<String> {
        if n < 0 || n > self.max_spoken {
            return None;
        }
        let word_for = |v: i64| self.values.iter().find(|(_, x)| **x == v).map(|(w, _)| w.clone());
        match self.grammar {
            Grammar::Single => None,
            Grammar::Additive => {
                if n == 0 {
                    return word_for(0);
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
            Grammar::Positional(b) => {
                let mut digits = Vec::new();
                let mut rest = n;
                loop {
                    digits.push(rest % b);
                    rest /= b;
                    if rest == 0 {
                        break;
                    }
                }
                digits.iter().rev().map(|&d| word_for(d)).collect::<Option<Vec<_>>>().map(|w| w.join(" "))
            }
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
const MAX_BASE: i64 = 16;

/// Give the words of `equations` (a run of words, the number it makes) the numbers that make every equation hold under `grammar`, by fixing a word wherever an equation
/// has only one word left that is not fixed. None if an equation fails or a word would need a number that is not whole or below zero.
fn fix_runs(equations: &[(Vec<String>, i64)], grammar: Grammar) -> Option<BTreeMap<String, i64>> {
    let weights = |len: usize| -> Vec<i64> {
        match grammar {
            Grammar::Positional(b) => (0..len).map(|i| b.pow((len - 1 - i) as u32)).collect(),
            _ => vec![1; len],
        }
    };
    let mut known: BTreeMap<String, i64> = BTreeMap::new();
    loop {
        let mut changed = false;
        for (run, n) in equations {
            let w = weights(run.len());
            let mut sum = 0i64;
            let mut open: Vec<usize> = Vec::new();
            for (i, t) in run.iter().enumerate() {
                match known.get(t) {
                    Some(v) => sum += w[i] * v,
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
            let weight: i64 = open.iter().map(|&i| w[i]).sum();
            let rest = n - sum;
            if rest < 0 || rest % weight != 0 {
                return None;
            }
            let v = rest / weight;
            if let Grammar::Positional(b) = grammar {
                if v >= b {
                    return None;
                }
            }
            known.insert(run[open[0]].clone(), v);
            changed = true;
        }
        if !changed {
            return Some(known);
        }
    }
}

/// Number words found from a kind that reads them: its questions are made of number words and its answers are written numbers ("twenty three" is
/// 23). Which words of a question are the number is not known; they are what is left when the words that every lesson of the kind has (its name
/// and its frame) are taken away from what all the wordings of one fact have in common. The meaning of each word is then fixed the way the counting
/// is: a number of one word fixes it, a longer one fixes its last unknown word. Two ways of making a number from a run of words are tried: the sum
/// of its words, and its words as the digits of a number in some base; the first that holds in every lesson is kept. It is then required that saying
/// the numbers back reproduces what the lessons said, or the lexicon only reads.
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
    let mut df: BTreeMap<&str, usize> = BTreeMap::new();
    for (words, _) in &reading {
        for w in words.iter().collect::<BTreeSet<_>>() {
            *df.entry(w.as_str()).or_insert(0) += 1;
        }
    }
    let names: Vec<&str> = df.iter().filter(|(_, n)| **n >= MIN_READING).map(|(w, _)| *w).collect();
    let mut best: Option<(Lexicon, usize)> = None;
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
        let grammars = std::iter::once(Grammar::Additive).chain((2..=MAX_BASE).map(Grammar::Positional));
        for grammar in grammars {
            let Some(values) = fix_runs(&equations, grammar) else { continue };
            let determined = equations.iter().filter(|(run, _)| run.iter().all(|w| values.contains_key(w))).count();
            if values.len() < MIN_WORDS || determined * 10 < equations.len() * 9 {
                continue;
            }
            let max_spoken = equations.iter().map(|(_, v)| *v).max().unwrap_or(0);
            let mut lexicon = Lexicon { values, grammar, spoken: BTreeMap::new(), max_spoken, speaks_runs: true };
            lexicon.speaks_runs = equations.iter().filter(|(run, _)| run.iter().all(|w| lexicon.values.contains_key(w))).all(|(run, v)| lexicon.compose(*v).as_deref() == Some(run.join(" ").as_str()));
            if best.as_ref().is_none_or(|(_, n)| lexicon.values.len() > *n) {
                let n = lexicon.values.len();
                best = Some((lexicon, n));
            }
            break;
        }
    }
    best.map(|(l, _)| l).unwrap_or_default()
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
        assert_eq!(lexicon.grammar(), Grammar::Additive);
        assert_eq!(lexicon.get("twenty"), Some(20));
        assert_eq!(lexicon.get("seventeen"), Some(17));
        assert_eq!(lexicon.get("zero"), Some(0));
        assert_eq!(lexicon.value_of_text("sixty three"), Some(63));
        // Nothing is a number that no lesson says is one, and the name of the kind is not.
        assert_eq!(lexicon.get("tehayarn"), None);
        assert_eq!(lexicon.get("what"), None);
        // Saying numbers back, within what the lessons showed.
        assert_eq!(lexicon.speak(55).as_deref(), Some("fifty five"));
        assert_eq!(lexicon.speak(0).as_deref(), Some("zero"));
        assert_eq!(lexicon.speak(19).as_deref(), Some("nineteen"));
        assert_eq!(lexicon.speak(100), None);
    }

    #[test]
    fn digit_words_with_place_value_are_found_in_their_base() {
        let lexicon = learn_reading(&reading("henuyarn", invented, 99));
        assert_eq!(lexicon.grammar(), Grammar::Positional(6));
        assert_eq!(lexicon.value_of_text("sim dag"), Some(6));
        assert_eq!(lexicon.value_of_text("bat bat gar"), Some(2 * 36 + 2 * 6 + 3));
        assert_eq!(lexicon.speak(77).as_deref(), Some(invented(77).as_str()));
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
}
