//! What an editor wants to know about a text: its problems, what each word
//! of it is, what an instruction becomes on the machine, and where a table
//! begins and is used. `ace check`, `ace format` and `ace lsp` all rest here.

use crate::assemble::{assemble, Unit};
use crate::machine::{sig, ts_of, Type};
use crate::notation::{
    code_part, is_name, lex, parse_line, Label, Line, Problem, Span, Tok, Token, LOGICAL, STORAGES,
};
use crate::word::{bits, digit, field, Pos, Word, P17};
use crate::{ERRATA, LIBRARY};

const ALL: &[&str] = &["all"];

pub fn check(text: &str) -> Vec<Problem> {
    assemble(text, ALL).problems
}

/// (line, start, end), lines from 0.
pub type Place = (usize, usize, usize);

fn place(s: Span) -> Place {
    (s.line, s.start, s.end)
}

pub const TOKEN_TYPES: [&str; 9] =
    ["comment", "keyword", "function", "variable", "number", "operator", "string", "macro", "property"];

/// What kind of word each token of a line is, as an index into TOKEN_TYPES.
fn kinds(toks: &[Token]) -> Vec<usize> {
    let mut out = Vec::with_capacity(toks.len());
    let name = |i: usize| match toks.get(i).map(|t| &t.tok) {
        Some(Tok::Name(n)) => Some(n.as_str()),
        _ => None,
    };
    // A number that opens the line, with no comma after it, is a detail figure.
    let label = match (toks.first().map(|t| &t.tok), toks.get(1).map(|t| &t.tok)) {
        (Some(Tok::Int(_)), Some(Tok::Comma)) => None,
        (Some(Tok::Int(_)), _) => Some(0),
        (Some(Tok::Name(n)), Some(Tok::Int(_))) if is_name(n) && !matches!(toks.get(2).map(|t| &t.tok), Some(Tok::Dash)) => {
            Some(1)
        }
        _ => None,
    };
    for (i, t) in toks.iter().enumerate() {
        out.push(match &t.tok {
            Tok::Name(n) if STORAGES.contains(&n.as_str()) => 3,
            Tok::Name(n) if sig::NAMES.iter().any(|s| s.0 == n) => 7,
            Tok::Name(n) if is_name(n) => 2,
            Tok::Name(_) => 1,
            Tok::Int(_) if label == Some(i) => 8,
            // The numbers of TS 6 and DL 1,14 belong to the storage.
            Tok::Int(_) if matches!(name(i.wrapping_sub(1)), Some("TS" | "DL")) => 3,
            Tok::Int(_) if i >= 3 && name(i - 3) == Some("DL") && toks[i - 1].tok == Tok::Comma => 3,
            Tok::Int(_) => 4,
            Tok::Tick(_) => 6,
            Tok::Dash | Tok::Plus => 5,
            _ => usize::MAX,
        });
    }
    out
}

/// Every token to colour: (line, start, length, kind).
pub fn semantic(text: &str) -> Vec<(usize, usize, usize, usize)> {
    let mut out = Vec::new();
    for (n, line) in text.lines().enumerate() {
        let toks = lex(line);
        for (t, kind) in toks.iter().zip(kinds(&toks)) {
            if kind != usize::MAX {
                out.push((n, t.start, t.end - t.start, kind));
            }
        }
        let code = code_part(line).chars().count();
        let all = line.chars().count();
        if all > code {
            out.push((n, code, all - code, 0));
        }
    }
    out
}

/// The tables of a text: name, first line, last line.
pub fn tables(text: &str) -> Vec<(String, usize, usize)> {
    let unit = assemble(text, ALL);
    let mut list: Vec<_> =
        unit.tables.iter().filter(|t| !t.library).map(|t| (t.name.clone(), t.header.line, t.last_line)).collect();
    list.sort_by_key(|t| t.1);
    list
}

/// Every place a table's name is written: its head, the instructions that
/// name it, and the detail figures that carry it.
fn occurrences(text: &str, name: &str) -> Vec<Place> {
    let mut out = Vec::new();
    for (n, line) in text.lines().enumerate() {
        for t in lex(line) {
            if t.tok == Tok::Name(name.to_string()) {
                out.push((n, t.start, t.end));
            }
        }
    }
    out
}

fn name_at(text: &str, line: usize, column: usize) -> Option<(String, Place)> {
    let toks = lex(text.lines().nth(line)?);
    toks.iter().find_map(|t| match &t.tok {
        Tok::Name(n) if is_name(n) && t.start <= column && column <= t.end => Some((n.clone(), (line, t.start, t.end))),
        _ => None,
    })
}

pub fn references(text: &str, line: usize, column: usize) -> Vec<Place> {
    match name_at(text, line, column) {
        Some((name, _)) => occurrences(text, &name),
        None => Vec::new(),
    }
}

/// Where the table named under the pointer begins, or the instruction a
/// detail figure names.
pub fn definition(text: &str, line: usize, column: usize) -> Option<Place> {
    let (name, at) = name_at(text, line, column)?;
    let unit = assemble(text, ALL);
    let table = unit.table(&name).filter(|t| !t.library)?;
    // INDEXIN 11 goes to instruction 11, if that is what is written.
    let toks = lex(text.lines().nth(line)?);
    let fig = toks.iter().position(|t| t.start == at.1).and_then(|i| match toks.get(i + 1).map(|t| &t.tok) {
        Some(Tok::Int(n)) => n.parse::<u32>().ok(),
        _ => None,
    });
    let slot = fig.and_then(|f| table.slot_of(f).ok()).and_then(|s| table.slots[s].span);
    Some(slot.map(place).unwrap_or(place(table.header)))
}

/// The special duties of the temporary storages (section 12), and the uses
/// the report's tables make of the rest.
pub fn storage(n: u32) -> &'static str {
    match n {
        1 => "carries the retiring data: the position left by the last instruction of type B. Type N cannot fill it.",
        2 => "the argument of the arithmetical operations: what is added into the partial sum.",
        3 => "the multiplier of a long multiplication.",
        4 => "the less significant half of the partial sum. Filled only by the arithmetic.",
        5 => "the more significant half of the partial sum. Filled only by the arithmetic.",
        6 => "the shunting station: every transfer passes through it, and an instruction can be obeyed from it.",
        7 => "the number of places ROTATE turns the partial sum.",
        8 => "the result of the logical operations. Filled only by them.",
        9 => "the first input of the logical operations.",
        10 => "the second input of the logical operations, and the one NOT works on.",
        11 => "used in error calculations: a source of multipliers (ERR).",
        12 => "the timing of automatic multiplication, and the choice of unusual digits of the multiplier (DIFF).",
        13 | 14 => "the argument of CALPOL; BINDEC translates TS 13.",
        15 => "DISCRIM passes this into TS 24 if TS 8 contains any digit 1.",
        16 => "DISCRIM passes this into TS 24 if TS 8 contains no digit 1.",
        18 | 19 => "the first number MULTIP and ADD take.",
        20 | 21 => "the second number MULTIP and ADD take; TRANS 45 fills them from TS 22 and TS 23.",
        22 | 23 => "the result of MULTIP and ADD.",
        24 => "where DISCRIM puts the one it chose of TS 15 and TS 16.",
        25 | 26 => "the result of CALPOL.",
        27 => "the position INDEXIN fetches from, in digits 17-32; PLUSIND adds 1 to it.",
        28 => "where INDEXIN puts the minor cycle it fetched.",
        31 => "the position of the next note BURY will make.",
        _ => "a temporary storage with no special duty.",
    }
}

/// What an instruction does, in a sentence.
pub fn english(w: Word, unit: &Unit) -> String {
    let at = |w: Word| {
        let pos = Pos::of(w);
        match unit.name_of(pos) {
            Some(name) => format!("{} ({})", name, pos),
            None => pos.to_string(),
        }
    };
    if digit(w, 3) {
        return if w & P17 != 0 {
            format!("Type B, with a 1 in column 17: carry out the instruction in TS 6. If that is of type A, go on at the instruction after {}.", at(w))
        } else {
            format!("Type B: proceed with instruction {}. The position left goes into TS 1.", at(w))
        };
    }
    let ts = ts_of(field(w, 10, 14));
    match Type::of(w) {
        None => "Digits 5-8 are not the code of any type of operation.".into(),
        Some(Type::K) => format!("Type K: pass the content of TS 6 into {}.", at(w)),
        Some(Type::L) => format!("Type L: pass the content of {} into TS 6.", at(w)),
        Some(Type::M) => format!("Type M: pass the content of TS {} into TS 6.", ts),
        Some(Type::N) => format!("Type N: pass the content of TS 6 into TS {}.", ts),
        Some(Type::O) => format!("Type O: pass the first 12 minor cycles of DL {} out onto a card.", field(w, 23, 32)),
        Some(Type::P) => format!("Type P: pass the card in the reader on to the first 12 minor cycles of DL {}.", field(w, 23, 32)),
        Some(Type::Q) => "Type Q: pass digits 17-32 of this instruction into TS 6. Digits 1-16 of TS 6 stay as they were.".into(),
        Some(Type::T) => format!("Type T: stimulate the valve element FLEX {}.", field(w, 18, 23)),
        Some(Type::R) => {
            let tra = (digit(w, 23) as u32) << 2 | (digit(w, 24) as u32) << 1 | digit(w, 25) as u32;
            match LOGICAL.iter().find(|l| l.1 == tra && field(w, 26, 28) == 0) {
                Some(l) => format!("Type R, a logical operation digit by digit: {}.", l.2),
                None => "Type R with digits that choose no logical operation: TS 8 = 0.".into(),
            }
        }
        Some(Type::S) => {
            let on = |k| digit(w, k);
            let mut s = String::from("Type S: ");
            if on(sig::CANCEL) {
                s.push_str("cancel the partial sum (TS 4 and TS 5), then ");
            }
            let what = if on(sig::SELF) { "the partial sum itself" } else { "TS 2" };
            if on(sig::AUTO) {
                s.push_str(&format!("{} {} multiplied by TS 3", if on(sig::POS) { "add" } else { "subtract" }, what));
            } else if on(sig::GIV) {
                let times = match field(w, sig::GIVEN.0, sig::GIVEN.1) {
                    1 => String::new(),
                    n => format!("{} times ", n),
                };
                s.push_str(&format!("{} {}{}", if on(sig::POS) { "add" } else { "subtract" }, times, what));
            } else if on(sig::NOR) {
                s.push_str(&format!("{} {} multiplied by digits 1-4 of TS 3", if on(sig::POS) { "add" } else { "subtract" }, what));
            } else if on(sig::ERR) {
                s.push_str(&format!("{} {} multiplied by four digits of TS 11", if on(sig::POS) { "add" } else { "subtract" }, what));
            } else if on(sig::ROTATE) {
                s = "Type S: ".into();
            } else {
                s.push_str("add nothing");
            }
            if on(sig::ROTATE) {
                if !s.ends_with(": ") {
                    s.push_str(", then ");
                }
                s.push_str("rotate the partial sum (TS 4 and TS 5) by the number of places in TS 7, towards the more significant end");
            }
            s.push('.');
            s
        }
    }
}

/// How long an instruction takes once it is in CI, in minor cycles, where
/// that does not depend on where a word happens to be.
fn duration(w: Word) -> &'static str {
    if digit(w, 3) {
        return "No time in CA: the next instruction is looked for at once.";
    }
    match Type::of(w) {
        Some(Type::K | Type::L) => "Waits for the minor cycle to come round: up to 32 minor cycles, about a millisecond.",
        Some(Type::O | Type::P) => "Takes as long as the card does to pass.",
        Some(Type::S) if digit(w, sig::AUTO) => "16 minor cycles: the addition done eight times, four digits of the multiplier at a time.",
        Some(Type::S) => "2 minor cycles, the partial sum being 64 digits long. The next instruction has then just gone by, and is waited for.",
        Some(_) => "1 minor cycle: the next instruction arrives just as it ends.",
        None => "",
    }
}

fn table_hover(unit: &Unit, name: &str) -> Option<String> {
    let table = unit.table(name)?;
    let whose = match LIBRARY.iter().find(|e| e.name == name) {
        Some(e) if table.library && e.printed => "From the library: a table the report prints.",
        Some(_) if table.library => "From the library: the report describes it, the table is ours.",
        _ => "A table of this text.",
    };
    let mut s = format!("**{}**: {} minor cycles, from {}. {}", name, table.slots.len(), table.slots.first()?.pos, whose);
    if !table.doc.is_empty() {
        s.push_str("\n\n");
        s.push_str(&table.doc.replace('\n', "  \n"));
    }
    if table.library {
        for e in ERRATA.iter().filter(|e| e.table == name) {
            s.push_str(&format!("\n\nCorrected (`{}`): {}.", e.id, e.title));
        }
    }
    Some(s)
}

/// What to show when the pointer rests at a line and column.
pub fn hover(text: &str, line: usize, column: usize) -> Option<(String, Place)> {
    let unit = assemble(text, ALL);
    let source = text.lines().nth(line)?;
    let toks = lex(source);
    let i = toks.iter().position(|t| t.start <= column && column < t.end.max(t.start + 1))?;
    let t = &toks[i];
    // The name of a table.
    if let Tok::Name(n) = &t.tok {
        if is_name(n) {
            let on_header = matches!(parse_line(line, source), Some(Ok(Line::Header { .. })));
            let is_ref = unit.at(line, column).is_some();
            if on_header || is_ref || unit.table(n).is_some() {
                if let Some(doc) = table_hover(&unit, n) {
                    // On a B, say where it goes as well.
                    let mut doc = doc;
                    if let Some((_, slots)) = unit.at(line, column) {
                        doc = format!("{}\n\n---\n\n{}", machine_forms(&unit, &slots), doc);
                    }
                    return Some((doc, (line, t.start, t.end)));
                }
            }
        }
    }
    // A temporary storage.
    let ts = match (&t.tok, toks.get(i + 1).map(|t| &t.tok), i.checked_sub(1).map(|j| &toks[j].tok)) {
        (Tok::Name(n), Some(Tok::Int(v)), _) if n == "TS" => Some((v, t.start, toks[i + 1].end)),
        (Tok::Int(v), _, Some(Tok::Name(n))) if n == "TS" => Some((v, toks[i - 1].start, t.end)),
        _ => None,
    };
    if let Some((v, start, end)) = ts {
        if let Ok(n) = v.parse::<u32>() {
            if (1..=32).contains(&n) {
                let mut s = format!("**TS {}**: {}", n, storage(n));
                if let Some((_, slots)) = unit.at(line, column) {
                    s = format!("{}\n\n---\n\n{}", s, machine_forms(&unit, &slots));
                }
                return Some((s, (line, start, end)));
            }
        }
    }
    let (span, slots) = unit.at(line, column)?;
    Some((machine_forms(&unit, &slots), place(span)))
}

fn machine_forms(unit: &Unit, slots: &[&crate::assemble::Slot]) -> String {
    let mut s = String::new();
    if slots.len() > 1 {
        s.push_str("A transfer in two stages, through TS 6.\n\n");
    }
    s.push_str("On the machine:\n\n```\n");
    for slot in slots {
        s.push_str(&format!("{:<10} {}   {}\n", slot.pos.to_string(), bits(slot.word), unit.popular(slot.word)));
    }
    s.push_str("```\n");
    for slot in slots {
        s.push_str(&format!("\n{}", english(slot.word, unit)));
        let d = duration(slot.word);
        if !d.is_empty() {
            s.push_str(&format!("  \n{}", d));
        }
        s.push('\n');
    }
    s
}

/// Words to offer: (label, kind, detail). Kind is the protocol's number:
/// 14 keyword, 3 function, 6 variable.
pub fn completions(text: &str) -> Vec<(String, usize, String)> {
    let mut out: Vec<(String, usize, String)> = vec![
        ("B, ".into(), 14, "Type B: proceed with another instruction".into()),
        ("B, 1".into(), 14, "Type B with a 1 in column 17: carry out the instruction in TS 6".into()),
        ("Q, ".into(), 14, "Type Q: pass digits 17-32 into TS 6".into()),
        ("O, DL ".into(), 14, "Type O: punch the first 12 minor cycles of a delay line".into()),
        ("P, DL ".into(), 14, "Type P: read a card into a delay line".into()),
        ("T, ".into(), 14, "Type T: stimulate a valve element".into()),
        ("S, ".into(), 14, "Type S, by its signals: S, POS, CANCEL, GIV, 6".into()),
        ("TS ".into(), 6, "a temporary storage, TS 1 to TS 32".into()),
        ("DL ".into(), 6, "a minor cycle of a delay line: DL 3,1".into()),
        ("ADD".into(), 14, "add TS 2 into the partial sum".into()),
        ("ADD 'A'".into(), 14, "add TS 2 into TS 4 cancelling the partial sums".into()),
        ("SUBTRACT".into(), 14, "subtract TS 2 from the partial sum".into()),
        ("SUBTRACT 'A'".into(), 14, "subtract TS 2 from zero".into()),
        ("MULTIPLY".into(), 14, "add TS 2 times TS 3 into the partial sum".into()),
        ("MULTIPLY 'A'".into(), 14, "TS 2 times TS 3, cancelling the partial sums".into()),
        ("ROTATE".into(), 14, "rotate the partial sum by the places in TS 7".into()),
        ("CLEAR TS ".into(), 14, "put zero into temporary storages".into()),
    ];
    for (name, _, meaning) in LOGICAL {
        out.push((name.to_string(), 14, meaning.to_string()));
    }
    for (name, k) in sig::NAMES {
        out.push((name.to_string(), 14, format!("a signal of type S, digit {}", k)));
    }
    let unit = assemble(text, ALL);
    for t in unit.tables.iter().filter(|t| !t.library) {
        out.push((t.name.clone(), 3, format!("table, line {}", t.header.line + 1)));
    }
    for e in LIBRARY {
        if !unit.tables.iter().any(|t| t.name == e.name && !t.library) {
            let first = e.source.lines().next().unwrap_or("").trim_start_matches("# ").to_string();
            out.push((e.name.to_string(), 3, first));
        }
    }
    out
}

fn show(toks: &[Token], source: &[char]) -> String {
    let mut s = String::new();
    for (i, t) in toks.iter().enumerate() {
        let text: String = match &t.tok {
            Tok::Dash => "-".into(),
            Tok::Tick(a) => format!("'{}'", a),
            _ => source[t.start..t.end].iter().collect(),
        };
        let prev = i.checked_sub(1).map(|j| &toks[j]);
        let glued = |a: &Token, b: &Token| a.end == b.start;
        let space = match (prev.map(|p| &p.tok), &t.tok) {
            (None, _) => false,
            (_, Tok::Comma | Tok::Semi | Tok::Dot) => false,
            // Digits in groups, and DL 1,14, keep their commas tight.
            (Some(Tok::Comma), Tok::Int(_)) => {
                let before = i.checked_sub(2).map(|j| &toks[j].tok);
                !(matches!(before, Some(Tok::Int(b)) if b.len() == 4) || (i >= 3 && toks[i - 3].tok == Tok::Name("DL".into())))
            }
            // A sign stays against its number.
            (Some(Tok::Dash | Tok::Plus), Tok::Int(_)) => !glued(prev.unwrap(), t) || matches!(i.checked_sub(2).map(|j| &toks[j].tok), Some(Tok::Int(_))),
            _ => true,
        };
        if space {
            s.push(' ');
        }
        s.push_str(&text);
    }
    s
}

/// The text set out in columns: detail figures to the right of a column of
/// four, instructions after them, the number of the next instruction last.
pub fn format(text: &str) -> String {
    let mut out = String::new();
    for (n, raw) in text.lines().enumerate() {
        let source: Vec<char> = raw.chars().collect();
        let toks = lex(raw);
        let comment: String = source[code_part(raw).chars().count()..].iter().collect();
        let line = match parse_line(n, raw) {
            None | Some(Err(_)) => raw.trim_end().to_string(),
            Some(Ok(Line::Header { name, .. })) => name,
            Some(Ok(Line::Store { .. })) => {
                // The storage, then its content in a column.
                let split = if toks[0].tok == Tok::Name("DL".into()) { 4 } else { 2 };
                format!("{:<9}{}", show(&toks[..split], &source), show(&toks[split..], &source))
            }
            Some(Ok(Line::Code { label, instrs, next })) => {
                let within = |s: &Span| -> Vec<Token> {
                    toks.iter().filter(|t| t.start >= s.start && t.end <= s.end).cloned().collect()
                };
                let figure = |l: &Label| match &l.table {
                    Some(t) => format!("{} {}.", t, l.fig),
                    None => l.fig.to_string(),
                };
                let head = match &label {
                    Some(l) if l.table.is_some() => format!("{} ", figure(l)),
                    Some(l) => format!("{:>4}   ", l.fig),
                    None => "       ".to_string(),
                };
                let body: Vec<String> = instrs.iter().map(|i| show(&within(&i.span), &source)).collect();
                let mut s = format!("{}{}", head, body.join(";  "));
                if let Some(next) = &next {
                    let pad = 34usize.saturating_sub(s.chars().count()).max(3);
                    s.push_str(&" ".repeat(pad));
                    s.push_str(&figure(next).replace('.', ""));
                }
                s.trim_end().to_string()
            }
        };
        out.push_str(&line);
        if !comment.is_empty() && !line.ends_with(comment.as_str()) {
            if !line.is_empty() {
                out.push_str("   ");
            }
            out.push_str(comment.trim_end());
        }
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEXT: &str = "# Fetch a number.\nFETCH\n  1   B, BURY\n  2   B, INDEXIN\n  3   TS 28 - DL 2,1\n  5   O, DL 2\n  6   B, FETCH 6\n\nTS 27   L, DL 3,5\nDL 3,5  1945\n";

    #[test]
    fn a_clean_text_has_no_problems() {
        assert_eq!(check(TEXT), Vec::new());
    }

    #[test]
    fn formatting_is_stable() {
        let once = format(TEXT);
        assert_eq!(format(&once), once);
        assert!(once.contains("   3   TS 28 - DL 2,1\n"), "{}", once);
        assert!(once.contains("TS 27    L, DL 3,5\n"), "{}", once);
        assert_eq!(check(&once), Vec::new());
        assert_eq!(format("INDEXIN\n3 ADD ‘A’ 4\n 1  Q,0000,0100,0000,0000   2\n"), "INDEXIN\n   3   ADD 'A'                    4\n   1   Q, 0000,0100,0000,0000     2\n");
    }

    #[test]
    fn hover_shows_the_machine_form() {
        let (text, _) = hover(TEXT, 4, 10).unwrap();
        assert!(text.contains("two stages"), "{}", text);
        assert!(text.contains("Type M: pass the content of TS 28 into TS 6."), "{}", text);
        let (text, _) = hover(TEXT, 3, 12).unwrap();
        assert!(text.contains("**INDEXIN**") && text.contains("General Description"), "{}", text);
    }

    #[test]
    fn names_are_found_and_followed() {
        assert_eq!(references(TEXT, 6, 12).len(), 2);
        assert_eq!(definition(TEXT, 6, 12), Some((6, 6, 16)));
        assert_eq!(tables(TEXT), vec![("FETCH".to_string(), 1, 6)]);
    }
}
