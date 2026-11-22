//! The calculator of the 1945 report, at the level of whole instructions.
//!
//! What is simulated is sections 11 and 12 with Fig. 19: the delay lines, the
//! 32 temporary storages, the logical control (CD and CI) and the ten types
//! of operation of the arithmetic part. Valves, pulses and the circuits of
//! Figs. 12-36 are not simulated, but time is kept in minor cycles, since
//! where a word sits in its delay line decides how long it takes to reach.
//!
//! Where the report disagrees with itself, the reading taken here is one of
//! the numbered decisions in `crate::DECISIONS`.

use crate::word::{digit, field, partial_sum, Pos, Word, P17, P19, UPPER};
use std::collections::VecDeque;

/// The report proposes room for 200 delay lines (section 10).
pub const LINES: usize = 200;

/// Rows of a Hollerith card, and so minor cycles moved by types O and P.
pub const ROWS: usize = 12;

/// Minor cycles for a card to pass the reader or the punch. The report gives
/// no figure; this is 100 cards a minute.
pub const CARD_TIME: u64 = 18_750;

pub type Card = [Word; ROWS];

/// The ten types of operation, with the code Fig. 19 gives in digits 5-8.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
}

impl Type {
    /// Digits 5, 6, 7, 8 as Fig. 19 prints them.
    pub fn code(self) -> &'static str {
        match self {
            Type::K => "0100",
            Type::L => "0101",
            Type::M => "0010",
            Type::N => "0011",
            Type::O => "0001",
            Type::P => "0000",
            Type::Q => "0110",
            Type::R => "0111",
            Type::S => "1000",
            Type::T => "1001",
        }
    }

    pub fn letter(self) -> char {
        "KLMNOPQRST".chars().nth(self as usize).unwrap()
    }

    pub const ALL: [Type; 10] =
        [Type::K, Type::L, Type::M, Type::N, Type::O, Type::P, Type::Q, Type::R, Type::S, Type::T];

    /// The code as digits 5-8 of a word.
    pub fn word(self) -> Word {
        crate::word::parse_bits(self.code(), 5).unwrap()
    }

    /// The type of an instruction of type A, if its digits 5-8 are one of
    /// the ten codes in use.
    pub fn of(ci: Word) -> Option<Type> {
        Type::ALL.into_iter().find(|t| t.word() == ci & 0xF0)
    }
}

/// Signals of an operation of type S, with their digits (Figs. 19, 32, 37).
pub mod sig {
    pub const POS: u32 = 10;
    pub const CANCEL: u32 = 11;
    pub const SELF: u32 = 12;
    pub const NOR: u32 = 13;
    pub const GIV: u32 = 14;
    pub const ERR: u32 = 15;
    pub const ERR1: u32 = 16;
    pub const DIFF: u32 = 17;
    pub const AUTO: u32 = 18;
    pub const ROTATE: u32 = 19;
    pub const THREE: u32 = 20;
    /// The four digits of a given multiplier (Fig. 37).
    pub const GIVEN: (u32, u32) = (24, 27);
    pub const NAMES: [(&str, u32); 11] = [
        ("POS", POS),
        ("CANCEL", CANCEL),
        ("SELF", SELF),
        ("NOR", NOR),
        ("GIV", GIV),
        ("ERR", ERR),
        ("ERR1", ERR1),
        ("DIFF", DIFF),
        ("AUTO", AUTO),
        ("ROTATE", ROTATE),
        ("THREE", THREE),
    ];
}

/// The temporary storage a five-digit field names. There are 32 of them,
/// TS 1 to TS 32, and five digits count 0 to 31: 0 is taken to be TS 32.
pub fn ts_of(n: u32) -> usize {
    if n == 0 {
        32
    } else {
        n as usize
    }
}

/// The temporary storages that type N cannot fill: they "have no input
/// connections except those shown on other diagrams" (section 12).
pub const NO_INPUT: [usize; 4] = [1, 4, 5, 8];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stop {
    /// An instruction of type B that names itself: nothing further can happen.
    Idle(Pos),
    /// Type P with no card in the reader. The machine waits.
    Reader,
    /// A position in a delay line the machine does not have.
    NoLine(u32),
    /// The step limit of the simulator.
    Limit,
}

impl std::fmt::Display for Stop {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Stop::Idle(_) => write!(f, "the instruction passes control to itself"),
            Stop::Reader => write!(f, "waiting for a card that is not in the reader"),
            Stop::NoLine(n) => write!(f, "there is no delay line {} (the machine has {})", n, LINES),
            Stop::Limit => write!(f, "the step limit was reached"),
        }
    }
}

/// One instruction obeyed.
#[derive(Debug, Clone)]
pub struct Event {
    /// CD when the instruction was looked for.
    pub cd: Word,
    /// The instruction, as it stood in CI.
    pub ci: Word,
    /// The minor cycle in which it reached CI, and the one after it finished.
    pub fetched: u64,
    pub done: u64,
    /// What changed, in words.
    pub effect: String,
}

impl Event {
    pub fn from_ts6(&self) -> bool {
        self.cd & P17 != 0
    }
}

pub struct Machine {
    /// TS 1 to TS 32; index 0 is unused.
    pub ts: [Word; 33],
    pub lines: Vec<[Word; 32]>,
    /// Current data: the position of the next instruction, in digits 17-32.
    pub cd: Word,
    /// Minor cycles since the start. Minor cycle t of every delay line is the
    /// one in slot t mod 32.
    pub time: u64,
    pub steps: u64,
    pub reader: VecDeque<Card>,
    pub punched: Vec<Card>,
    /// Valve elements stimulated by type T, in order.
    pub flex: Vec<u32>,
}

impl Default for Machine {
    fn default() -> Self {
        Machine::new()
    }
}

/// The first minor cycle at or after `from` in which `slot` is at the end of
/// its delay line.
fn arrives(from: u64, slot: u32) -> u64 {
    from + (slot as u64 + 32 - from % 32) % 32
}

impl Machine {
    pub fn new() -> Machine {
        Machine {
            ts: [0; 33],
            lines: vec![[0; 32]; LINES],
            cd: 0,
            time: 0,
            steps: 0,
            reader: VecDeque::new(),
            punched: Vec::new(),
            flex: Vec::new(),
        }
    }

    pub fn read(&self, at: Pos) -> Result<Word, Stop> {
        self.lines.get(at.dl as usize).map(|l| l[at.slot as usize]).ok_or(Stop::NoLine(at.dl))
    }

    pub fn write(&mut self, at: Pos, w: Word) -> Result<(), Stop> {
        match self.lines.get_mut(at.dl as usize) {
            Some(l) => {
                l[at.slot as usize] = w;
                Ok(())
            }
            None => Err(Stop::NoLine(at.dl)),
        }
    }

    /// The partial sum: TS 4 and TS 5 in series.
    pub fn sum(&self) -> i64 {
        partial_sum(self.ts[4], self.ts[5])
    }

    fn set_sum(&mut self, v: i64) {
        self.ts[4] = v as u64 as u32;
        self.ts[5] = ((v as u64) >> 32) as u32;
    }

    /// Obey one instruction.
    pub fn step(&mut self) -> Result<Event, Stop> {
        let cd = self.cd;
        // The search for the new CI. A 1 in digit 17 takes it from TS 6.
        let (ci, fetched) = if cd & P17 != 0 {
            (self.ts[6], self.time)
        } else {
            let at = Pos::of(cd);
            let w = self.read(at)?;
            (w, arrives(self.time, at.slot))
        };
        let mut done = fetched + 1;
        let effect;
        if digit(ci, 3) {
            // Type B: the new CD is CI 17-32, and the old CD retires to TS 1.
            let new = ci & UPPER;
            if new == cd && cd & P17 == 0 {
                return Err(Stop::Idle(Pos::of(cd)));
            }
            self.ts[1] = cd;
            self.cd = new;
            effect = String::from("TS 1 = the position left");
        } else {
            let (text, took) = self.operate(ci, done)?;
            effect = text;
            done += took;
            // Type A: CD = (CD' + P19) & -P17.
            self.cd = cd.wrapping_add(P19) & !P17 & UPPER;
        }
        self.time = done;
        self.steps += 1;
        Ok(Event { cd, ci, fetched, done, effect })
    }

    /// The operation of CA that CI 5-32 asks for, beginning in minor cycle
    /// `start`. Returns what it did and the minor cycles it took.
    fn operate(&mut self, ci: Word, start: u64) -> Result<(String, u64), Stop> {
        let Some(kind) = Type::of(ci) else {
            return Ok((String::from("digits 5-8 name no type of operation: nothing happens"), 1));
        };
        let at = Pos::of(ci);
        let ts = ts_of(field(ci, 10, 14));
        Ok(match kind {
            Type::K => {
                self.write(at, self.ts[6])?;
                (format!("{} = {}", at, show(self.ts[6])), arrives(start, at.slot) - start + 1)
            }
            Type::L => {
                self.ts[6] = self.read(at)?;
                (format!("TS 6 = {}", show(self.ts[6])), arrives(start, at.slot) - start + 1)
            }
            Type::M => {
                self.ts[6] = self.ts[ts];
                (format!("TS 6 = {}", show(self.ts[6])), 1)
            }
            Type::N => {
                if NO_INPUT.contains(&ts) {
                    (format!("TS {} has no input from TS 6: nothing happens", ts), 1)
                } else {
                    self.ts[ts] = self.ts[6];
                    (format!("TS {} = {}", ts, show(self.ts[6])), 1)
                }
            }
            Type::O => {
                let line = self.lines.get(at.dl as usize).ok_or(Stop::NoLine(at.dl))?;
                let mut card = [0; ROWS];
                card.copy_from_slice(&line[..ROWS]);
                self.punched.push(card);
                (format!("card {} punched from DL {}", self.punched.len(), at.dl), CARD_TIME)
            }
            Type::P => {
                if at.dl as usize >= self.lines.len() {
                    return Err(Stop::NoLine(at.dl));
                }
                let card = self.reader.pop_front().ok_or(Stop::Reader)?;
                self.lines[at.dl as usize][..ROWS].copy_from_slice(&card);
                (format!("a card read into DL {}", at.dl), CARD_TIME)
            }
            Type::Q => {
                // CATIM lasts from P17 to P32 only, so digits 1-16 of TS 6
                // go on circulating.
                self.ts[6] = (self.ts[6] & !UPPER) | (ci & UPPER);
                (format!("TS 6 = {}", show(self.ts[6])), 1)
            }
            Type::R => {
                let (a, b) = (self.ts[9], self.ts[10]);
                // The operation is chosen by the tree elements TRA (digits
                // 23-25) and TRB 000 (digits 26-28), as Fig. 29 draws it.
                let tra = (digit(ci, 23), digit(ci, 24), digit(ci, 25));
                self.ts[8] = match tra {
                    _ if field(ci, 26, 28) != 0 => 0,
                    (false, false, false) => a & b,
                    (false, false, true) => a | b,
                    (false, true, false) => !b,
                    (false, true, true) => !(a ^ b),
                    _ => 0,
                };
                (format!("TS 8 = {}", show(self.ts[8])), 1)
            }
            Type::S => {
                let took = self.arithmetic(ci);
                (format!("TS 5, TS 4 = {} ({})", show64(self.sum()), self.sum()), took)
            }
            Type::T => {
                let element = field(ci, 18, 23);
                self.flex.push(element);
                (format!("valve element FLEX {} stimulated", element), 1)
            }
        })
    }

    /// Type S. One process, modified by the signals: a multiple of the
    /// multiplicand is added into the partial sum or taken from it.
    fn arithmetic(&mut self, ci: Word) -> u64 {
        let on = |k| digit(ci, k);
        let sum = self.sum();
        // The multiplicand: the partial sum itself, or TS 2 with its most
        // significant digit repeated to make 64 (MUCAND 2).
        let cand = if on(sig::SELF) { sum } else { self.ts[2] as i32 as i64 };
        let mut took = 2;
        let multiple = if on(sig::AUTO) {
            // The fundamental process eight times, taking four digits of the
            // multiplier each time: a whole multiplier, sign and all.
            took = 16;
            let multiplier = if on(sig::ERR) { self.ts[11] } else { self.ts[3] };
            cand.wrapping_mul(multiplier as i32 as i64)
        } else if on(sig::NOR) || on(sig::GIV) || on(sig::ERR) {
            let (source, mut first) = if on(sig::GIV) {
                (ci, sig::GIVEN.0)
            } else if on(sig::ERR) {
                (self.ts[11], if on(sig::ERR1) { 10 } else { 14 })
            } else {
                (self.ts[3], 1)
            };
            if on(sig::DIFF) && self.ts[12] != 0 {
                first = self.ts[12].trailing_zeros() + 1;
            }
            cand.wrapping_mul((source.rotate_right(first - 1) & 15) as i64)
        } else {
            0
        };
        let mut sum = if on(sig::CANCEL) { 0 } else { sum };
        sum = if on(sig::POS) { sum.wrapping_add(multiple) } else { sum.wrapping_sub(multiple) };
        if on(sig::ROTATE) {
            // Multiplication by a power of 2, the amount in digits 1-5 of
            // TS 7. The 64 digits are a closed cycle, so those that leave the
            // top return at the bottom.
            sum = (sum as u64).rotate_left(self.ts[7] & 31) as i64;
            took = if on(sig::THREE) { 3 } else { 2 };
        }
        self.set_sum(sum);
        took
    }
}

/// A word for a trace: its digits, and what number it is.
pub fn show(w: Word) -> String {
    crate::word::bits(w)
}

fn show64(v: i64) -> String {
    format!("{} {}", crate::word::bits(((v as u64) >> 32) as u32), crate::word::bits(v as u64 as u32))
}

/// A card as a listing: one character for each of the 32 columns. Minor
/// cycles 1 to 12 of the card are the rows 12, 11, 0, 1, ... 9, so a single
/// hole in a column is a digit, a hole in row 11 a minus sign, and one in
/// row 12 a plus sign. A column with several holes prints as `*`.
pub fn listing(card: &Card) -> String {
    let mut line = String::new();
    for column in 1..=32 {
        let rows: Vec<usize> = (0..ROWS).filter(|r| digit(card[*r], column)).collect();
        line.push(match rows.as_slice() {
            [] => ' ',
            [0] => '+',
            [1] => '-',
            [r] => char::from(b'0' + (*r as u8 - 2)),
            _ => '*',
        });
    }
    line.trim_end().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::word::p;

    #[test]
    fn codes_are_those_of_figure_19() {
        assert_eq!(Type::L.word(), p(6) | p(8));
        assert_eq!(Type::of(p(5)), Some(Type::S));
        assert_eq!(Type::of(0), Some(Type::P));
        assert_eq!(Type::of(p(5) | p(6)), None);
    }

    #[test]
    fn a_blank_instruction_reads_a_card_into_line_0() {
        let mut m = Machine::new();
        let mut card = [0; ROWS];
        card[3] = 99;
        m.reader.push_back(card);
        m.step().unwrap();
        assert_eq!(m.lines[0][3], 99);
        // And the next instruction is number 1 of the same line, two minor cycles on.
        assert_eq!(Pos::of(m.cd), Pos { dl: 0, slot: 2 });
    }

    #[test]
    fn waiting_for_a_minor_cycle_takes_time() {
        assert_eq!(arrives(0, 0), 0);
        assert_eq!(arrives(1, 0), 32);
        assert_eq!(arrives(33, 5), 37);
    }

    #[test]
    fn the_sum_keeps_its_sign_over_64_digits() {
        let mut m = Machine::new();
        m.ts[2] = (-5i32) as u32;
        // Add TS 2, cancelling: POS, CANCEL, GIV, multiplier 1.
        m.arithmetic(Type::S.word() | p(sig::POS) | p(sig::CANCEL) | p(sig::GIV) | p(24));
        assert_eq!(m.sum(), -5);
        assert_eq!(m.ts[5], u32::MAX);
    }
}
