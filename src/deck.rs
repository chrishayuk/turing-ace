//! Starting from cold: a job as a deck of cards that loads itself.
//!
//! Section 7 of the report: "Arrangements are made for setting into CI and
//! CD a certain invariable initial order and IN. These state that the card
//! is to be transferred into a particular delay line, and that the next
//! order is to be taken from a particular spot, which will actually be in
//! this same delay line. ... The first few orders obeyed will probably be to
//! take in a few more cards. ... Actually it has been arranged that the
//! special initial order consists of 0 throughout so that there is no need
//! to set it up."
//!
//! An instruction of 32 zeros is type P with delay line 0: read the card in
//! the reader into the first twelve minor cycles of DL 0. A machine with
//! nothing in it holds that instruction everywhere. Started at DL 0,1 it
//! reads a card into DL 0, and then, being type A, goes on two minor cycles
//! later, at DL 0,3: the third row of the card it has just read.
//!
//! The report gives the start and not the cards. The format here is ours.
//! Every card of the deck carries its own orders, in the rows that will be
//! obeyed, and two words, in the rows that will not:
//!
//! | row | minor cycle | content |
//! |-----|-------------|---------|
//! | 1   | DL 0,1      | 0: read the next card into DL 0 (the initial order) |
//! | 2   | DL 0,2      | the first word |
//! | 3   | DL 0,3      | DL 0,2 - TS 6 |
//! | 4   | DL 0,4      | the second word |
//! | 5   | DL 0,5      | TS 6 - its destination |
//! | 7   | DL 0,7      | DL 0,4 - TS 6 |
//! | 9   | DL 0,9      | TS 6 - its destination |
//! | 11  | DL 0,11     | B, DL 0,1: back to row 1, which reads the next card |
//!
//! A card with one word has its B in row 7. A row of zeros that was obeyed
//! would read a card itself.
//!
//! Cards that set a temporary storage use type N in rows 5 and 9 instead of
//! K. The last card puts the job's TS 6 into TS 6, zero if it gives none, and
//! passes control to the first instruction of the job. The job's own cards of data
//! follow the deck in the reader.

use crate::assemble::Unit;
use crate::machine::{Card, Machine, Type, ROWS};
use crate::word::{p, Pos, Word};

/// A card, and what it puts where, for people.
pub struct Punched {
    pub card: Card,
    pub what: String,
}

fn l(slot: u32) -> Word {
    Type::L.word() | Pos { dl: 0, slot }.word()
}

fn b(to: Pos) -> Word {
    p(2) | p(3) | to.word()
}

/// Where a word goes: a minor cycle, or a temporary storage.
#[derive(Clone, Copy)]
enum To {
    Cell(Pos),
    Ts(usize),
}

impl To {
    fn store(self) -> Word {
        match self {
            To::Cell(pos) => Type::K.word() | pos.word(),
            To::Ts(n) => Type::N.word() | ((n as u32) % 32) << 9,
        }
    }
    fn name(self) -> String {
        match self {
            To::Cell(pos) => pos.to_string(),
            To::Ts(n) => format!("TS {}", n),
        }
    }
}

/// The deck that loads `unit` into an empty machine and starts it.
pub fn deck(unit: &Unit) -> Result<Vec<Punched>, String> {
    if let Some(p) = unit.errors().first() {
        return Err(format!("line {}: {}", p.span.line + 1, p.message));
    }
    let mut words: Vec<(To, Word, String)> = Vec::new();
    for table in &unit.tables {
        for slot in &table.slots {
            if slot.word != 0 {
                words.push((To::Cell(slot.pos), slot.word, format!("{} {}", table.name, slot.fig)));
            }
        }
    }
    for (pos, w, _) in &unit.store {
        if *w != 0 {
            words.push((To::Cell(*pos), *w, pos.to_string()));
        }
    }
    for (to, _, _) in &words {
        if let To::Cell(pos) = to {
            if pos.dl == 0 && (pos.slot as usize) < ROWS {
                return Err(format!(
                    "{} is where each card of the deck is read: a job loaded from cold cannot keep anything there",
                    pos
                ));
            }
        }
    }
    let mut ts6 = None;
    for (n, w) in crate::STANDING.iter().map(|(n, w, _)| (*n, *w)).chain(unit.ts.iter().map(|(n, w, _)| (*n, *w))) {
        match n {
            6 => ts6 = Some(w),
            1 | 4 | 5 | 8 => {
                return Err(format!("TS {} cannot be filled from TS 6, so a deck cannot set it", n));
            }
            _ => {
                words.retain(|(to, _, _)| !matches!(to, To::Ts(m) if *m == n));
                if w != 0 {
                    words.push((To::Ts(n), w, format!("TS {}", n)));
                }
            }
        }
    }
    let entry = unit.entry.ok_or("there is no instruction to start at")?;

    let mut out = Vec::new();
    for pair in words.chunks(2) {
        let mut card: Card = [0; ROWS];
        card[1] = pair[0].1;
        card[2] = l(1);
        card[4] = pair[0].0.store();
        let mut what = format!("{} into {}", pair[0].2, pair[0].0.name());
        // A row of zeros obeyed would read a card: the way back comes straight
        // after the last order the card needs.
        match pair.get(1) {
            Some(second) => {
                card[3] = second.1;
                card[6] = l(3);
                card[8] = second.0.store();
                card[10] = b(Pos { dl: 0, slot: 0 });
                what.push_str(&format!(", {} into {}", second.2, second.0.name()));
            }
            None => card[6] = b(Pos { dl: 0, slot: 0 }),
        }
        out.push(Punched { card, what });
    }
    let mut last: Card = [0; ROWS];
    let name = unit.name_of(entry).unwrap_or_else(|| entry.to_string());
    // TS 6 is left as the job expects it, zero unless it says otherwise, not
    // holding the last word the deck moved.
    last[1] = ts6.unwrap_or(0);
    last[2] = l(1);
    last[4] = b(entry);
    out.push(Punched { card: last, what: format!("start: go to {}", name) });
    Ok(out)
}

/// An empty machine with the deck for `unit` in the reader, followed by the
/// job's own cards, about to obey the initial order at DL 0,1.
pub fn cold(unit: &Unit, input: &[Word]) -> Result<(Machine, usize), String> {
    let deck = deck(unit)?;
    let mut m = Machine::new();
    for punched in &deck {
        m.reader.push_back(punched.card);
    }
    for chunk in input.chunks(ROWS) {
        let mut card: Card = [0; ROWS];
        card[..chunk.len()].copy_from_slice(chunk);
        m.reader.push_back(card);
    }
    m.cd = Pos { dl: 0, slot: 0 }.word();
    Ok((m, deck.len()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assemble::assemble;
    use crate::machine::Stop;

    #[test]
    fn a_job_loads_itself_from_cold() {
        let text = "ECHO\n  1   P, DL 2\n  2   O, DL 2\n  3   B, ECHO 3\n";
        let unit = assemble(text, &["all"]);
        let (mut m, n) = cold(&unit, &[7, 8, 9]).unwrap();
        // Three instructions and TS 7 and TS 31 make five words: three cards, and the start.
        assert_eq!(n, 4);
        let stop = crate::run(&mut m, 10_000, |_, _| {});
        assert!(matches!(stop, Stop::Idle(_)), "{}", stop);
        assert_eq!(m.punched.len(), 1);
        assert_eq!(&m.punched[0][..3], &[7, 8, 9]);
        assert_eq!(m.ts[7], 16);
    }
}
