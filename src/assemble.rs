//! From popular form to machine form.
//!
//! Section 7 of the report gives the procedure, for cards and a collator:
//! take the tables a job uses, sort them by name and detail figure, number
//! them consecutively, and fill those numbers in wherever an instruction
//! names another. This does the same.
//!
//! Consecutive instructions are two minor cycles apart (section 11), so the
//! tables lie in the even minor cycles of the delay lines from `ORIGIN` up,
//! sixteen instructions to a line. The odd minor cycles there, and every
//! line below `ORIGIN`, are left for numbers.

use crate::machine::{sig, ts_of, Type, NO_INPUT};
use crate::notation::{parse_line, Half, Instr, Label, Line, Op, Place, Problem, Ref, Span, LOGICAL};
use crate::word::{bits, digit, field, half_bits, p, Pos, Word, P17};

/// The first delay line given to instruction tables. The examples in the
/// report keep numbers in DL 1, DL 3 and DL 10, so tables start well above.
pub const ORIGIN: u32 = 32;

/// Where the list of notes kept by BURY begins, and so what TS 31 holds at
/// the start.
pub const NOTES: Pos = Pos { dl: 31, slot: 0 };

/// One machine instruction before the numbers are filled in.
#[derive(Debug, Clone)]
enum Micro {
    B { ts6: bool, to: Option<Ref> },
    Q(Half),
    K(Place),
    L(Place),
    Op(Word),
}

#[derive(Debug, Clone)]
pub struct Slot {
    pub word: Word,
    pub pos: Pos,
    /// Its detail figure, counted on from the last one written.
    pub fig: u32,
    /// The instruction it came from, for a table in the text.
    pub span: Option<Span>,
}

#[derive(Debug, Clone)]
pub struct Table {
    pub name: String,
    /// The general description: the comment above the table.
    pub doc: String,
    /// From the library, not from the text.
    pub library: bool,
    pub header: Span,
    /// The last line of the table in the text.
    pub last_line: usize,
    pub slots: Vec<Slot>,
    /// Detail figures written in the text, and the slot each names.
    pub labels: Vec<(u32, usize)>,
    micros: Vec<(Micro, Option<Span>)>,
}

impl Table {
    fn fig_of(&self, slot: usize) -> u32 {
        match self.labels.iter().rev().find(|l| l.1 <= slot) {
            Some((fig, at)) => fig + (slot - at) as u32,
            None => 1 + slot as u32,
        }
    }

    /// The slot a detail figure names.
    pub fn slot_of(&self, fig: u32) -> Result<usize, String> {
        let (base, at) = self.labels.iter().rev().find(|l| l.0 <= fig).copied().unwrap_or((1, 0));
        let slot = at + (fig - base.min(fig)) as usize;
        let limit = self.labels.iter().find(|l| l.0 > fig).map(|l| l.1).unwrap_or(self.micros.len());
        if fig < base || slot >= limit {
            let last = if limit > at { base + (limit - at - 1) as u32 } else { base };
            return Err(format!(
                "there is no {} {}: the instructions from {} {} end at {} {}",
                self.name, fig, self.name, base, self.name, last
            ));
        }
        Ok(slot)
    }
}

#[derive(Debug, Clone, Default)]
pub struct Unit {
    /// In the order they lie in the delay lines: by name.
    pub tables: Vec<Table>,
    pub store: Vec<(Pos, Word, Span)>,
    pub ts: Vec<(usize, Word, Span)>,
    /// The first instruction of the first table in the text.
    pub entry: Option<Pos>,
    pub problems: Vec<Problem>,
    /// Every table named in the text, with where it is named.
    pub refs: Vec<Ref>,
    /// How the cards are to be shown, if the text says: `# cards: listing`.
    pub cards: Option<String>,
}

fn expand(op: &Op) -> Vec<Micro> {
    let m = |n: u32| Micro::Op(Type::M.word() | (n % 32) << 9);
    let n = |n: u32| Micro::Op(Type::N.word() | (n % 32) << 9);
    match op {
        Op::B { ts6, to } => vec![Micro::B { ts6: *ts6, to: to.clone() }],
        Op::Q(half) => vec![Micro::Q(half.clone())],
        Op::K(place) => vec![Micro::K(place.clone())],
        Op::L(place) => vec![Micro::L(place.clone())],
        Op::M(t) => vec![m(*t)],
        Op::N(t) => vec![n(*t)],
        Op::O(line) => vec![Micro::Op(Type::O.word() | line << 22)],
        Op::P(line) => vec![Micro::Op(Type::P.word() | line << 22)],
        Op::R(tra) => vec![Micro::Op(logical(*tra))],
        Op::S { signals, given } => {
            let mut w = Type::S.word();
            for s in signals {
                w |= p(*s);
            }
            vec![Micro::Op(w | given.unwrap_or(0) << (sig::GIVEN.0 - 1))]
        }
        Op::T(element) => vec![Micro::Op(Type::T.word() | element << 17)],
        Op::Word(w) => vec![Micro::Op(*w)],
        Op::Clear(list) => {
            let mut out = vec![Micro::Op(logical(0b100)), m(8)];
            out.extend(list.iter().map(|t| n(*t)));
            out
        }
        // "Each time we make a transfer we have to do it in two stages":
        // everything passes through TS 6.
        Op::Move(from, to) => match (from, to) {
            (Place::Ts(6), Place::Ts(b)) => vec![n(*b)],
            (Place::Ts(a), Place::Ts(6)) => vec![m(*a)],
            (Place::Ts(a), Place::Ts(b)) => vec![m(*a), n(*b)],
            (Place::Ts(6), cell) => vec![Micro::K(cell.clone())],
            (cell, Place::Ts(6)) => vec![Micro::L(cell.clone())],
            (Place::Ts(a), cell) => vec![m(*a), Micro::K(cell.clone())],
            (cell, Place::Ts(b)) => vec![Micro::L(cell.clone()), n(*b)],
            (a, b) => vec![Micro::L(a.clone()), Micro::K(b.clone())],
        },
    }
}

/// A logical operation: TRA in digits 23-25, the first digit written being
/// digit 23, and TRB 000 in digits 26-28.
pub fn logical(tra: u32) -> Word {
    Type::R.word() | (tra >> 2 & 1) << 22 | (tra >> 1 & 1) << 23 | (tra & 1) << 24
}

struct Builder<'a> {
    errata: &'a [&'a str],
    unit: Unit,
    stores: Vec<(Place, Instr, Span)>,
}

impl Builder<'_> {
    fn problem(&mut self, span: Span, error: bool, message: impl Into<String>) {
        self.unit.problems.push(Problem { span, error, message: message.into() });
    }

    /// Reads tables out of a text. Problems in a library table are reported
    /// where `blame` says the table was asked for.
    fn read(&mut self, text: &str, blame: Option<Span>) {
        let library = blame.is_some();
        let mut current: Option<usize> = None;
        let mut doc: Vec<String> = Vec::new();
        for (n, raw) in text.lines().enumerate() {
            let parsed = match parse_line(n, raw) {
                None => {
                    // Comments straight above a table are its general description.
                    match raw.trim().strip_prefix('#') {
                        Some(c) => doc.push(c.strip_prefix(' ').unwrap_or(c).to_string()),
                        None => doc.clear(),
                    }
                    if let Some(c) = raw.trim().strip_prefix("# cards:") {
                        if !library {
                            self.unit.cards = Some(c.trim().to_string());
                        }
                    }
                    continue;
                }
                Some(Err(mut problem)) => {
                    if let Some(span) = blame {
                        problem.message = format!("in the library: {}", problem.message);
                        problem.span = span;
                    }
                    self.unit.problems.push(problem);
                    doc.clear();
                    continue;
                }
                Some(Ok(line)) => line,
            };
            let own_doc = std::mem::take(&mut doc);
            match parsed {
                Line::Header { name, span } => {
                    if self.unit.tables.iter().any(|t| t.name == name) {
                        self.problem(blame.unwrap_or(span), true, format!("there are two tables called {}", name));
                        current = None;
                        continue;
                    }
                    if name.len() > 8 && !library {
                        self.problem(span, false, "the cards of section 7 give a name columns 1 to 8: eight letters");
                    }
                    self.unit.tables.push(Table {
                        name,
                        doc: own_doc.join("\n").trim().to_string(),
                        library,
                        header: span,
                        last_line: n,
                        slots: Vec::new(),
                        labels: Vec::new(),
                        micros: Vec::new(),
                    });
                    current = Some(self.unit.tables.len() - 1);
                }
                Line::Store { place, value, span } => {
                    self.note_refs(&value, library);
                    self.stores.push((place, value, blame.unwrap_or(span)));
                }
                Line::Code { label, instrs, next } => {
                    let Some(at) = current else {
                        let span = instrs.first().map(|i| i.span).or(label.as_ref().map(|l| l.span)).unwrap();
                        self.problem(blame.unwrap_or(span), true, "an instruction belongs to a table: give the table's name on a line of its own first");
                        continue;
                    };
                    for i in &instrs {
                        self.note_refs(i, library);
                    }
                    self.code(at, label, instrs, next, n, blame, raw);
                }
            }
        }
    }

    fn note_refs(&mut self, instr: &Instr, library: bool) {
        let mut note = |place: &Place| {
            if let Place::Tab(r) = place {
                let mut r = r.clone();
                if library {
                    r.span = Span::default();
                    r.name = Span::default();
                }
                self.unit.refs.push(r);
            }
        };
        match &instr.op {
            Op::B { to: Some(r), .. } => note(&Place::Tab(r.clone())),
            Op::Q(Half::Place(place)) | Op::K(place) | Op::L(place) => note(place),
            Op::Move(a, b) => {
                note(a);
                note(b);
            }
            _ => {}
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn code(&mut self, at: usize, label: Option<Label>, instrs: Vec<Instr>, next: Option<Label>, line: usize, blame: Option<Span>, raw: &str) {
        let name = self.unit.tables[at].name.clone();
        self.unit.tables[at].last_line = line;
        if let Some(label) = label {
            let table = &self.unit.tables[at];
            let here = table.micros.len();
            let expected = if here == 0 && table.labels.is_empty() { label.fig } else { table.fig_of(here) };
            let last = table.labels.last().map(|l| l.0).unwrap_or(0);
            let span = blame.unwrap_or(label.span);
            if label.table.as_deref().is_some_and(|t| t != name) {
                self.problem(span, true, format!("this is the table {}, not {}", name, label.table.unwrap()));
            } else if label.fig <= last {
                self.problem(span, true, format!("detail figures go upwards: {} {} comes after {} {}", name, label.fig, name, last));
            } else {
                if label.fig < expected && blame.is_none() {
                    let since = table.labels.last().map(|l| l.0).unwrap_or(1);
                    self.problem(
                        span,
                        false,
                        format!(
                            "counting on from {} {}, this instruction is {} {}, not {} {}",
                            name, since, name, expected, name, label.fig
                        ),
                    );
                }
                self.unit.tables[at].labels.push((label.fig, here));
            }
        }
        let mut last_is_b = false;
        for instr in &instrs {
            self.lint(instr, blame);
            self.lint_rotate(instr, blame, raw);
            for micro in expand(&instr.op) {
                last_is_b = matches!(micro, Micro::B { .. });
                self.unit.tables[at].micros.push((micro, blame.is_none().then_some(instr.span)));
            }
        }
        if let (Some(next), false, None) = (next, last_is_b || instrs.is_empty(), blame) {
            let table = &self.unit.tables[at];
            let expected = table.fig_of(table.micros.len());
            if next.table.as_deref().is_some_and(|t| t != name) {
                self.problem(next.span, true, format!("an instruction of type A goes on to the next of its own table, {} {}", name, expected));
            } else if next.fig != expected {
                self.problem(next.span, false, format!("the instruction after this one is {} {}, not {} {}", name, expected, name, next.fig));
            }
        }
    }

    /// `ROTATE n`: the machine rotates by TS 7, which holds 16 unless the
    /// job changes it. A different n is a hope, not an instruction.
    fn lint_rotate(&mut self, instr: &Instr, blame: Option<Span>, raw: &str) {
        if blame.is_some() || !matches!(&instr.op, Op::S { signals, .. } if signals == &[sig::ROTATE]) {
            return;
        }
        let text: String = raw.chars().skip(instr.span.start).take(instr.span.end - instr.span.start).collect();
        let n = text.trim_start_matches("ROTATE").trim();
        if !n.is_empty() && n != "16" {
            self.problem(
                instr.span,
                false,
                format!("the machine rotates by the number in TS 7, which is 16 unless this job puts {} there: the {} is not part of the instruction", n, n),
            );
        }
    }

    /// What an instruction will do that its writer probably did not mean.
    fn lint(&mut self, instr: &Instr, blame: Option<Span>) {
        if blame.is_some() {
            return;
        }
        let dead = |t: &u32| NO_INPUT.contains(&(*t as usize));
        let message = match &instr.op {
            Op::N(t) | Op::Move(_, Place::Ts(t)) if dead(t) => Some(format!(
                "TS {} takes nothing from TS 6: TS 1 is filled by type B, TS 4 and TS 5 by the arithmetic, TS 8 by the logical operations",
                t
            )),
            Op::Clear(list) if list.iter().any(dead) => {
                Some("TS 1, TS 4, TS 5 and TS 8 take nothing from TS 6, so CLEAR does not reach them".to_string())
            }
            Op::Move(a, b) if a == b => Some("this transfers a storage to itself".to_string()),
            Op::S { signals, given } => {
                let has = |s| signals.contains(&s);
                if given.is_some() && !has(sig::GIV) {
                    Some("a multiplier is given, but without GIV the machine does not look at it".to_string())
                } else if has(sig::GIV) && given.is_none() {
                    Some("GIV takes the multiplier from the instruction: write it last, as S, POS, GIV, 6".to_string())
                } else if !has(sig::ROTATE) && !has(sig::CANCEL) && !has(sig::GIV) && !has(sig::NOR) && !has(sig::ERR) && !has(sig::AUTO) {
                    Some("with no multiplier (GIV, NOR, ERR or AUTO) nothing is added".to_string())
                } else {
                    None
                }
            }
            _ => None,
        };
        if let Some(message) = message {
            self.problem(instr.span, false, message);
        }
    }

    fn find(&self, r: &Ref) -> Result<Pos, String> {
        let table = self.unit.tables.iter().find(|t| t.name == r.table).ok_or_else(|| {
            format!("there is no table {} here or in the library", r.table)
        })?;
        Ok(table.slots[table.slot_of(r.fig)?].pos)
    }

    fn position(&self, place: &Place) -> Result<Pos, String> {
        match place {
            Place::Dl(m, n) => Ok(Pos { dl: *m, slot: n - 1 }),
            Place::Tab(r) => self.find(r),
            Place::Ts(_) => Err("a temporary storage has no position in the delay lines".into()),
        }
    }

    fn encode(&self, micro: &Micro, own: Pos) -> Result<Word, String> {
        Ok(match micro {
            Micro::Op(w) => *w,
            Micro::B { ts6, to } => {
                let to = match to {
                    Some(r) => self.find(r)?,
                    None => own,
                };
                p(2) | p(3) | if *ts6 { P17 } else { 0 } | to.word()
            }
            Micro::Q(Half::Bits(w)) => Type::Q.word() | w,
            Micro::Q(Half::Place(place)) => Type::Q.word() | self.position(place)?.word(),
            Micro::K(place) => Type::K.word() | self.position(place)?.word(),
            Micro::L(place) => Type::L.word() | self.position(place)?.word(),
        })
    }
}

/// Reads a text and everything it uses from the library, and places it.
pub fn assemble(text: &str, errata: &[&str]) -> Unit {
    let mut b = Builder { errata, unit: Unit::default(), stores: Vec::new() };
    b.read(text, None);
    let first = b.unit.tables.first().map(|t| t.name.clone());

    // Fetch from the library whatever is named and not here.
    loop {
        let missing = b.unit.refs.iter().find(|r| !b.unit.tables.iter().any(|t| t.name == r.table)).cloned();
        let Some(r) = missing else { break };
        match crate::library(&r.table, b.errata) {
            Some(source) => {
                let before = b.unit.tables.len();
                b.read(&source, Some(r.span));
                if b.unit.tables.len() == before {
                    break;
                }
            }
            None => break,
        }
    }

    // Sort by name, and number consecutively.
    b.unit.tables.sort_by(|x, y| x.name.cmp(&y.name));
    let mut at = Pos { dl: ORIGIN, slot: 0 };
    for table in &mut b.unit.tables {
        for i in 0..table.micros.len() {
            table.slots.push(Slot { word: 0, pos: at, fig: table.fig_of(i), span: table.micros[i].1 });
            at = at.after(1);
        }
    }
    let end = at;

    // Fill in the numbers.
    for t in 0..b.unit.tables.len() {
        for i in 0..b.unit.tables[t].micros.len() {
            let (micro, span) = b.unit.tables[t].micros[i].clone();
            match b.encode(&micro, b.unit.tables[t].slots[i].pos) {
                Ok(w) => b.unit.tables[t].slots[i].word = w,
                Err(message) => b.problem(span.unwrap_or(b.unit.tables[t].header), true, message),
            }
        }
    }
    b.unit.entry = first.and_then(|n| b.unit.tables.iter().find(|t| t.name == n)).and_then(|t| t.slots.first()).map(|s| s.pos);

    // Contents given to storages.
    for (place, value, span) in std::mem::take(&mut b.stores) {
        let word = match expand(&value.op).as_slice() {
            [micro] => match &place {
                Place::Ts(_) => b.encode(micro, Pos { dl: 0, slot: 0 }),
                place => b.position(place).and_then(|own| b.encode(micro, own)),
            },
            _ => Err("the content of a storage is one minor cycle".to_string()),
        };
        let word = match word {
            Ok(w) => w,
            Err(message) => {
                b.problem(value.span, true, message);
                continue;
            }
        };
        match &place {
            Place::Ts(n) => b.unit.ts.push((*n as usize, word, span)),
            place => match b.position(place) {
                Ok(pos) => {
                    let in_tables = matches!(place, Place::Dl(..)) && pos.dl >= ORIGIN && pos.slot % 2 == 0 && pos.number() < end.number();
                    if in_tables {
                        b.problem(span, true, format!("{} holds an instruction of a table: the tables lie in the even minor cycles from DL {} up", pos, ORIGIN));
                    } else if pos.dl as usize >= crate::machine::LINES {
                        b.problem(span, true, format!("the machine has {} delay lines, DL 0 to DL {}", crate::machine::LINES, crate::machine::LINES - 1));
                    } else {
                        b.unit.store.push((pos, word, span));
                    }
                }
                Err(message) => b.problem(span, true, message),
            },
        }
    }
    if b.unit.tables.iter().all(|t| t.library) && !b.unit.problems.iter().any(|p| p.error) {
        b.problem(Span::default(), true, "there is no instruction table here");
    }
    b.unit.problems.sort_by_key(|p| (p.span.line, p.span.start));
    b.unit
}

impl Unit {
    pub fn errors(&self) -> Vec<&Problem> {
        self.problems.iter().filter(|p| p.error).collect()
    }

    pub fn table(&self, name: &str) -> Option<&Table> {
        self.tables.iter().find(|t| t.name == name)
    }

    /// The popular name of a position: INDEXIN 11.
    pub fn name_of(&self, pos: Pos) -> Option<String> {
        self.tables.iter().find_map(|t| t.slots.iter().find(|s| s.pos == pos).map(|s| format!("{} {}", t.name, s.fig)))
    }

    /// The instruction in the text at a line and column, with its machine
    /// forms: one, or two for a transfer in two stages.
    pub fn at(&self, line: usize, column: usize) -> Option<(Span, Vec<&Slot>)> {
        let covers = |s: &Span| s.line == line && s.start <= column && column <= s.end;
        let slots: Vec<&Slot> = self.tables.iter().flat_map(|t| &t.slots).filter(|s| s.span.as_ref().is_some_and(covers)).collect();
        slots.first().and_then(|s| s.span).map(|span| (span, slots))
    }

    /// A word in popular form, with the names this job gives to positions.
    pub fn popular(&self, w: Word) -> String {
        popular(w, &|pos| self.name_of(pos))
    }
}

/// A minor cycle read as an instruction, in popular form.
pub fn popular(w: Word, name: &dyn Fn(Pos) -> Option<String>) -> String {
    let place = |w: Word| name(Pos::of(w)).unwrap_or_else(|| Pos::of(w).to_string());
    if digit(w, 3) {
        return format!("B, {}{}", if w & P17 != 0 { "1, " } else { "" }, place(w));
    }
    let ts = ts_of(field(w, 10, 14));
    match Type::of(w) {
        None => bits(w),
        Some(Type::K) => format!("TS 6 - {}", place(w)),
        Some(Type::L) => format!("{} - TS 6", place(w)),
        Some(Type::M) => format!("TS {} - TS 6", ts),
        Some(Type::N) => format!("TS 6 - TS {}", ts),
        Some(Type::O) => format!("O, DL {}", field(w, 23, 32)),
        Some(Type::P) => format!("P, DL {}", field(w, 23, 32)),
        Some(Type::Q) => format!("Q, {}", half_bits(w)),
        Some(Type::T) => format!("T, {}", field(w, 18, 23)),
        Some(Type::R) => {
            let tra = (digit(w, 23) as u32) << 2 | (digit(w, 24) as u32) << 1 | digit(w, 25) as u32;
            match LOGICAL.iter().find(|l| l.1 == tra && field(w, 26, 28) == 0) {
                Some(l) => l.0.to_string(),
                None => format!("R, {:03b}", tra),
            }
        }
        Some(Type::S) => {
            let on: Vec<u32> = sig::NAMES.iter().map(|s| s.1).filter(|k| digit(w, *k)).collect();
            let given = field(w, sig::GIVEN.0, sig::GIVEN.1);
            let is = |list: &[u32], g: u32| on == list && given == g;
            if is(&[sig::POS, sig::GIV], 1) {
                "ADD".into()
            } else if is(&[sig::POS, sig::CANCEL, sig::GIV], 1) {
                "ADD 'A'".into()
            } else if is(&[sig::GIV], 1) {
                "SUBTRACT".into()
            } else if is(&[sig::CANCEL, sig::GIV], 1) {
                "SUBTRACT 'A'".into()
            } else if is(&[sig::POS, sig::NOR, sig::AUTO], 0) {
                "MULTIPLY".into()
            } else if is(&[sig::POS, sig::CANCEL, sig::NOR, sig::AUTO], 0) {
                "MULTIPLY 'A'".into()
            } else if is(&[sig::ROTATE], 0) {
                "ROTATE".into()
            } else {
                let mut s = String::from("S");
                for (n, k) in sig::NAMES {
                    if digit(w, k) {
                        s.push_str(", ");
                        s.push_str(n);
                    }
                }
                if digit(w, sig::GIV) || given != 0 {
                    s.push_str(&format!(", {}", given));
                }
                s
            }
        }
    }
}
