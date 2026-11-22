//! The popular form of an instruction table, as section 13 of the report
//! writes one, read into instructions.
//!
//! A table is a name on a line of its own and then its instructions. They
//! may be set out one to a line with the number of the instruction in front
//! and the number of the next behind, as INDEXIN is:
//!
//! ```text
//! INDEXIN
//!   1   Q, 0000,0100,0000,0000   2
//!   2   TS 6 - TS 2              3
//! ```
//!
//! or several to a line with semicolons between, as CALPOL is:
//!
//! ```text
//! CALPOL 8. B, BURY; B, INDEXIN; TS 28 - TS 18;
//! ```
//!
//! Outside the report's own usage there are three additions. `#` begins a
//! comment. A line `DL 1,14  <word>` or `TS 27  <word>` gives a storage its
//! content before the start ("Before starting we require special contents in
//! DL 1,14"). And every type of operation can be written by its letter, for
//! the ones the report never abbreviates: `O, DL 10`, `S, POS, GIV, 6`.

use crate::machine::sig;
use crate::word::{parse_bits, Word};

/// A stretch of one line, in characters. Lines count from 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    pub line: usize,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Problem {
    pub span: Span,
    pub error: bool,
    pub message: String,
}

/// An instruction named by table and detail figure: INDEXIN 11.
#[derive(Debug, Clone, PartialEq)]
pub struct Ref {
    pub table: String,
    pub fig: u32,
    /// The name alone, and the name with its figure.
    pub name: Span,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Place {
    Ts(u32),
    /// DL m,n: the nth minor cycle of delay line m, n from 1 to 32.
    Dl(u32, u32),
    Tab(Ref),
}

/// What digits 17-32 of a Q or a B hold.
#[derive(Debug, Clone, PartialEq)]
pub enum Half {
    Bits(Word),
    Place(Place),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    /// `B, X`, `B, 1, X` and `B, 1`.
    B { ts6: bool, to: Option<Ref> },
    Q(Half),
    K(Place),
    L(Place),
    M(u32),
    N(u32),
    O(u32),
    P(u32),
    /// The three digits of TRA, the first written being digit 23.
    R(u32),
    /// The signals stimulated, as digits, and the given multiplier.
    S { signals: Vec<u32>, given: Option<u32> },
    T(u32),
    /// `X - Y`: one instruction if either is TS 6, otherwise two.
    Move(Place, Place),
    Clear(Vec<u32>),
    /// A minor cycle written out, or a number.
    Word(Word),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Instr {
    pub op: Op,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Label {
    pub table: Option<String>,
    pub fig: u32,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Line {
    Header { name: String, span: Span },
    Store { place: Place, value: Instr, span: Span },
    Code { label: Option<Label>, instrs: Vec<Instr>, next: Option<Label> },
}

/// What a line of text is made of. The language server colours by these.
#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
    Name(String),
    Int(String),
    Comma,
    Semi,
    Dot,
    Dash,
    Plus,
    /// A letter in quotation marks: the 'A' of ADD 'A'.
    Tick(String),
    Bad(char),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub tok: Tok,
    pub start: usize,
    pub end: usize,
}

pub const TYPES: [&str; 11] = ["B", "K", "L", "M", "N", "O", "P", "Q", "R", "S", "T"];
pub const LOGICAL: [(&str, u32, &str); 5] = [
    ("AND", 0b000, "TS 8 = TS 9 & TS 10"),
    ("OR", 0b001, "TS 8 = TS 9 v TS 10"),
    ("NOT", 0b010, "TS 8 = -TS 10"),
    ("IFF", 0b011, "TS 8 = (TS 9 & TS 10) v (-TS 9 & -TS 10)"),
    ("NEVER", 0b100, "TS 8 = 0"),
];
pub const ARITHMETICAL: [&str; 4] = ["ADD", "SUBTRACT", "MULTIPLY", "ROTATE"];
pub const STORAGES: [&str; 2] = ["TS", "DL"];

pub fn is_reserved(name: &str) -> bool {
    TYPES.contains(&name)
        || STORAGES.contains(&name)
        || ARITHMETICAL.contains(&name)
        || name == "CLEAR"
        || LOGICAL.iter().any(|l| l.0 == name)
        || sig::NAMES.iter().any(|s| s.0 == name)
}

/// A name a table may have: a letter, then letters and digits. The cards of
/// section 7 give it columns 1 to 8.
pub fn is_name(name: &str) -> bool {
    let mut chars = name.chars();
    chars.next().is_some_and(|c| c.is_ascii_uppercase())
        && chars.all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
        && !is_reserved(name)
}

/// The text of a line before its comment.
pub fn code_part(line: &str) -> &str {
    match line.find('#') {
        Some(i) => &line[..i],
        None => line,
    }
}

pub fn lex(line: &str) -> Vec<Token> {
    let chars: Vec<char> = code_part(line).chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let start = i;
        let tok = if c.is_whitespace() {
            i += 1;
            continue;
        } else if c.is_ascii_alphabetic() {
            while i < chars.len() && chars[i].is_ascii_alphanumeric() {
                i += 1;
            }
            Tok::Name(chars[start..i].iter().collect())
        } else if c.is_ascii_digit() {
            while i < chars.len() && chars[i].is_ascii_digit() {
                i += 1;
            }
            Tok::Int(chars[start..i].iter().collect())
        } else if "'‘’".contains(c) {
            let close = (i + 1..chars.len()).find(|j| "'‘’".contains(chars[*j]));
            match close {
                Some(j) if j > i + 1 && chars[i + 1..j].iter().all(|c| c.is_ascii_alphabetic()) => {
                    i = j + 1;
                    Tok::Tick(chars[start + 1..j].iter().collect())
                }
                _ => {
                    i += 1;
                    Tok::Bad(c)
                }
            }
        } else {
            i += 1;
            match c {
                ',' => Tok::Comma,
                ';' => Tok::Semi,
                '.' => Tok::Dot,
                '+' => Tok::Plus,
                '–' | '—' | '→' | '−' => Tok::Dash,
                '-' => {
                    if chars.get(i) == Some(&'>') {
                        i += 1;
                    }
                    Tok::Dash
                }
                c => Tok::Bad(c),
            }
        };
        out.push(Token { tok, start, end: i });
    }
    out
}

struct Reader<'a> {
    line: usize,
    toks: &'a [Token],
    at: usize,
    width: usize,
}

type Parse<T> = Result<T, Problem>;

impl Reader<'_> {
    fn peek(&self, ahead: usize) -> Option<&Tok> {
        self.toks.get(self.at + ahead).map(|t| &t.tok)
    }
    fn name(&self, ahead: usize) -> Option<&str> {
        match self.peek(ahead) {
            Some(Tok::Name(n)) => Some(n),
            _ => None,
        }
    }
    fn int(&self, ahead: usize) -> Option<&str> {
        match self.peek(ahead) {
            Some(Tok::Int(n)) => Some(n),
            _ => None,
        }
    }
    fn is(&self, ahead: usize, tok: Tok) -> bool {
        self.peek(ahead) == Some(&tok)
    }
    /// The span of the token `ahead`, or the end of the line.
    fn here(&self, ahead: usize) -> Span {
        match self.toks.get(self.at + ahead) {
            Some(t) => Span { line: self.line, start: t.start, end: t.end },
            None => {
                let end = self.toks.last().map(|t| t.end).unwrap_or(self.width);
                Span { line: self.line, start: end, end: end + 1 }
            }
        }
    }
    /// From the start of token `from` to the end of the last one taken.
    fn since(&self, from: usize) -> Span {
        let start = self.toks.get(from).map(|t| t.start).unwrap_or(0);
        let end = if self.at > 0 { self.toks[self.at - 1].end } else { start };
        Span { line: self.line, start, end: end.max(start) }
    }
    fn fail<T>(&self, ahead: usize, message: impl Into<String>) -> Parse<T> {
        Err(Problem { span: self.here(ahead), error: true, message: message.into() })
    }
    fn comma(&mut self, after: &str) -> Parse<()> {
        if self.is(0, Tok::Comma) {
            self.at += 1;
            Ok(())
        } else {
            self.fail(0, format!("a comma is wanted after {}", after))
        }
    }
    fn number(&mut self, what: &str, from: u32, to: u32) -> Parse<u32> {
        match self.int(0).and_then(|n| n.parse::<u32>().ok()) {
            Some(n) if (from..=to).contains(&n) => {
                self.at += 1;
                Ok(n)
            }
            Some(_) => self.fail(0, format!("{} is a number from {} to {}", what, from, to)),
            None => self.fail(0, format!("{} is wanted here, a number from {} to {}", what, from, to)),
        }
    }

    /// How many tokens of binary digits in groups of four begin here, and
    /// the digits. `0000,0100` is two groups.
    fn groups(&self) -> (usize, String) {
        let mut digits = String::new();
        let mut n = 0;
        loop {
            match self.int(n) {
                Some(g) if g.len() == 4 && g.chars().all(|c| c == '0' || c == '1') => digits.push_str(g),
                _ => break,
            }
            n += 1;
            if self.is(n, Tok::Comma) && matches!(self.int(n + 1), Some(g) if g.len() == 4) {
                n += 1;
            } else {
                break;
            }
        }
        (n, digits)
    }

    fn ts(&mut self) -> Parse<u32> {
        if self.name(0) != Some("TS") {
            return self.fail(0, "a temporary storage is wanted here, as TS 6");
        }
        self.at += 1;
        self.number("a temporary storage", 1, 32)
    }

    fn line_number(&mut self) -> Parse<u32> {
        if self.name(0) != Some("DL") {
            return self.fail(0, "a delay line is wanted here, as DL 10");
        }
        self.at += 1;
        self.number("a delay line", 0, 1023)
    }

    fn reference(&mut self) -> Parse<Ref> {
        let from = self.at;
        let table = match self.name(0) {
            // The report has a table ADD as well as an instruction ADD.
            Some(n) if is_name(n) || n == "ADD" => n.to_string(),
            Some(n) if n.chars().all(|c| c.is_ascii_alphanumeric()) && !is_reserved(n) => {
                return self.fail(0, format!("the name of a table is in capitals: {}", n.to_uppercase()));
            }
            _ => return self.fail(0, "the name of a table is wanted here"),
        };
        self.at += 1;
        let name = self.since(from);
        let fig = match self.int(0) {
            Some(_) => self.number("a detail figure", 1, 999)?,
            None => 1,
        };
        Ok(Ref { table, fig, name, span: self.since(from) })
    }

    fn place(&mut self) -> Parse<Place> {
        match self.name(0) {
            Some("TS") => Ok(Place::Ts(self.ts()?)),
            Some("DL") => {
                let m = self.line_number()?;
                self.comma("the delay line: DL 3,1 is its first minor cycle")?;
                Ok(Place::Dl(m, self.number("a minor cycle", 1, 32)?))
            }
            _ => Ok(Place::Tab(self.reference()?)),
        }
    }

    /// A position in the delay lines: not a temporary storage.
    fn minor_cycle(&mut self) -> Parse<Place> {
        if self.name(0) == Some("TS") {
            return self.fail(0, "a minor cycle of a delay line is wanted here, as DL 3,1");
        }
        self.place()
    }

    fn tick_a(&mut self) -> bool {
        if matches!(self.peek(0), Some(Tok::Tick(a)) if a == "A") {
            self.at += 1;
            true
        } else {
            false
        }
    }

    fn instr(&mut self) -> Parse<Instr> {
        let from = self.at;
        let op = self.op()?;
        Ok(Instr { op, span: self.since(from) })
    }

    fn op(&mut self) -> Parse<Op> {
        // A minor cycle written out in full, or a number with its sign.
        let (n, digits) = self.groups();
        if n > 0 {
            if digits.len() != 32 {
                return self.fail(0, format!("a minor cycle has 32 digits; here are {}", digits.len()));
            }
            self.at += n;
            return Ok(Op::Word(parse_bits(&digits, 1).unwrap()));
        }
        if self.is(0, Tok::Plus) || self.is(0, Tok::Dash) {
            let negative = self.is(0, Tok::Dash);
            let Some(v) = self.int(1).and_then(|n| n.parse::<i64>().ok()) else {
                return self.fail(1, "a number is wanted after the sign");
            };
            let v = if negative { -v } else { v };
            if v < i32::MIN as i64 || v > u32::MAX as i64 {
                return self.fail(1, "the number does not go into 32 binary digits");
            }
            self.at += 2;
            return Ok(Op::Word(v as u32));
        }
        let Some(word) = self.name(0).map(str::to_string) else {
            return self.fail(0, "an instruction is wanted here");
        };
        // A transfer begins with the place the material comes from.
        if word == "TS" || word == "DL" || !is_reserved(&word) {
            let from = self.place()?;
            if !self.is(0, Tok::Dash) {
                return self.fail(0, "a transfer is written with a dash: TS 6 - TS 2");
            }
            self.at += 1;
            return Ok(Op::Move(from, self.place()?));
        }
        self.at += 1;
        if let Some(l) = LOGICAL.iter().find(|l| l.0 == word) {
            return Ok(Op::R(l.1));
        }
        let s = |signals: &[u32], given| Ok(Op::S { signals: signals.to_vec(), given });
        match word.as_str() {
            "ADD" => match self.tick_a() {
                true => s(&[sig::POS, sig::CANCEL, sig::GIV], Some(1)),
                false => s(&[sig::POS, sig::GIV], Some(1)),
            },
            "SUBTRACT" => match self.tick_a() {
                true => s(&[sig::CANCEL, sig::GIV], Some(1)),
                false => s(&[sig::GIV], Some(1)),
            },
            "MULTIPLY" => match self.tick_a() {
                true => s(&[sig::POS, sig::CANCEL, sig::NOR, sig::AUTO], None),
                false => s(&[sig::POS, sig::NOR, sig::AUTO], None),
            },
            "ROTATE" => {
                // The number says how far, for the reader: the machine takes
                // the amount from TS 7.
                if self.int(0).is_some() && self.groups().0 == 0 {
                    self.number("the places to rotate", 0, 31)?;
                }
                s(&[sig::ROTATE], None)
            }
            "CLEAR" => {
                let mut list = vec![self.ts()?];
                while self.is(0, Tok::Comma) {
                    self.at += 1;
                    if self.name(0) == Some("TS") {
                        self.at += 1;
                    }
                    list.push(self.number("a temporary storage", 1, 32)?);
                }
                Ok(Op::Clear(list))
            }
            "B" => {
                self.comma("B")?;
                let ts6 = self.int(0) == Some("1");
                if ts6 {
                    self.at += 1;
                    if !self.is(0, Tok::Comma) {
                        return Ok(Op::B { ts6, to: None });
                    }
                    self.at += 1;
                }
                Ok(Op::B { ts6, to: Some(self.reference()?) })
            }
            "Q" => {
                self.comma("Q")?;
                let (n, digits) = self.groups();
                if n > 0 {
                    if digits.len() != 16 {
                        return self.fail(0, format!("Q gives digits 17 to 32, sixteen of them; here are {}", digits.len()));
                    }
                    self.at += n;
                    return Ok(Op::Q(Half::Bits(parse_bits(&digits, 17).unwrap())));
                }
                Ok(Op::Q(Half::Place(self.minor_cycle()?)))
            }
            "K" | "L" => {
                self.comma(&word)?;
                let place = self.minor_cycle()?;
                Ok(if word == "K" { Op::K(place) } else { Op::L(place) })
            }
            "M" | "N" => {
                self.comma(&word)?;
                let n = self.ts()?;
                Ok(if word == "M" { Op::M(n) } else { Op::N(n) })
            }
            "O" | "P" => {
                self.comma(&word)?;
                let n = self.line_number()?;
                Ok(if word == "O" { Op::O(n) } else { Op::P(n) })
            }
            "R" => {
                self.comma("R")?;
                match self.int(0) {
                    Some(d) if d.len() == 3 && d.chars().all(|c| c == '0' || c == '1') => {
                        let v = u32::from_str_radix(d, 2).unwrap();
                        self.at += 1;
                        Ok(Op::R(v))
                    }
                    _ => self.fail(0, "three binary digits are wanted, as R, 001 for OR"),
                }
            }
            "S" => {
                let mut signals = Vec::new();
                let mut given = None;
                while self.is(0, Tok::Comma) {
                    self.at += 1;
                    if let Some(name) = self.name(0) {
                        match sig::NAMES.iter().find(|s| s.0 == name) {
                            Some(s) => signals.push(s.1),
                            None => return self.fail(0, format!("{} is not a signal of type S", name)),
                        }
                        self.at += 1;
                    } else {
                        given = Some(self.number("a given multiplier", 0, 15)?);
                    }
                }
                Ok(Op::S { signals, given })
            }
            "T" => {
                self.comma("T")?;
                Ok(Op::T(self.number("a valve element", 0, 63)?))
            }
            _ => self.fail(0, format!("{} does not begin an instruction", word)),
        }
    }

    /// `8`, `8.`, `CALPOL 8` or `CALPOL 8.`, if that is what comes next.
    fn label(&mut self) -> Option<Label> {
        let from = self.at;
        let table = match self.name(0).map(str::to_string) {
            Some(n) if is_name(&n) && self.int(1).is_some() => {
                self.at += 1;
                Some(n)
            }
            _ => None,
        };
        let fig = self.int(0).and_then(|n| n.parse::<u32>().ok()).filter(|n| (1..=999).contains(n));
        match fig {
            Some(fig) => {
                self.at += 1;
                let span = self.since(from);
                if self.is(0, Tok::Dot) {
                    self.at += 1;
                }
                Some(Label { table, fig, span })
            }
            None => {
                self.at = from;
                None
            }
        }
    }
}

/// Reads one line. `None` is a line with nothing on it.
pub fn parse_line(line: usize, text: &str) -> Option<Result<Line, Problem>> {
    let toks = lex(text);
    if toks.is_empty() {
        return None;
    }
    let mut r = Reader { line, toks: &toks, at: 0, width: text.chars().count() };
    Some(read_line(&mut r))
}

fn read_line(r: &mut Reader) -> Parse<Line> {
    if let Some(t) = r.toks.iter().find(|t| matches!(t.tok, Tok::Bad(_))) {
        let Tok::Bad(c) = t.tok else { unreachable!() };
        return Err(Problem {
            span: Span { line: r.line, start: t.start, end: t.end },
            error: true,
            message: format!("`{}` is not used in an instruction table", c),
        });
    }
    // A name alone is the head of a table.
    let alone = r.toks.len() == 1 || (r.toks.len() == 2 && r.is(1, Tok::Dot));
    if let (true, Some(name)) = (alone, r.name(0)) {
        if is_name(name) || (name == "ADD" && r.toks.len() == 2) {
            return Ok(Line::Header { name: name.to_string(), span: r.here(0) });
        }
        if !is_reserved(name) {
            return r.fail(0, format!("the name of a table is in capitals: {}", name.to_uppercase()));
        }
        if TYPES.contains(&name) || STORAGES.contains(&name) {
            return r.fail(0, format!("{} is a word of the notation, so a table needs another name", name));
        }
    }
    // A storage and then something other than a dash gives it a content.
    let after = match (r.name(0), r.int(1)) {
        (Some("TS"), Some(_)) => Some(2),
        (Some("DL"), Some(_)) if r.is(2, Tok::Comma) && r.int(3).is_some() => Some(4),
        _ => None,
    };
    if let Some(n) = after {
        // A dash hard against a number is its sign, not a transfer.
        let signed = r.is(n, Tok::Dash) && r.int(n + 1).is_some() && r.toks[n].end == r.toks[n + 1].start;
        if r.peek(n).is_some() && (signed || !r.is(n, Tok::Dash)) {
            let place = r.place()?;
            if r.int(0).is_some() && r.groups().0 == 0 {
                // A number without a sign is allowed here, where no label can stand.
                let v = r.int(0).unwrap().parse::<u64>().ok().filter(|v| *v <= u32::MAX as u64);
                let Some(v) = v else {
                    return r.fail(0, "the number does not go into 32 binary digits");
                };
                let from = r.at;
                r.at += 1;
                let value = Instr { op: Op::Word(v as u32), span: r.since(from) };
                return finish(r, Line::Store { place, value, span: r.since(0) });
            }
            // A position alone: the digits 18-32 that name it.
            if r.name(0) == Some("DL") && r.int(1).is_some() && r.is(2, Tok::Comma) && r.int(3).is_some() && r.peek(4).is_none() {
                let from = r.at;
                let Place::Dl(m, k) = r.place()? else { unreachable!() };
                let value = Instr { op: Op::Word(crate::word::Pos { dl: m, slot: k - 1 }.word()), span: r.since(from) };
                return finish(r, Line::Store { place, value, span: r.since(0) });
            }
            let value = r.instr()?;
            if matches!(value.op, Op::Move(..) | Op::Clear(_)) {
                return Err(Problem {
                    span: value.span,
                    error: true,
                    message: "the content of a storage is one minor cycle: one instruction, or a number".into(),
                });
            }
            let span = r.since(0);
            return finish(r, Line::Store { place, value, span });
        }
    }
    // Otherwise instructions, with or without a number in front.
    let label = match (r.peek(0), r.peek(1)) {
        (Some(Tok::Int(_)), Some(Tok::Comma)) => None,
        (Some(Tok::Int(_)), _) => r.label(),
        (Some(Tok::Name(n)), Some(Tok::Int(_))) if is_name(n) && !r.is(2, Tok::Dash) => r.label(),
        _ => None,
    };
    let mut instrs = Vec::new();
    let mut next = None;
    while r.peek(0).is_some() {
        if r.is(0, Tok::Semi) || r.is(0, Tok::Dot) {
            r.at += 1;
            continue;
        }
        // After an instruction with no semicolon, a number is that of the next.
        if !instrs.is_empty() && !matches!(r.toks[r.at - 1].tok, Tok::Semi) {
            match r.label() {
                Some(l) if r.peek(0).is_none() => {
                    next = Some(l);
                    break;
                }
                _ => return r.fail(0, "a semicolon is wanted between instructions"),
            }
        }
        instrs.push(r.instr()?);
    }
    Ok(Line::Code { label, instrs, next })
}

fn finish(r: &mut Reader, line: Line) -> Parse<Line> {
    while r.is(0, Tok::Dot) || r.is(0, Tok::Semi) {
        r.at += 1;
    }
    match r.peek(0) {
        None => Ok(line),
        Some(_) => r.fail(0, "nothing more is wanted on this line"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn code(text: &str) -> (Option<Label>, Vec<Op>, Option<u32>) {
        match parse_line(0, text) {
            Some(Ok(Line::Code { label, instrs, next })) => {
                (label, instrs.into_iter().map(|i| i.op).collect(), next.map(|n| n.fig))
            }
            other => panic!("{}: {:?}", text, other),
        }
    }

    #[test]
    fn the_lines_of_indexin() {
        let (label, ops, next) = code("  1   Q, 0000,0100,0000,0000   2");
        assert_eq!(label.unwrap().fig, 1);
        assert_eq!(ops, vec![Op::Q(Half::Bits(crate::word::p(22)))]);
        assert_eq!(next, Some(2));
        let (_, ops, next) = code(" 4  ROTATE 16   5");
        assert_eq!(ops, vec![Op::S { signals: vec![sig::ROTATE], given: None }]);
        assert_eq!(next, Some(5));
        let (_, ops, _) = code("11  B,1, INDEXIN 11");
        assert!(matches!(&ops[0], Op::B { ts6: true, to: Some(r) } if r.table == "INDEXIN" && r.fig == 11));
        let (_, ops, _) = code("3 ADD ‘A’ 4");
        assert_eq!(ops, vec![Op::S { signals: vec![sig::POS, sig::CANCEL, sig::GIV], given: Some(1) }]);
    }

    #[test]
    fn the_lines_of_calpol() {
        let (label, ops, next) = code("CALPOL 1. Clear TS 22, 23; DL 1, 14 - TS 27;".replace("Clear", "CLEAR").as_str());
        assert_eq!(label.unwrap().table.as_deref(), Some("CALPOL"));
        assert_eq!(ops.len(), 2);
        assert_eq!(next, None);
        let (label, ops, next) = code("DL 1,15 - TS 29. CALPOL 8");
        assert!(label.is_none());
        assert_eq!(ops, vec![Op::Move(Place::Dl(1, 15), Place::Ts(29))]);
        assert_eq!(next, Some(8));
        let (_, ops, _) = code("TS 6 - TS 16; B, BURY; B, DISCRIM; B, 1");
        assert_eq!(ops.len(), 4);
        assert_eq!(ops[3], Op::B { ts6: true, to: None });
    }

    #[test]
    fn heads_and_contents() {
        assert!(matches!(parse_line(0, "INDEXIN"), Some(Ok(Line::Header { .. }))));
        assert!(matches!(parse_line(0, "CALPOL."), Some(Ok(Line::Header { .. }))));
        assert!(matches!(parse_line(0, "OR"), Some(Ok(Line::Code { .. }))));
        let store = parse_line(0, "DL 1,14  0000,0101,0000,0000,0100,0110,0000,0000");
        assert!(matches!(store, Some(Ok(Line::Store { place: Place::Dl(1, 14), .. }))));
        assert!(matches!(parse_line(0, "TS 13  -7"), Some(Ok(Line::Store { .. }))));
        assert!(matches!(parse_line(0, "TS 13  42"), Some(Ok(Line::Store { .. }))));
        assert!(matches!(parse_line(0, "DL 2,1  L, DL 3,1"), Some(Ok(Line::Store { .. }))));
        assert!(parse_line(0, "   # nothing").is_none());
        assert!(matches!(parse_line(0, "TS 6 - TS 2"), Some(Ok(Line::Code { .. }))));
        assert!(matches!(parse_line(0, "TS 6 = 2"), Some(Err(_))));
    }
}
