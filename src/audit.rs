//! The report compared with itself, and its tables tried on its machine.
//!
//! Fig. 38 shows INDEXIN a second time, as the cards that would be punched:
//! the "genuine input", digit by digit. Section 13 says "We use the
//! convention that no digit is shown if the value of the digit is not
//! significant." The figure is transcribed here as drawn and set against
//! the table it is the cards for.

use crate::assemble::{assemble, Unit};
use crate::machine::Stop;
use crate::word::{bits, digit, p, Pos, Word};
use crate::{library, load, run};

/// One card of Fig. 38: the digits shown, as (first column, digits), and the
/// instruction number written across columns 18-32 in words, if any.
pub struct Row {
    pub shown: &'static [(u32, &'static str)],
    pub to: Option<&'static str>,
}

pub const FIG_38: [Row; 13] = [
    Row { shown: &[(3, "0"), (5, "0110"), (17, "0000010000000000")], to: None },
    Row { shown: &[(3, "0"), (5, "0011"), (10, "01000")], to: None },
    Row { shown: &[(3, "0"), (5, "1000"), (10, "110100000000")], to: None },
    Row { shown: &[(3, "0"), (5, "1000"), (10, "0000000001")], to: None },
    Row { shown: &[(3, "0"), (5, "0010"), (10, "00100")], to: None },
    Row { shown: &[(3, "0"), (5, "0011"), (10, "10010")], to: None },
    Row { shown: &[(3, "0"), (5, "0010"), (10, "11011")], to: None },
    Row { shown: &[(3, "0"), (5, "0011"), (10, "01010")], to: None },
    Row { shown: &[(3, "0"), (5, "0111"), (23, "000")], to: None },
    Row { shown: &[(3, "0"), (5, "0010"), (10, "00100")], to: None },
    Row { shown: &[(3, "1"), (17, "1")], to: Some("INDEXIN 11") },
    Row { shown: &[(3, "0"), (5, "0011"), (10, "00111")], to: None },
    Row { shown: &[(3, "1")], to: Some("BURY") },
];

impl Row {
    /// The digits the card shows: (which are shown, what they are).
    pub fn digits(&self) -> (Word, Word) {
        let (mut mask, mut value) = (0, 0);
        for (first, digits) in self.shown {
            for (i, c) in digits.chars().enumerate() {
                mask |= p(first + i as u32);
                if c == '1' {
                    value |= p(first + i as u32);
                }
            }
        }
        (mask, value)
    }

    /// The card as drawn: a space where no digit is shown.
    pub fn drawn(&self) -> String {
        let (mask, value) = self.digits();
        (1..=32)
            .map(|k| match (digit(mask, k), digit(value, k)) {
                (false, _) => if self.to.is_some() && k >= 18 { '~' } else { ' ' },
                (true, true) => '1',
                (true, false) => '0',
            })
            .collect()
    }
}

/// INDEXIN as the cards of Fig. 38 have it, in the notation: each card's
/// digits written out, the two instruction numbers kept as names.
pub fn fig_38_table() -> String {
    let mut s = String::from("INDEXIN\n");
    for (n, row) in FIG_38.iter().enumerate() {
        let (_, value) = row.digits();
        match row.to {
            Some("BURY") => s.push_str(&format!("{:>4}   B, BURY\n", n + 1)),
            Some(_) => s.push_str(&format!("{:>4}   B, 1, INDEXIN 11\n", n + 1)),
            None => s.push_str(&format!("{:>4}   {}\n", n + 1, bits(value))),
        }
    }
    s
}

/// The job the tables are tried on: fetch the number whose position is in
/// TS 27, by INDEXIN, and stop.
pub const TRIAL: &str = "TRIAL\n  1   B, BURY\n  2   B, INDEXIN\n  3   B, TRIAL 3\n\nTS 27   DL 3,5\nDL 3,5  1945\n";

pub struct Outcome {
    pub fetched: Word,
    pub cell: Word,
    pub stop: Stop,
    pub steps: u64,
    pub at: String,
}

impl Outcome {
    pub fn works(&self) -> bool {
        self.fetched == 1945 && self.cell == 1945 && matches!(self.stop, Stop::Idle(_))
    }

    pub fn told(&self) -> String {
        let end = match &self.stop {
            Stop::Idle(_) => format!("came back after {} instructions", self.steps),
            Stop::Limit => format!("was still running after {} instructions, at {}", self.steps, self.at),
            stop => format!("stopped at {} after {} instructions: {}", self.at, self.steps, stop),
        };
        let cell = if self.cell == 1945 { String::new() } else { format!("; DL 3,5 now holds {}", self.cell as i32) };
        format!("TS 28 = {}{}; {}", self.fetched as i32, cell, end)
    }
}

/// Runs the trial with a given INDEXIN (or the library's, under `errata`).
pub fn trial(indexin: Option<&str>, errata: &[&str]) -> Outcome {
    let text = format!("{}\n{}", TRIAL, indexin.unwrap_or(""));
    let unit: Unit = assemble(&text, errata);
    let mut m = load(&unit, &[]).expect("the trial assembles");
    let stop = run(&mut m, 2000, |_, _| {});
    let pos = Pos::of(m.cd);
    Outcome {
        fetched: m.ts[28],
        cell: m.lines[3][4],
        stop,
        steps: m.steps,
        at: unit.name_of(pos).unwrap_or_else(|| pos.to_string()),
    }
}

/// Which word DISCRIM passes to TS 24 for a given TS 8: true if TS 15.
pub fn discrim(ts8: Word, errata: &[&str]) -> bool {
    let text = format!("ASK\n  1   B, BURY\n  2   B, DISCRIM\n  3   B, ASK 3\n\nTS 8   {}\nTS 15  +15\nTS 16  +16\n", bits(ts8));
    let unit = assemble(&text, errata);
    let mut m = load(&unit, &[]).expect("the question assembles");
    run(&mut m, 5000, |_, _| {});
    m.ts[24] == 15
}

/// The job CALPOL is tried on: x^3 - 2x + 5 at x = 7, which is 334, worked
/// out `times` times in a row. Returns TS 25, or why there is none.
pub fn calpol(times: usize, errata: &[&str]) -> Result<i32, String> {
    let mut text = String::from("POLY\n");
    for _ in 0..times {
        text.push_str("      B, BURY;  B, CALPOL\n");
    }
    text.push_str("      B, POLY 1\n");
    // The last B names the first instruction after the calls: number 1 + 2 * times.
    let text = text.replace("B, POLY 1\n", &format!("B, POLY {}\n", 1 + 2 * times));
    let text = format!("{}\nTS 13    7\nDL 3,25  +1\nDL 3,29  -2\nDL 3,31  +5\n", text);
    let unit = assemble(&text, errata);
    let mut m = load(&unit, &[])?;
    match run_watching_for_a_loop(&mut m, 400_000) {
        Ok(Stop::Idle(_)) => Ok(m.ts[25] as i32),
        Err(steps) => Err(format!("loops: after {} instructions the machine is exactly as it was before, so it never stops", steps)),
        Ok(stop) => {
            let pos = Pos::of(m.cd);
            let at = if m.cd & crate::word::P17 != 0 { "TS 6".to_string() } else { unit.name_of(pos).unwrap_or_else(|| pos.to_string()) };
            Err(format!("stopped at {}: {}", at, stop))
        }
    }
}

/// Everything that decides what the machine does next. Time is left out:
/// it changes only how long things take, never what they do.
fn state(m: &crate::machine::Machine) -> (u32, [Word; 33], Vec<[Word; 32]>, usize, usize) {
    (m.cd, m.ts, m.lines.clone(), m.reader.len(), m.punched.len())
}

/// Runs like `run`, but every 997 instructions keeps the whole state of the
/// machine. The machine is deterministic, so if a state comes round again it
/// will go round for ever: `Err` with the number of instructions obeyed.
/// Once inside a loop of period p, the state at two kept instructions p·997
/// apart is the same, so a loop is found within p·997 instructions of
/// entering it.
pub fn run_watching_for_a_loop(m: &mut crate::machine::Machine, limit: u64) -> Result<Stop, u64> {
    use std::collections::HashMap;
    let mut seen: HashMap<u64, Vec<(u32, [Word; 33], Vec<[Word; 32]>, usize, usize)>> = HashMap::new();
    loop {
        if m.steps >= limit {
            return Ok(Stop::Limit);
        }
        if m.steps % 997 == 0 {
            let now = state(m);
            let key = {
                use std::hash::{Hash, Hasher};
                let mut h = std::collections::hash_map::DefaultHasher::new();
                now.hash(&mut h);
                h.finish()
            };
            let bucket = seen.entry(key).or_default();
            if bucket.iter().any(|earlier| *earlier == now) {
                return Err(m.steps);
            }
            bucket.push(now);
        }
        if let Err(stop) = m.step() {
            return Ok(stop);
        }
    }
}

pub const CALPOL_ERRATA: [&str; 5] = ["clear", "inputs", "branch", "carry", "contents"];

pub fn report() -> String {
    let mut s = String::new();
    let printed = assemble(&library("INDEXIN", &["none"]).unwrap(), &["none"]);
    let table = printed.table("INDEXIN").unwrap();
    s.push_str("1. The cards of Fig. 38 against the table they are cards for (INDEXIN as printed)\n\n");
    s.push_str("        1   5    10   15   20   25   30\n");
    let mut differ = 0;
    for (n, row) in FIG_38.iter().enumerate() {
        let (mask, value) = row.digits();
        let word = table.slots[n].word;
        let wrong: Vec<u32> = (1..=32).filter(|k| digit(mask, *k) && digit(value, *k) != digit(word, *k)).collect();
        let popular = printed.popular(word);
        s.push_str(&format!("  {:>2}    {}  {:<24}", n + 1, row.drawn(), popular));
        if wrong.is_empty() {
            s.push_str("agrees\n");
        } else {
            differ += 1;
            let list: Vec<String> = wrong.iter().map(|k| k.to_string()).collect();
            s.push_str(&format!("DIFFERS in digit{} {}: the card is {}\n", if wrong.len() == 1 { "" } else { "s" }, list.join(", "), printed.popular(value)));
        }
    }
    s.push_str(&format!("\n  {} of the 13 cards do not say what the table says. (~ marks an instruction number written in words.)\n\n", differ));

    s.push_str("2. INDEXIN on the machine: fetch the 1945 kept in DL 3,5, whose position is in TS 27\n\n");
    let rows: Vec<(String, Outcome)> = vec![
        ("the table as printed".into(), trial(None, &["none"])),
        ("the cards of Fig. 38".into(), trial(Some(&fig_38_table()), &["none"])),
        ("the table, all corrections".into(), trial(None, &["all"])),
    ];
    for (name, outcome) in &rows {
        s.push_str(&format!("  {:<28}{}  {}\n", name, if outcome.works() { "works " } else { "FAILS " }, outcome.told()));
    }
    s.push_str("\n3. Each correction to INDEXIN left out in turn\n\n");
    for id in ["literal", "half", "return"] {
        let others: Vec<&str> = ["literal", "half", "return", "sign"].into_iter().filter(|e| *e != id).collect();
        let outcome = trial(None, &others);
        s.push_str(&format!("  without `{}`{}{}  {}\n", id, " ".repeat(15 - id.len()), if outcome.works() { "works " } else { "FAILS " }, outcome.told()));
    }
    s.push_str("\n4. CALPOL on the machine: x^3 - 2x + 5 at x = 7, which is 334\n\n");
    let say = |r: Result<i32, String>| match r {
        Ok(334) => "334".to_string(),
        Ok(v) => format!("{}  WRONG", v),
        Err(e) if e.starts_with("loops") => "loops for ever".to_string(),
        Err(e) if e.contains("step limit") => "still running at the limit".to_string(),
        Err(e) if e.contains("card") => "waits for a card".to_string(),
        Err(e) => e,
    };
    s.push_str(&format!("  {:<28}{:<22}{}\n", "", "once", "twice in a row"));
    let line = |name: &str, errata: &[&str]| format!("  {:<28}{:<22}{}\n", name, say(calpol(1, errata)), say(calpol(2, errata)));
    s.push_str(&line("as printed", &["literal", "half", "return", "sign"]));
    s.push_str(&line("all corrections", &["all"]));
    for id in CALPOL_ERRATA {
        let others: Vec<&str> = crate::ERRATA.iter().map(|e| e.id).filter(|e| *e != id).collect();
        s.push_str(&line(&format!("without `{}`", id), &others));
    }
    s.push_str("\n  INDEXIN, BURY, UNBURY and DISCRIM are corrected in every row: CALPOL calls them.\n");
    s.push_str("\n5. DISCRIM: does TS 8 contain any digit 1?\n\n");
    for (name, w) in [("no digits", 0), ("digit 1 alone", p(1)), ("digit 32 alone", p(32)), ("all 32 digits", u32::MAX)] {
        let say = |yes: bool| if yes { "yes" } else { "no " };
        let (outlined, corrected) = (discrim(w, &["none"]), discrim(w, &["all"]));
        let right = (w != 0) == outlined;
        s.push_str(&format!("  {:<16}as outlined: {}{}   corrected: {}\n", name, say(outlined), if right { "        " } else { "  WRONG " }, say(corrected)));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_cards_of_fig_38_differ_from_the_table() {
        let text = report();
        assert!(text.contains("3 of the 13 cards"), "{}", text);
        for line in [3, 9, 10] {
            let row = text.lines().find(|l| l.starts_with(&format!("  {:>2}    ", line))).unwrap();
            assert!(row.contains("DIFFERS"), "{}", row);
        }
    }

    #[test]
    fn indexin_runs_only_when_corrected() {
        assert!(!trial(None, &["none"]).works());
        assert!(!trial(Some(&fig_38_table()), &["none"]).works());
        assert!(trial(None, &["all"]).works());
        for id in ["literal", "half", "return"] {
            let others: Vec<&str> = ["literal", "half", "return"].into_iter().filter(|e| *e != id).collect();
            assert!(!trial(None, &others).works(), "INDEXIN works without `{}`", id);
        }
    }

    #[test]
    fn calpol_needs_every_correction() {
        assert_eq!(calpol(1, &["all"]), Ok(334));
        assert_eq!(calpol(2, &["all"]), Ok(334));
        for id in CALPOL_ERRATA {
            let others: Vec<&str> = crate::ERRATA.iter().map(|e| e.id).filter(|e| *e != id).collect();
            assert_ne!(calpol(2, &others), Ok(334), "CALPOL works twice without `{}`", id);
        }
        assert_eq!(calpol(1, &["literal", "half", "return", "sign", "inputs", "branch", "carry", "contents"]), Ok(334));
    }

    #[test]
    fn discrim_as_outlined_misreads_digit_32() {
        assert!(discrim(1, &["none"]) && !discrim(0, &["none"]));
        assert!(!discrim(p(32), &["none"]) && !discrim(u32::MAX, &["none"]));
        for w in [1, p(32), u32::MAX, 0x8000_0001, 12345] {
            assert!(discrim(w, &["all"]));
        }
        assert!(!discrim(0, &["all"]));
    }
}
