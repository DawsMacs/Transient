// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DawsMacs
//! The bridge to real text. Everything before this file was taught tidy synthetic lessons (made-up words, small numbers, a handful of
//! templates). `baby bridge` takes question/answer pairs somebody else wrote (a TSV, or the SVAMP word-problem file) and asks what survives:
//! how much of real text the baby can even read, what it answers, what it refuses and why, and whether it ever answers WRONG.
//!
//! Nothing here is tuned to a corpus. The loader only reshapes a file into lessons; the filters say exactly what the baby cannot read.

use crate::kinds::Item;
use crate::rng::Rng;
use crate::school::{unreadable_number, School};

/// One problem as it came in: the text to ask, the answer in the form the baby gives answers (digits), and the kind the file gave it.
#[derive(Clone, Debug)]
pub struct Problem {
    pub id: String,
    pub query: String,
    pub answer: String,
    pub kind: String,
}

// ---- a minimal JSON reader (zero dependencies): enough for SVAMP.json ----

#[derive(Debug, Clone, PartialEq)]
enum J {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<J>),
    Obj(Vec<(String, J)>),
}

struct P<'a> {
    s: &'a [u8],
    i: usize,
}

impl<'a> P<'a> {
    fn ws(&mut self) {
        while self.i < self.s.len() && self.s[self.i].is_ascii_whitespace() {
            self.i += 1;
        }
    }
    fn eat(&mut self, c: u8) -> Result<(), String> {
        self.ws();
        if self.s.get(self.i) == Some(&c) {
            self.i += 1;
            Ok(())
        } else {
            Err(format!("expected '{}' at byte {}", c as char, self.i))
        }
    }
    fn value(&mut self) -> Result<J, String> {
        self.ws();
        match self.s.get(self.i).copied() {
            Some(b'{') => {
                self.i += 1;
                let mut v = Vec::new();
                self.ws();
                if self.s.get(self.i) == Some(&b'}') {
                    self.i += 1;
                    return Ok(J::Obj(v));
                }
                loop {
                    self.ws();
                    let k = match self.value()? {
                        J::Str(k) => k,
                        _ => return Err(format!("object key is not a string at byte {}", self.i)),
                    };
                    self.eat(b':')?;
                    v.push((k, self.value()?));
                    self.ws();
                    match self.s.get(self.i) {
                        Some(b',') => self.i += 1,
                        Some(b'}') => {
                            self.i += 1;
                            return Ok(J::Obj(v));
                        }
                        _ => return Err(format!("bad object at byte {}", self.i)),
                    }
                }
            }
            Some(b'[') => {
                self.i += 1;
                let mut v = Vec::new();
                self.ws();
                if self.s.get(self.i) == Some(&b']') {
                    self.i += 1;
                    return Ok(J::Arr(v));
                }
                loop {
                    v.push(self.value()?);
                    self.ws();
                    match self.s.get(self.i) {
                        Some(b',') => self.i += 1,
                        Some(b']') => {
                            self.i += 1;
                            return Ok(J::Arr(v));
                        }
                        _ => return Err(format!("bad array at byte {}", self.i)),
                    }
                }
            }
            Some(b'"') => {
                self.i += 1;
                let mut out: Vec<u8> = Vec::new();
                while let Some(&c) = self.s.get(self.i) {
                    self.i += 1;
                    match c {
                        b'"' => return String::from_utf8(out).map(J::Str).map_err(|e| e.to_string()),
                        b'\\' => {
                            let e = *self.s.get(self.i).ok_or("dangling escape")?;
                            self.i += 1;
                            match e {
                                b'n' => out.push(b'\n'),
                                b't' => out.push(b'\t'),
                                b'r' => out.push(b'\r'),
                                b'b' | b'f' => out.push(b' '),
                                b'u' => {
                                    let hex = std::str::from_utf8(self.s.get(self.i..self.i + 4).ok_or("short \\u escape")?).map_err(|e| e.to_string())?;
                                    let code = u32::from_str_radix(hex, 16).map_err(|e| e.to_string())?;
                                    self.i += 4;
                                    let ch = char::from_u32(code).unwrap_or(' ');
                                    let mut buf = [0u8; 4];
                                    out.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
                                }
                                other => out.push(other),
                            }
                        }
                        other => out.push(other),
                    }
                }
                Err("unterminated string".into())
            }
            Some(b't') if self.s[self.i..].starts_with(b"true") => {
                self.i += 4;
                Ok(J::Bool(true))
            }
            Some(b'f') if self.s[self.i..].starts_with(b"false") => {
                self.i += 5;
                Ok(J::Bool(false))
            }
            Some(b'n') if self.s[self.i..].starts_with(b"null") => {
                self.i += 4;
                Ok(J::Null)
            }
            Some(c) if c == b'-' || c.is_ascii_digit() => {
                let start = self.i;
                while self.i < self.s.len() && matches!(self.s[self.i], b'-' | b'+' | b'.' | b'e' | b'E' | b'0'..=b'9') {
                    self.i += 1;
                }
                std::str::from_utf8(&self.s[start..self.i]).unwrap().parse::<f64>().map(J::Num).map_err(|e| e.to_string())
            }
            other => Err(format!("unexpected {:?} at byte {}", other.map(|c| c as char), self.i)),
        }
    }
}

fn parse_json(text: &str) -> Result<J, String> {
    let mut p = P { s: text.as_bytes(), i: 0 };
    let v = p.value()?;
    p.ws();
    if p.i != p.s.len() {
        return Err(format!("trailing characters at byte {}", p.i));
    }
    Ok(v)
}

fn field<'a>(o: &'a [(String, J)], key: &str) -> Option<&'a J> {
    o.iter().find(|(k, _)| k == key).map(|(_, v)| v)
}

fn text_of(j: Option<&J>) -> String {
    match j {
        Some(J::Str(s)) => s.clone(),
        Some(J::Num(n)) => format_number(*n),
        _ => String::new(),
    }
}

/// 8.0 -> "8", 2.5 -> "2.5".
fn format_number(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        format!("{n}")
    }
}

/// SVAMP.json: a list of {"ID", "Body", "Question", "Equation", "Type", "Answer", ...}. The text asked is the story followed by the question.
pub fn load_svamp(text: &str) -> Result<Vec<Problem>, String> {
    let J::Arr(items) = parse_json(text)? else { return Err("expected a JSON list of problems".into()) };
    let mut out = Vec::new();
    for (n, item) in items.iter().enumerate() {
        let J::Obj(o) = item else { return Err(format!("item {n} is not an object")) };
        let body = text_of(field(o, "Body"));
        let question = text_of(field(o, "Question"));
        let id = match text_of(field(o, "ID")) {
            id if id.is_empty() => format!("#{n}"),
            id => id,
        };
        out.push(Problem {
            id,
            query: format!("{body} {question}").split_whitespace().collect::<Vec<_>>().join(" "),
            answer: text_of(field(o, "Answer")),
            // (one problem in SVAMP.json is typed "Common-Divison": a typo in the file, folded into its kind)
            kind: text_of(field(o, "Type")).to_lowercase().replace(' ', "-").replace("divison", "division"),
        });
    }
    Ok(out)
}

/// question TAB answer [TAB kind], one per line; lines starting with # are comments.
pub fn load_tsv(text: &str) -> Result<Vec<Problem>, String> {
    let mut out = Vec::new();
    for (n, line) in text.lines().enumerate() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 2 {
            return Err(format!("line {}: expected question TAB answer", n + 1));
        }
        out.push(Problem {
            id: format!("#{n}"),
            query: parts[0].trim().to_string(),
            answer: parts[1].trim().to_string(),
            kind: parts.get(2).map_or("all", |k| k.trim()).to_lowercase().replace(' ', "-"),
        });
    }
    Ok(out)
}

// ---- what the baby can read at all ----

/// Why a problem cannot be given to the baby as it stands (the baby reads whole numbers of up to four digits, and says whole-number answers).
pub fn unreadable_reason(p: &Problem) -> Option<&'static str> {
    if unreadable_number(&p.query).is_some() {
        return Some("a number in the text it cannot read (decimal, sign, fraction, percent, thousands comma)");
    }
    let digits: Vec<&str> = p.query.split(|c: char| !c.is_ascii_digit()).filter(|d| !d.is_empty()).collect();
    if digits.iter().any(|d| d.len() > 4) {
        return Some("a number in the text of more than four digits");
    }
    if p.answer.parse::<u64>().is_err() {
        return Some("an answer that is not a whole non-negative number");
    }
    if p.answer.len() > 4 {
        return Some("an answer of more than four digits");
    }
    None
}

/// How many numbers a problem's text holds (runs of digits).
pub fn numbers_in_text(q: &str) -> usize {
    q.split(|c: char| !c.is_ascii_digit()).filter(|d| !d.is_empty()).count()
}

/// The problems the baby can be given, and how many were set aside for each reason.
pub fn readable(problems: Vec<Problem>) -> (Vec<Problem>, Vec<(&'static str, usize)>) {
    let mut kept = Vec::new();
    let mut dropped: Vec<(&'static str, usize)> = Vec::new();
    for p in problems {
        match unreadable_reason(&p) {
            None => kept.push(p),
            Some(why) => match dropped.iter_mut().find(|(w, _)| *w == why) {
                Some(d) => d.1 += 1,
                None => dropped.push((why, 1)),
            },
        }
    }
    (kept, dropped)
}

/// How a problem is split between teaching and testing. Variations of one story sit next to each other in SVAMP, so a random split lets the
/// baby be tested on a near-copy of what it was taught; `Blocks` holds out whole runs of neighbouring problems instead.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Split {
    Random,
    Blocks,
}

pub fn split(problems: &[Problem], how: Split, test_share: f32, seed: u64) -> (Vec<Problem>, Vec<Problem>) {
    let mut train = Vec::new();
    let mut test = Vec::new();
    match how {
        Split::Random => {
            let mut rng = Rng::new(seed ^ 0xB41D6E);
            for p in problems {
                if rng.uniform() < test_share { test.push(p.clone()) } else { train.push(p.clone()) }
            }
        }
        Split::Blocks => {
            // runs of 12 neighbouring problems, every k-th run held out
            let every = ((1.0 / test_share).round() as usize).max(2);
            for (i, p) in problems.iter().enumerate() {
                if (i / 12) % every == (seed as usize) % every { test.push(p.clone()) } else { train.push(p.clone()) }
            }
        }
    }
    (train, test)
}

// ---- teaching and grading ----

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Units {
    /// the file's own kinds are the units (a teacher groups the lessons and never says how to learn them)
    ByKind,
    /// the baby finds the units itself from the unlabelled lessons
    Discover,
}

#[derive(Default, Clone, Debug)]
pub struct Tally {
    pub n: usize,
    pub right: usize,
    pub wrong: usize,
    pub refused: usize,
    pub guess_right: usize,
    pub guess_wrong: usize,
    /// the sum of the hit rates it stated for the guesses it made (to compare with how often they were right)
    pub stated: f64,
}

/// What would have happened had the baby been told each test problem's kind (the file's) instead of recognising it from the words.
pub struct Oracle {
    /// of the test questions, how many it routed to the unit of the file's own kind
    pub routed_right: usize,
    pub n: usize,
    pub right: usize,
    pub wrong: usize,
    pub guess_right: usize,
    pub guess_wrong: usize,
    pub refused: usize,
}

pub struct Report {
    pub oracle: Oracle,
    pub kept: usize,
    pub dropped: Vec<(&'static str, usize)>,
    pub train: usize,
    pub test: usize,
    pub units: Vec<(String, String, [f64; 5])>,
    pub by_kind: Vec<(String, Tally)>,
    pub all: Tally,
    /// why it refused (the start of its own reason), most common first
    pub refusals: Vec<(String, usize)>,
    /// (question, truth, what it said): every WRONG answer, so none can hide in a percentage
    pub wrongs: Vec<(String, String, String)>,
}

fn reason_class(how: &str) -> String {
    let h = how.trim();
    if let Some(rest) = h.strip_prefix("I haven't met the word") {
        let _ = rest;
        return "a word it has never met".into();
    }
    for (needle, label) in [
        ("no kind of lesson I was taught is like this question", "no kind of lesson it knows is like it"),
        ("I only read plain whole numbers", "a number it cannot read"),
        ("won't state it as fact", "a number beyond what it was taught (a guess, if one was offered)"),
        ("they disagree", "its rule and what it was taught disagree"),
        ("I only memorise this kind of thing", "a memorise-only unit with no close lesson"),
        ("I couldn't work it out", "it has a word or shape it cannot handle"),
        ("not sure enough", "a network not sure enough"),
    ] {
        if h.contains(needle) {
            return label.into();
        }
    }
    "other".into()
}

/// Teach on `train`, grade on `test`. The kind of each test problem is the file's, used only to organise the report.
pub fn run(train: &[Problem], test: &[Problem], units: Units, seed: u64) -> Report {
    let (items, names): (Vec<Item>, Vec<String>) = match units {
        Units::ByKind => {
            let mut names: Vec<String> = Vec::new();
            for p in train {
                if !names.contains(&p.kind) {
                    names.push(p.kind.clone());
                }
            }
            let items = train
                .iter()
                .enumerate()
                .map(|(i, p)| Item { unit: names.iter().position(|n| *n == p.kind).unwrap(), fact: i, template: 0, query: p.query.clone(), answer: p.answer.clone() })
                .collect();
            (items, names)
        }
        Units::Discover => {
            let lessons: Vec<(String, String)> = train.iter().map(|p| (p.query.clone(), p.answer.clone())).collect();
            let d = crate::discover::discover(&lessons, seed);
            (d.items, d.names)
        }
    };
    let school = School::from_items(&items, names, None, seed, true);
    let mut oracle = Oracle { routed_right: 0, n: 0, right: 0, wrong: 0, guess_right: 0, guess_wrong: 0, refused: 0 };
    if units == Units::ByKind {
        for p in test {
            let Some(r) = school.ask_as(&p.query, &p.kind) else { continue };
            oracle.n += 1;
            let (routed, _) = school.route(&p.query, &mut crate::ledger::Ledger::default());
            if school.units[routed].name == p.kind {
                oracle.routed_right += 1;
            }
            match (&r.answer, &r.guess) {
                (Some(a), _) if *a == p.answer => oracle.right += 1,
                (Some(_), _) => oracle.wrong += 1,
                (None, Some(g)) if g.answer == p.answer => oracle.guess_right += 1,
                (None, Some(_)) => oracle.guess_wrong += 1,
                (None, None) => oracle.refused += 1,
            }
        }
    }
    let mut report = Report {
        oracle,
        kept: train.len() + test.len(),
        dropped: Vec::new(),
        train: train.len(),
        test: test.len(),
        units: school.units.iter().map(|u| (u.name.clone(), u.chosen.name().to_string(), u.evidence)).collect(),
        by_kind: Vec::new(),
        all: Tally::default(),
        refusals: Vec::new(),
        wrongs: Vec::new(),
    };
    let mut refusals: Vec<(String, usize)> = Vec::new();
    for p in test {
        let r = school.ask(&p.query);
        let mut t = Tally { n: 1, ..Tally::default() };
        match (&r.answer, &r.guess) {
            (Some(a), _) if *a == p.answer => t.right = 1,
            (Some(a), _) => {
                t.wrong = 1;
                report.wrongs.push((p.query.clone(), p.answer.clone(), a.clone()));
            }
            (None, g) => {
                t.refused = 1;
                match g {
                    Some(g) if g.answer == p.answer => {
                        t.guess_right = 1;
                        t.stated = g.rate as f64;
                    }
                    Some(g) => {
                        t.guess_wrong = 1;
                        t.stated = g.rate as f64;
                    }
                    None => {}
                }
                let class = reason_class(&r.how);
                match refusals.iter_mut().find(|(c, _)| *c == class) {
                    Some(e) => e.1 += 1,
                    None => refusals.push((class, 1)),
                }
            }
        }
        for tally in [&mut report.all, kind_entry(&mut report.by_kind, &p.kind)] {
            tally.n += t.n;
            tally.right += t.right;
            tally.wrong += t.wrong;
            tally.refused += t.refused;
            tally.guess_right += t.guess_right;
            tally.guess_wrong += t.guess_wrong;
            tally.stated += t.stated;
        }
    }
    refusals.sort_by(|a, b| b.1.cmp(&a.1));
    report.refusals = refusals;
    report
}

fn kind_entry<'a>(v: &'a mut Vec<(String, Tally)>, kind: &str) -> &'a mut Tally {
    if let Some(i) = v.iter().position(|(k, _)| k == kind) {
        return &mut v[i].1;
    }
    v.push((kind.to_string(), Tally::default()));
    &mut v.last_mut().unwrap().1
}

/// What the baby makes of problems that hold more numbers than its rules use, when it also learns which numbers matter.
pub struct SelectReport {
    pub taught_two: usize,
    pub taught_many: usize,
    /// of the many-number lessons, how many a unit's own program explains with exactly one pair (the labels the selector learned from)
    pub labelled: usize,
    pub two: Tally,
    pub many: Tally,
    /// of the many-number test problems whose program-consistent pair is unique: how often the selector kept exactly that pair (and how many there were)
    pub kept_right: (usize, usize),
    pub oracle_routing: (usize, usize),
    /// what its stated hit rates were multiplied by after checking itself on held-back answers: [read as written, cut down to the numbers that matter]
    pub scale: [f32; 2],
}

fn tally_reply(t: &mut Tally, r: &crate::school::Reply, truth: &str) {
    t.n += 1;
    match (&r.answer, &r.guess) {
        (Some(a), _) if a == truth => t.right += 1,
        (Some(_), _) => t.wrong += 1,
        (None, g) => {
            t.refused += 1;
            if let Some(g) = g {
                t.stated += g.rate as f64;
                if g.answer == truth { t.guess_right += 1 } else { t.guess_wrong += 1 }
            }
        }
    }
}

/// Teach in two steps and grade on all test problems: (1) on the lessons with at most two numbers only, as before; (2) from the programs that gives, find in
/// each many-number lesson the one pair of numbers a unit's program turns into the answer, learn from those which numbers matter (`selector`), blank
/// the others in the lessons that have such a pair, and teach again on everything. The kinds are the file's; the labels come from the baby's own programs,
/// never from the corpus's equations.
pub fn run_select(train: &[Problem], test: &[Problem], seed: u64) -> SelectReport {
    use crate::school::School;
    let line = crate::quantity::NumberLine::from_counting(&crate::quantity::counting_lessons());
    // every fourth lesson is held back from teaching: the baby checks its stated hit rates against those answers, then is taught everything
    let held: Vec<&Problem> = train.iter().enumerate().filter(|(i, _)| i % 4 == 3).map(|(_, p)| p).collect();
    let fit: Vec<Problem> = train.iter().enumerate().filter(|(i, _)| i % 4 != 3).map(|(_, p)| p.clone()).collect();
    let (mut checked, _, _) = teach_selecting(&fit, seed, &line);
    checked.calibrate_guesses(&held.iter().map(|p| (p.query.clone(), p.answer.clone())).collect::<Vec<_>>());
    let scale = checked.guess_scales();
    let (mut school, school_a, labelled) = teach_selecting(train, seed, &line);
    school.set_guess_scales(scale);
    let used = |p: &Problem| -> Option<Vec<usize>> {
        let e = school_a.unit_expression(&p.kind)?;
        let truth: f32 = p.answer.parse().ok()?;
        crate::selector::used_pair(&p.query, truth, &line, &|a, b| e.eval(a, b))
    };
    let (taught_two, taught_many) = (train.iter().filter(|p| numbers_in_text(&p.query) <= 2).count(), train.iter().filter(|p| numbers_in_text(&p.query) > 2).count());
    let mut rep = SelectReport { taught_two, taught_many, labelled, two: Tally::default(), many: Tally::default(), kept_right: (0, 0), oracle_routing: (0, 0), scale };
    for p in test {
        let r = school.ask(&p.query);
        let t = if numbers_in_text(&p.query) <= 2 { &mut rep.two } else { &mut rep.many };
        tally_reply(t, &r, &p.answer);
        if numbers_in_text(&p.query) > 2 {
            if let Some(pair) = used(p) {
                rep.kept_right.1 += 1;
                if school.focus(&p.query) == keep_only(&p.query, &pair, &line) {
                    rep.kept_right.0 += 1;
                }
            }
            let (routed, _) = school.route(&school.focus(&p.query), &mut crate::ledger::Ledger::default());
            rep.oracle_routing.1 += 1;
            if school.units[routed].name == p.kind {
                rep.oracle_routing.0 += 1;
            }
        }
    }
    rep
}

/// The two-step teaching: units from the lessons with at most two numbers; from their programs, the one pair of numbers that turns each many-number lesson
/// into its answer (where there is exactly one); a selector learned from those; and the baby taught again with the unused numbers of those lessons blanked.
/// Returns the school, the first-step school (whose programs gave the labels) and how many lessons were labelled.
fn teach_selecting(train: &[Problem], seed: u64, line: &crate::quantity::NumberLine) -> (crate::school::School, crate::school::School, usize) {
    use crate::school::School;
    let (few, many): (Vec<&Problem>, Vec<&Problem>) = train.iter().partition(|p| numbers_in_text(&p.query) <= 2);
    let mut names: Vec<String> = Vec::new();
    for p in train {
        if !names.contains(&p.kind) {
            names.push(p.kind.clone());
        }
    }
    let item = |i: usize, p: &Problem, query: String| Item { unit: names.iter().position(|n| *n == p.kind).unwrap(), fact: i, template: 0, query, answer: p.answer.clone() };
    let first: Vec<Item> = few.iter().enumerate().map(|(i, p)| item(i, p, p.query.clone())).collect();
    let school_a = School::from_items(&first, names.clone(), None, seed, true);
    let mut examples: Vec<(String, Vec<usize>)> = Vec::new();
    let mut labelled: Vec<(&Problem, Vec<usize>)> = Vec::new();
    for p in &many {
        let Some(e) = school_a.unit_expression(&p.kind) else { continue };
        let Some(truth) = p.answer.parse::<f32>().ok() else { continue };
        if let Some(pair) = crate::selector::used_pair(&p.query, truth, line, &|a, b| e.eval(a, b)) {
            examples.push((p.query.clone(), pair.clone()));
            labelled.push((p, pair));
        }
    }
    let selector = crate::selector::Selector::train(&examples, line);
    let mut second = first.clone();
    let mut fact = first.len();
    for (p, pair) in &labelled {
        second.push(item(fact, p, keep_only(&p.query, pair, line)));
        fact += 1;
    }
    let mut school = School::from_items(&second, names, None, seed, true);
    school.set_selector(selector);
    (school, school_a, labelled.len())
}

/// The text (as the baby reads it) with every number blanked except those at the given positions among its numbers.
fn keep_only(text: &str, keep: &[usize], line: &crate::quantity::NumberLine) -> String {
    let tokens = crate::data::tokenize(text);
    let mut k = 0usize;
    tokens
        .iter()
        .map(|t| {
            if line.value(t).is_some() {
                let here = keep.contains(&k);
                k += 1;
                if here { t.clone() } else { crate::selector::BLANK.to_string() }
            } else {
                t.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn print_select(r: &SelectReport) {
    let pct = |a: usize, b: usize| if b == 0 { 0.0 } else { 100.0 * a as f64 / b as f64 };
    println!("taught on {} lessons with at most two numbers, then {} lessons with more numbers ({} of them with exactly one pair of numbers that a unit's program turns into the answer: the labels the selector learned from)\n", r.taught_two, r.taught_many, r.labelled);
    println!("  {:<28} {:>5} {:>8} {:>8} {:>8} {:>13} {:>13}", "tested on", "n", "fact", "WRONG", "refused", "guess right", "guess wrong");
    for (name, t) in [("two numbers or fewer", &r.two), ("more than two numbers", &r.many)] {
        println!("  {:<28} {:>5} {:>7.1}% {:>7.1}% {:>7.1}% {:>12.1}% {:>12.1}%", name, t.n, pct(t.right, t.n), pct(t.wrong, t.n), pct(t.refused, t.n), pct(t.guess_right, t.n), pct(t.guess_wrong, t.n));
    }
    println!("\nself-check on held-back lessons scaled its stated hit rates by {:.2} (questions as written) and {:.2} (questions cut down to two numbers)", r.scale[0], r.scale[1]);
    println!("of the more-than-two-number test problems with a unique program-consistent pair ({}), the selector kept exactly that pair in {:.1}%", r.kept_right.1, pct(r.kept_right.0, r.kept_right.1));
    println!("of all more-than-two-number test problems, {:.1}% were routed to the unit of their own kind (after blanking)", pct(r.oracle_routing.0, r.oracle_routing.1));
    for (name, t) in [("two or fewer", &r.two), ("more than two", &r.many)] {
        if t.guess_right + t.guess_wrong > 0 {
            println!("  {name}: stated hit rate of its guesses averaged {:.1}%, {:.1}% were right", 100.0 * t.stated / (t.guess_right + t.guess_wrong) as f64, pct(t.guess_right, t.guess_right + t.guess_wrong));
        }
    }
}

pub fn print(report: &Report, total: usize) {
    let dropped_total: usize = report.dropped.iter().map(|d| d.1).sum();
    println!("read {total} problems; the baby can be given {} of them (set aside {dropped_total}):", report.kept);
    for (why, n) in &report.dropped {
        println!("    {n:>5}  {why}");
    }
    println!("taught {} problems, tested on {} it was not taught\n", report.train, report.test);
    println!("  {:<22} {:>9}   held-back score of each way to learn it: {:>9} {:>6} {:>7} {:>8} {:>8}", "units it formed", "learns by", "remember", "rule", "select", "program", "network");
    for (name, how, ev) in report.units.iter().take(12) {
        println!("  {name:<22} {how:>9}   {:>46.0}% {:>5.0}% {:>6.0}% {:>7.0}% {:>7.0}%", 100.0 * ev[0], 100.0 * ev[1], 100.0 * ev[2], 100.0 * ev[3], 100.0 * ev[4]);
    }
    if report.units.len() > 12 {
        println!("  ... and {} more", report.units.len() - 12);
    }
    println!();
    println!("  {:<22} {:>5} {:>8} {:>9} {:>8}   {:>12} {:>12}", "kind (from the file)", "n", "right", "refused", "WRONG", "guess right", "guess wrong");
    let row = |name: &str, t: &Tally| {
        let p = |x: usize| 100.0 * x as f64 / t.n.max(1) as f64;
        println!("  {name:<22} {:>5} {:>7.1}% {:>8.1}% {:>7.1}%   {:>11.1}% {:>11.1}%", t.n, p(t.right), p(t.refused), p(t.wrong), p(t.guess_right), p(t.guess_wrong));
    };
    for (k, t) in &report.by_kind {
        row(k, t);
    }
    row("ALL", &report.all);
    let made = report.all.guess_right + report.all.guess_wrong;
    if made > 0 {
        println!(
            "\nit made {made} labelled guesses; the hit rate it stated for them averaged {:.1}%, and {:.1}% were right",
            100.0 * report.all.stated / made as f64,
            100.0 * report.all.guess_right as f64 / made as f64
        );
    }
    if report.oracle.n > 0 {
        let o = &report.oracle;
        let p = |x: usize| 100.0 * x as f64 / o.n as f64;
        println!(
            "\nIF IT WERE TOLD EACH QUESTION'S KIND (the oracle: isolates the way of learning from recognising the kind), {} questions: right as fact {:.1}%, WRONG as fact {:.1}%, guess right {:.1}%, guess wrong {:.1}%, nothing {:.1}%\nrouting alone: {:.1}% of the questions went to the unit of their own kind",
            o.n, p(o.right), p(o.wrong), p(o.guess_right), p(o.guess_wrong), p(o.refused), p(o.routed_right)
        );
    }
    println!("\nwhy it refused:");
    for (why, n) in &report.refusals {
        println!("    {n:>5}  {why}");
    }
    if report.wrongs.is_empty() {
        println!("\nno answer stated as fact was wrong.");
    } else {
        println!("\n{} answers stated as fact were WRONG (every one listed):", report.wrongs.len());
        for (q, truth, said) in &report.wrongs {
            println!("    truth {truth}, said {said}: {q}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // hand-written fixture in SVAMP's shape (these are NOT from SVAMP)
    const FIXTURE: &str = r#"[
      {"ID": "t-1", "Body": "Anna had 5 pens. She bought 3 more pens.", "Question": "How many pens does Anna have now?", "Equation": "( 5.0 + 3.0 )", "Type": "Addition", "Answer": 8.0, "group_nums": [0, 1]},
      {"ID": "t-2", "Body": "A shelf holds 12 books. 4 books were taken.", "Question": "How many books are left?", "Equation": "( 12.0 - 4.0 )", "Type": "Subtraction", "Answer": 8.0},
      {"ID": "t-3", "Body": "Each box has 2.5 kg of rice.", "Question": "How much rice is in 2 boxes?", "Equation": "( 2.5 * 2.0 )", "Type": "Multiplication", "Answer": 5.0},
      {"ID": "t-4", "Body": "There are 9 sweets shared by 3 kids.", "Question": "How many does each kid get?", "Equation": "( 9.0 / 3.0 )", "Type": "Common-Division", "Answer": 3.0},
      {"ID": "t-5", "Body": "A \"big\" jar has 12000 beans.", "Question": "How many beans in 2 jars?", "Equation": "( 12000.0 * 2.0 )", "Type": "Multiplication", "Answer": 24000.0},
      {"ID": "t-6", "Body": "Tom has 7 cars.\nHe gave away 9.", "Question": "How many fewer cars does Tom have?", "Equation": "( 7.0 - 9.0 )", "Type": "Subtraction", "Answer": -2.0}
    ]"#;

    #[test]
    fn it_reads_the_word_problem_file_format_with_escapes_and_float_answers() {
        let p = load_svamp(FIXTURE).unwrap();
        assert_eq!(p.len(), 6);
        assert_eq!(p[0].id, "t-1");
        assert_eq!(p[0].query, "Anna had 5 pens. She bought 3 more pens. How many pens does Anna have now?");
        assert_eq!(p[0].answer, "8");
        assert_eq!(p[0].kind, "addition");
        assert!(p[4].query.contains("\"big\""), "{}", p[4].query);
        assert!(p[5].query.contains("7 cars. He gave"), "a newline in the text is whitespace: {}", p[5].query);
        assert_eq!(p[3].kind, "common-division");
        assert!(load_svamp("[{\"ID\": ").is_err());
        assert!(load_svamp("{\"a\": 1}").is_err());
    }

    #[test]
    fn it_says_exactly_why_it_cannot_be_given_a_problem() {
        let (kept, dropped) = readable(load_svamp(FIXTURE).unwrap());
        let ids: Vec<&str> = kept.iter().map(|p| p.id.as_str()).collect();
        assert_eq!(ids, ["t-1", "t-2", "t-4"]);
        let reasons: Vec<&str> = dropped.iter().map(|d| d.0).collect();
        assert!(reasons.iter().any(|r| r.contains("cannot read")), "{reasons:?}"); // 2.5
        assert!(reasons.iter().any(|r| r.contains("more than four digits")), "{reasons:?}"); // 12000
        assert!(reasons.iter().any(|r| r.contains("not a whole non-negative")), "{reasons:?}"); // -2
        assert_eq!(dropped.iter().map(|d| d.1).sum::<usize>(), 3);
    }

    #[test]
    fn it_loads_a_tsv_and_splits_without_losing_or_repeating_a_problem() {
        let p = load_tsv("# comment\nwhat is two plus two\t4\tsum\n\nwhat is three plus one\t4\n").unwrap();
        assert_eq!(p.len(), 2);
        assert_eq!(p[1].kind, "all");
        assert!(load_tsv("no tab here").is_err());
        let many: Vec<Problem> = (0..100).map(|i| Problem { id: i.to_string(), query: format!("q{i}"), answer: "1".into(), kind: "k".into() }).collect();
        for how in [Split::Random, Split::Blocks] {
            let (train, test) = split(&many, how, 0.25, 3);
            assert_eq!(train.len() + test.len(), 100, "{how:?}");
            assert!(test.len() >= 10 && test.len() <= 45, "{how:?}: {}", test.len());
            let mut ids: Vec<&str> = train.iter().chain(&test).map(|p| p.id.as_str()).collect();
            ids.sort();
            ids.dedup();
            assert_eq!(ids.len(), 100, "{how:?}");
        }
    }

    #[test]
    fn on_text_it_never_saw_it_refuses_and_says_why_and_never_states_a_wrong_fact() {
        // teach tidy "plus" sentences; test on sentences with words it never met
        let train: Vec<Problem> = (0..6)
            .flat_map(|a| (0..6).map(move |b| (a, b)))
            .map(|(a, b)| Problem { id: format!("{a}{b}"), query: format!("what is {a} plus {b}"), answer: (a + b).to_string(), kind: "plus".into() })
            .collect();
        let test = vec![
            Problem { id: "x".into(), query: "what is 2 plus 3".into(), answer: "5".into(), kind: "plus".into() },
            Problem { id: "y".into(), query: "my cousin found 2 shiny marbles then 3 more under the sofa so how many".into(), answer: "5".into(), kind: "plus".into() },
        ];
        let r = run(&train, &test, Units::ByKind, 1);
        assert_eq!(r.all.n, 2);
        assert_eq!(r.all.wrong, 0, "{:?}", r.wrongs);
        assert_eq!(r.all.right, 1);
        assert_eq!(r.all.refused, 1);
        assert_eq!(r.refusals[0].0, "a word it has never met");
    }
}
