//! The Automatic Computing Engine as Alan Turing proposed it in 1945: the
//! machine of the report, the notation of its instruction tables, the tables
//! the report prints, and the ones it only describes.

pub mod assemble;
pub mod audit;
pub mod check;
pub mod deck;
pub mod json;
pub mod lsp;
pub mod machine;
pub mod notation;
pub mod word;

use assemble::{Unit, NOTES};
use machine::{Card, Event, Machine, Stop, ROWS};
use word::Word;

pub const STEP_LIMIT: u64 = 20_000_000;

/// A minor cycle is 32 pulses a microsecond apart.
pub const MINOR_CYCLE_MICROSECONDS: u64 = 32;

/// A table in the library.
pub struct Entry {
    pub name: &'static str,
    /// Whose table it is: the report prints it, or it is written here to the
    /// report's description.
    pub printed: bool,
    pub source: &'static str,
}

pub const LIBRARY: &[Entry] = &[
    Entry { name: "INDEXIN", printed: true, source: include_str!("../tables/indexin.ace") },
    Entry { name: "BURY", printed: false, source: include_str!("../tables/bury.ace") },
    Entry { name: "UNBURY", printed: false, source: include_str!("../tables/unbury.ace") },
    Entry { name: "DISCRIM", printed: false, source: include_str!("../tables/discrim.ace") },
    Entry { name: "PLUSIND", printed: false, source: include_str!("../tables/plusind.ace") },
    Entry { name: "TRANS", printed: false, source: include_str!("../tables/trans.ace") },
    Entry { name: "BINDEC", printed: false, source: include_str!("../tables/bindec.ace") },
    Entry { name: "CALPOL", printed: true, source: include_str!("../tables/calpol.ace") },
    Entry { name: "MULTIP", printed: false, source: include_str!("../tables/multip.ace") },
    Entry { name: "ADD", printed: false, source: include_str!("../tables/add.ace") },
];

const DISCRIM_CORRECTED: &str = include_str!("../tables/discrim-corrected.ace");

/// A correction to a table as the report prints or outlines it.
pub struct Erratum {
    pub id: &'static str,
    pub table: &'static str,
    pub title: &'static str,
    pub detail: &'static str,
    /// Who noticed it first, as far as we could find.
    pub known: &'static str,
    /// What kind of correction it is: see `Kind`.
    pub kind: Kind,
    edits: &'static [(&'static str, &'static str)],
}

/// Not every correction is the same kind of thing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// The table is wrong under any reading of the report.
    Slip,
    /// The report says two things, and the table follows the one not taken
    /// here (see `ace decisions`).
    Conflict,
    /// Needed only because of how the tables written here, which the report
    /// describes but does not print, do their work.
    Ours,
}

impl Kind {
    pub fn describe(self) -> &'static str {
        match self {
            Kind::Slip => "a slip: wrong under any reading of the report",
            Kind::Conflict => "a conflict: the report says two things, and the table follows the other",
            Kind::Ours => "ours: needed only because of how the tables written here work",
        }
    }
}

pub const ERRATA: &[Erratum] = &[
    Erratum {
        id: "literal",
        table: "INDEXIN",
        title: "INDEXIN 1 builds the code of type K, not of type L",
        detail: "INDEXIN makes an instruction to fetch a minor cycle and obeys it from TS 6. \
                 Line 1 supplies the digits of its type: 0000,0100, which after the rotation \
                 are digits 5-8 = 0100. Fig. 19 gives 0100 to type K, which stores TS 6, and \
                 0101 to type L, which fetches. As printed the table writes the instruction it \
                 has just built over the minor cycle it was sent to read. The literal becomes \
                 0000,0101,0000,0000. This holds for INDEXIN as its general description \
                 defines it, with a position in digits 17-32 of TS 27. CALPOL calls it with a \
                 whole order of type L in TS 27, whose digits 5-8 already say L, and the OR on \
                 line 9 then makes the code right: Turing's own use of INDEXIN would not have \
                 shown this slip.",
        known: "Carpenter and Doran, 1986, note 17",
        kind: Kind::Slip,
        edits: &[("Q, 0000,0100,0000,0000", "Q, 0000,0101,0000,0000")],
    },
    Erratum {
        id: "half",
        table: "INDEXIN",
        title: "INDEXIN 5 takes the half of the partial sum the digits have left",
        detail: "Type Q can only fill digits 17-32, so the type digits are made there and moved \
                 by ROTATE 16, `Rotate the content of TS 4, TS 5 forwards 16 places`. Section 12 \
                 says which way forwards is: the facility is for `multiplying the partial sum by \
                 a power of 2`, and consists in `enabling the partial sums to be delayed`. A \
                 delayed digit comes out later and later digits are more significant, so the \
                 digits go 16 places up, out of TS 4 and into digits 1-16 of TS 5. Line 5 takes \
                 TS 4. It becomes TS 5 - TS 6.",
        known: "not that we found",
        kind: Kind::Slip,
        edits: &[("5   TS 4 - TS 6 ", "5   TS 5 - TS 6 ")],
    },
    Erratum {
        id: "return",
        table: "INDEXIN",
        title: "INDEXIN 13 buries where it should unbury",
        detail: "The table ends B, BURY. BURY adds a note to the list and comes back to the \
                 instruction after the one that called it, which is past the end of the table. \
                 A subsidiary ends by taking the note off: B, UNBURY, as CALPOL does.",
        known: "Carpenter and Doran, 1986, note 18",
        kind: Kind::Slip,
        edits: &[("B, BURY", "B, UNBURY")],
    },
    Erratum {
        id: "clear",
        table: "CALPOL",
        title: "CALPOL 1 clears the result, not the running product",
        detail: "Each turn adds the next coefficient, in TS 18, 19, to the product so far, in \
                 TS 20, 21. Before the first turn that product must be zero. The table clears \
                 TS 22, 23, which ADD is about to overwrite. On a machine just switched on \
                 TS 20, 21 are zero anyway; used a second time, CALPOL starts from the last \
                 product of the time before. It becomes CLEAR TS 20, 21.",
        known: "Carpenter and Doran analysed CALPOL in 1977; we have not seen that paper",
        kind: Kind::Slip,
        edits: &[("CLEAR TS 22, 23", "CLEAR TS 20, 21")],
    },
    Erratum {
        id: "inputs",
        table: "CALPOL",
        title: "CALPOL loads TS 2 and TS 3 for an AND that reads TS 9 and TS 10",
        detail: "The test for the last coefficient is TS 27 & TS 29. The table puts them in \
                 TS 2 and TS 3; the logical operations of section 12 and Fig. 29 take TS 9 and \
                 TS 10, as INDEXIN knows. The two transfers go to TS 9 and TS 10.",
        known: "Carpenter and Doran, 1986, note 12, for the conflict in section 12",
        kind: Kind::Conflict,
        edits: &[("TS 27 - TS 2; TS 29 - TS 3; AND", "TS 27 - TS 9; TS 29 - TS 10; AND")],
    },
    Erratum {
        id: "branch",
        table: "CALPOL",
        title: "CALPOL builds its two ways on with type Q, which cannot make an instruction",
        detail: "DISCRIM chooses between two instructions in TS 15 and TS 16, and B, 1 obeys \
                 the one chosen. The table makes them with Q, CALPOL 40 and Q, CALPOL 37. Type \
                 Q fills digits 17-32 only, so what reaches TS 15 has an instruction number \
                 but not the digits of type B: whatever was left in the first half of TS 6 \
                 decides what it does. And CALPOL 40 is the middle of the multiplication; the \
                 way out is CALPOL 50. The two instructions, B, CALPOL 50 and B, CALPOL 37, \
                 are kept as numbers at the end of the table and fetched.",
        known: "Carpenter and Doran analysed CALPOL in 1977; we have not seen that paper",
        kind: Kind::Slip,
        edits: &[
            ("Q, CALPOL 40;\nTS 6 - TS 15; Q, CALPOL 37; TS 6 - TS 16;", "CALPOL 61 - TS 15;\nCALPOL 62 - TS 16;"),
            ("TS 23 - TS 26; B, UNBURY.\n", "TS 23 - TS 26; B, UNBURY.\nCALPOL 61. B, CALPOL 50\nCALPOL 62. B, CALPOL 37\n"),
        ],
    },
    Erratum {
        id: "carry",
        table: "CALPOL",
        title: "CALPOL obeys TS 6 after DISCRIM, but the choice is in TS 24",
        detail: "DISCRIM passes the chosen instruction into TS 24 and comes back through \
                 UNBURY. The UNBURY written here returns by building the return instruction in \
                 TS 6 and obeying it there, so B, 1 then obeys the instruction that returned, \
                 which returns to B, 1 again, for ever. TS 24 - TS 6 is put in front of it. An \
                 UNBURY that returned through a minor cycle in a delay line instead, and left \
                 TS 6 alone, would not need this.",
        known: "not that we found; it depends on how UNBURY is written",
        kind: Kind::Ours,
        edits: &[("B, DISCRIM; B, 1", "B, DISCRIM; TS 24 - TS 6; B, 1")],
    },
    Erratum {
        id: "contents",
        table: "CALPOL",
        title: "The contents printed for DL 1,14 and DL 1,15 do not name DL 3,1",
        detail: "DL 1,14 is to be the order to transfer DL 3,1 to TS 6, and DL 1,15 the digit \
                 that shows when the last coefficient has been taken. By Fig. 19 the order \
                 printed has digits 18, 22 and 23: minor cycle 18 of DL 1. The contents \
                 become the instruction L, DL 3,1 and digit 25, which is 0 in DL 3 and 1 in \
                 DL 4, where PLUSIND arrives after 32 minor cycles.",
        known: "Carpenter and Doran analysed CALPOL in 1977; we have not seen that paper",
        kind: Kind::Slip,
        edits: &[
            ("DL 1,14  0000,0101,0000,0000,0100,0110,0000,0000", "DL 1,14  L, DL 3,1"),
            ("DL 1,15  0000,0000,0000,0000,0000,0100,0000,0000", "DL 1,15  0000,0000,0000,0000,0000,0000,1000,0000"),
        ],
    },
    Erratum {
        id: "sign",
        table: "DISCRIM",
        title: "DISCRIM, as outlined, calls a word zero when its digit 32 is a 1",
        detail: "The outline subtracts TS 8 from zero and takes TS 5 to be all ones if TS 8 had \
                 any digit 1. But digit 32 has reversed sign. A word with a 1 there is a \
                 negative number, zero minus it is positive, and TS 5 is all zeros: the same \
                 answer as for a word of 32 zeros. The corrected table adds TS 8 as well as \
                 subtracting it, and uses either TS 5.",
        known: "not that we found",
        kind: Kind::Slip,
        edits: &[],
    },
];

/// A place where the report says two things, and the one taken here.
pub struct Decision {
    pub id: u32,
    pub question: &'static str,
    pub one: &'static str,
    pub other: &'static str,
    pub taken: &'static str,
    pub noted: &'static str,
}

pub const DECISIONS: &[Decision] = &[
    Decision {
        id: 1,
        question: "Which digits 5-8 mean which type of operation",
        one: "Fig. 19: K 0100, L 0101, M 0010, N 0011, O 0001, P 0000, Q 0110, R 0111, S 1000, T 1001",
        other: "Section 12 and Fig. 30: the tree element TRG 000 is K v L, TRG 001 is M v N",
        taken: "Fig. 19. The cards of Fig. 38 and the constant in CALPOL use its codes.",
        noted: "",
    },
    Decision {
        id: 2,
        question: "Where type B puts the position it leaves",
        one: "Section 11: `Pass the old CD into TS 13`",
        other: "Section 12: TS 1 carries `the CD which applied just before the last instruction of type B`; BURY reads TS 1",
        taken: "TS 1. CALPOL keeps its argument in TS 13.",
        noted: "Carpenter and Doran, 1986, note 11",
    },
    Decision {
        id: 3,
        question: "Which storages the logical operations work on",
        one: "Section 12: TS 9 and TS 10 are the inputs, TS 8 the result; so Fig. 29 and INDEXIN",
        other: "Section 12, five paragraphs earlier: TS 2 and TS 3 hold the arguments `for the logical operations`; CALPOL loads TS 2 and TS 3 before AND",
        taken: "TS 9 and TS 10.",
        noted: "Carpenter and Doran, 1986, note 12",
    },
    Decision {
        id: 4,
        question: "Which digits select a logical operation",
        one: "Section 12: `digits 18 to 23`, values 000000, 001000, 010000, 011000, 100000",
        other: "Fig. 19 marks digits 23-28; Fig. 29 selects with the tree elements TRA and TRB 000, which section 11 gives to digits 23-25 and 26-28",
        taken: "Digits 23-28: the first three choose the operation, the last three are 0.",
        noted: "",
    },
    Decision {
        id: 5,
        question: "Which storage is the multiplicand",
        one: "Section 12: the multiplicand is TS 3, the multiplier TS 2",
        other: "Fig. 32 leads TS 2 to the adders and TS 3 to MULIER; INDEXIN and DISCRIM add TS 2",
        taken: "TS 2 is added; TS 3 is the multiplier. The printed tables depend on it.",
        noted: "",
    },
    Decision {
        id: 6,
        question: "Where a given multiplier sits in the instruction",
        one: "Section 12: P23, `the use of digits 23, 24, 25, 26`; Fig. 19",
        other: "Section 12, a page on: `the number 1 in columns 24-27`; Fig. 37",
        taken: "Digits 24-27, as the worked instructions of Fig. 37 have it.",
        noted: "Carpenter and Doran, 1986, note 13",
    },
    Decision {
        id: 7,
        question: "Where ROTATE finds how far to go",
        one: "Section 12: digits 1-5 of TS 7, though `any time up to 63` needs six",
        other: "Fig. 33 gates TS 7 at P13 to P17",
        taken: "Digits 1-5: TS 7 holds the number of places. The 64 digits of TS 4 and TS 5 turn as a closed cycle towards the more significant end.",
        noted: "Carpenter and Doran, 1986, note 14",
    },
    Decision {
        id: 8,
        question: "What type Q does to digits 1-16 of TS 6",
        one: "Section 12: `Pass CI 17-32 into TS 6`",
        other: "Section 12 and Fig. 30: in case Q, CATIM lasts from P17 for half a minor cycle; Fig. 27 stops TS 6 circulating only while Q lasts",
        taken: "Digits 1-16 stay as they were. CALPOL's `Clear TS 22, 23` in three instructions would need them cleared.",
        noted: "",
    },
    Decision {
        id: 9,
        question: "How minor cycles are numbered",
        one: "Fig. 19 weights digits 18-22 as 16, 1, 2, 4, 8, so that adding P19 goes to the next number",
        other: "The carry from digit 22 goes into digit 23: after number 15 of a line comes number 0 of the next line, not 16",
        taken: "Positions are kept as delay line and minor cycle. Tables lie in the even minor cycles, 16 instructions to a line.",
        noted: "",
    },
];

/// What the simulation does where the report says nothing, as distinct from
/// the decisions, where it says two things.
pub const CONVENTIONS: &[(&str, &str)] = &[
    ("Long multiplication", "AUTO is simulated as one signed multiplication of the multiplicand by all 32 digits of the multiplier (TS 3, or TS 11 with ERR), which is what eight repetitions on four digits at a time amount to. How DIFF and TS 12 choose the digits during AUTO is not simulated; GIV with AUTO is not supported."),
    ("DIFF without TS 12", "Outside AUTO, DIFF takes the multiplier digits from the lowest pulse in TS 12; with TS 12 empty it falls back to the ordinary choice."),
    ("Several multiplier sources", "If GIV, ERR and NOR are set together, GIV is taken first, then ERR."),
    ("Logical operations not listed", "TRA digits other than the five of section 12 put 0 into TS 8."),
    ("Unused codes", "Digits 5-8 that name none of the ten types do nothing for one minor cycle."),
    ("TS 32", "A five-digit TS field of 0 names TS 32."),
    ("The start", "Every storage starts at 0. A job put straight into the machine may also set TS 1, 4, 5 and 8, which no instruction can fill; a job loaded from cards cannot."),
    ("Cards", "A card takes 18,750 minor cycles, 100 cards a minute, and its twelve rows move at once; the report describes a signal for each row and gives no speed."),
    ("The end", "An instruction of type B that names itself ends the run. On the machine it would go on obeying itself."),
    ("Valve elements", "Type T records which element was stimulated; no circuit is connected to it."),
    ("Rotation with arithmetic", "When ROTATE is set with other signals, the arithmetic is done first and the rotation after. THREE changes only how long the rotation takes."),
    ("ROTATE n", "The machine rotates by the amount in TS 7. The n written after ROTATE is for the reader, and `ace check` warns when it is not 16, which TS 7 holds unless the job changes it."),
    ("BURY and UNBURY", "Ours, to the descriptions of sections 6 and 13. TS 31 points at the next free place for a note; UNBURY moves it back one before reading. The note is the position of the B that entered the subsidiary, and both tables go on at the instruction after it."),
    ("INDEXIN and TS 27", "INDEXIN ORs all 32 digits of TS 27 into the order it builds, although its description speaks only of digits 17-32. Digits 1-16 of TS 27 must be zero, or the code of type L, as CALPOL leaves them."),
    ("Timing", "The search for the next instruction overlaps the arithmetic of this one (section 11). What instructions do to the storages happens in order; only the timing overlaps."),
];

/// The source of a library table, with the chosen errata applied. `ids` may
/// hold "all" or "none".
pub fn library(name: &str, ids: &[&str]) -> Option<String> {
    let entry = LIBRARY.iter().find(|e| e.name == name)?;
    let chosen = |id: &str| ids.contains(&"all") || ids.contains(&id);
    if name == "DISCRIM" && chosen("sign") {
        return Some(DISCRIM_CORRECTED.to_string());
    }
    let mut source = entry.source.to_string();
    for e in ERRATA.iter().filter(|e| e.table == name && chosen(e.id)) {
        for (from, to) in e.edits {
            assert!(source.contains(from), "erratum {} does not apply", e.id);
            source = source.replace(from, to);
        }
    }
    Some(source)
}

pub fn check_errata(ids: &[&str]) -> Result<(), String> {
    for id in ids {
        if !["all", "none"].contains(id) && !ERRATA.iter().any(|e| e.id == *id) {
            return Err(format!("unknown erratum `{}`: see `ace errata`", id));
        }
    }
    Ok(())
}

/// What the tool puts in the temporary storages before the start, because
/// the tables need it and nothing in the report sets it.
pub const STANDING: [(usize, Word, &str); 2] = [
    (31, NOTES.word(), "the position of the first note BURY will make, DL 31,1"),
    (7, 16, "the 16 places INDEXIN rotates by"),
];

/// A machine with a job in it, about to obey the first instruction of the
/// first table in the text.
pub fn load(unit: &Unit, input: &[Word]) -> Result<Machine, String> {
    if let Some(p) = unit.errors().first() {
        return Err(format!("line {}: {}", p.span.line + 1, p.message));
    }
    let mut m = Machine::new();
    for (n, w, _) in STANDING {
        m.ts[n] = w;
    }
    for table in &unit.tables {
        for slot in &table.slots {
            m.write(slot.pos, slot.word).map_err(|e| e.to_string())?;
        }
    }
    for (pos, w, _) in &unit.store {
        m.write(*pos, *w).map_err(|e| e.to_string())?;
    }
    for (n, w, _) in &unit.ts {
        m.ts[*n] = *w;
    }
    for chunk in input.chunks(ROWS) {
        let mut card: Card = [0; ROWS];
        card[..chunk.len()].copy_from_slice(chunk);
        m.reader.push_back(card);
    }
    m.cd = unit.entry.ok_or("there is no instruction to start at")?.word();
    Ok(m)
}

/// Runs until the machine stops, telling `watch` of each instruction obeyed.
pub fn run(m: &mut Machine, limit: u64, mut watch: impl FnMut(&Machine, &Event)) -> Stop {
    loop {
        if m.steps >= limit {
            return Stop::Limit;
        }
        match m.step() {
            Ok(event) => watch(m, &event),
            Err(stop) => return stop,
        }
    }
}

/// Machine time, as the report would reckon it.
pub fn time(minor_cycles: u64) -> String {
    let us = minor_cycles * MINOR_CYCLE_MICROSECONDS;
    if us < 10_000 {
        format!("{} microseconds", us)
    } else if us < 10_000_000 {
        format!("{:.1} milliseconds", us as f64 / 1000.0)
    } else {
        format!("{:.1} seconds", us as f64 / 1_000_000.0)
    }
}

/// Numbers for the reader: decimal, with or without a sign, separated by
/// commas or spaces. Twelve go on a card.
pub fn numbers(text: &str) -> Result<Vec<Word>, String> {
    text.split(|c: char| c == ',' || c.is_whitespace())
        .filter(|p| !p.is_empty())
        .map(|p| {
            p.parse::<i64>()
                .ok()
                .filter(|v| *v >= i32::MIN as i64 && *v <= u32::MAX as i64)
                .map(|v| v as u32)
                .ok_or_else(|| format!("`{}` is not a number that goes into 32 binary digits", p))
        })
        .collect()
}
