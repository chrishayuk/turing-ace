//! Minor cycles: the 32-digit words of the machine, and the positions of
//! minor cycles in the delay lines.
//!
//! The report numbers the digits of a minor cycle 1 to 32 in the order they
//! arrive, least significant first. Here digit k is bit k-1 of a `u32`.

pub type Word = u32;

/// The pulse in digit position k, as a word.
pub const fn p(k: u32) -> Word {
    1 << (k - 1)
}

pub const P17: Word = p(17);
pub const P18: Word = p(18);
pub const P19: Word = p(19);

/// Digits 17 to 32, the half of a minor cycle that holds an instruction number.
pub const UPPER: Word = 0xFFFF_0000;

pub fn digit(w: Word, k: u32) -> bool {
    w & p(k) != 0
}

/// Digits `from..=to` as a number, `from` being the least significant.
pub fn field(w: Word, from: u32, to: u32) -> u32 {
    let width = to - from + 1;
    (w >> (from - 1)) & (((1u64 << width) - 1) as u32)
}

/// A word as the report prints one: digit 1 first, in groups of four.
pub fn bits(w: Word) -> String {
    let mut s = String::with_capacity(39);
    for k in 1..=32 {
        if k > 1 && k % 4 == 1 {
            s.push(',');
        }
        s.push(if digit(w, k) { '1' } else { '0' });
    }
    s
}

/// Digits 17 to 32 as the report prints the second half of an instruction.
pub fn half_bits(w: Word) -> String {
    bits(w)[20..].to_string()
}

/// Reads digits written first digit first, ignoring commas and spaces. The
/// first digit read is digit `first`.
pub fn parse_bits(text: &str, first: u32) -> Option<Word> {
    let mut w = 0;
    let mut k = first;
    for c in text.chars() {
        match c {
            '0' | '1' => {
                if k > 32 {
                    return None;
                }
                if c == '1' {
                    w |= p(k);
                }
                k += 1;
            }
            ',' | ' ' => {}
            _ => return None,
        }
    }
    Some(w)
}

/// Where a minor cycle is: a delay line, and its place in the 32 that
/// circulate in it. `slot` counts from 0 in the order they emerge; the report
/// writes DL m,n with n = slot + 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Pos {
    pub dl: u32,
    pub slot: u32,
}

impl Pos {
    /// The position named by digits 18 to 32 of a word (Fig. 19: digits 18-22
    /// are the time, 23-32 the delay line).
    pub fn of(w: Word) -> Pos {
        Pos { dl: field(w, 23, 32), slot: field(w, 18, 22) }
    }

    /// The position as digits 18 to 32 of a word.
    pub const fn word(self) -> Word {
        (self.slot << 17) | (self.dl << 22)
    }

    /// The instruction number Fig. 19 gives this position: within a delay
    /// line the even minor cycles are numbered 0 to 15 and the odd ones 16 to
    /// 31, so that adding a pulse in digit 19 goes to the next number.
    pub fn number(self) -> u32 {
        32 * self.dl + 16 * (self.slot & 1) + (self.slot >> 1)
    }

    /// The position `n` instructions on, following the rule for instructions
    /// of type A: add a pulse in digit 19.
    pub fn after(self, n: u32) -> Pos {
        Pos::of(self.word().wrapping_add(n.wrapping_mul(P19)))
    }
}

impl std::fmt::Display for Pos {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "DL {},{}", self.dl, self.slot + 1)
    }
}

/// The 64 digits of TS 4 and TS 5 in series, TS 4 being the less significant.
pub fn partial_sum(ts4: Word, ts5: Word) -> i64 {
    (((ts5 as u64) << 32) | ts4 as u64) as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn printed_words_round_trip() {
        let text = "0000,0101,0000,0000,0100,0110,0000,0000";
        let w = parse_bits(text, 1).unwrap();
        assert_eq!(bits(w), text);
        assert!(digit(w, 6) && digit(w, 8) && digit(w, 18) && digit(w, 22) && digit(w, 23));
    }

    #[test]
    fn numbering_follows_figure_19() {
        // Digit 19 weighs 1, 20 2, 21 4, 22 8, 18 16, 23 32.
        for (k, weight) in [(19, 1), (20, 2), (21, 4), (22, 8), (18, 16), (23, 32), (24, 64)] {
            assert_eq!(Pos::of(p(k)).number(), weight);
        }
        // Consecutive numbers are two minor cycles apart.
        let a = Pos { dl: 3, slot: 4 };
        assert_eq!(a.after(1), Pos { dl: 3, slot: 6 });
        assert_eq!(a.after(1).number(), a.number() + 1);
        // After number 15 of a line comes number 0 of the next, not 16.
        assert_eq!(Pos { dl: 3, slot: 30 }.after(1), Pos { dl: 4, slot: 0 });
    }
}
