use ace::assemble::{assemble, Unit};
use ace::machine::{listing, Card, Stop};
use ace::word::{bits, Pos, P17};
use ace::{check_errata, load, numbers, run, time, DECISIONS, ERRATA, LIBRARY, STANDING, STEP_LIMIT};
use std::io::{IsTerminal, Read};
use std::process::ExitCode;

const USAGE: &str = "\
ace: the Automatic Computing Engine as Alan Turing proposed it in 1945

usage:
  ace run FILE [--input N,N,...] [--cards numbers|listing|holes] [--trace]
          [--time] [--cold] [--json] [--errata LIST] [--limit N]
        Run a job: the instruction tables in FILE and those it uses from the
        library. It starts at the first table in the file and ends when an
        instruction passes control to itself.
        --input puts numbers on cards in the reader, twelve to a card.
        Without it they are read from standard input.
        --cards chooses how punched cards are shown: as twelve numbers, as
        the line a tabulator would list, or as rows of holes.
        --trace prints every instruction obeyed, with the minor cycle it
        was found in. --time prints the machine time taken.
        --cold starts with nothing in the machine but the deck in the
        reader: see `ace deck`.
        --json prints the whole run as one JSON object instead: the
        machine before the start, every instruction obeyed with what it
        changed, and the cards. The playground steps through it.
        FILE may be `-`: standard input is then the tables, a line `----`,
        and the numbers.
  ace deck FILE [--holes] [--errata LIST]
        List the deck of cards that loads the job into an empty machine:
        what each card puts where, and with --holes the holes themselves.
        `ace run FILE --cold` starts from that deck. Section 7 of the report:
        the initial order `consists of 0 throughout`, which reads a card into
        DL 0; each card of the deck carries its own orders.
  ace tables FILE [--emit listing|cards|digits] [--errata LIST]
        Show a job in machine form: every instruction with its position,
        its 32 digits and its popular form; or as the 80-column instruction
        cards of section 7; or as the digits alone.
  ace library [NAME]
        List the tables in the library, or print one.
  ace audit
        Compare what the report prints with itself: the cards of Fig. 38
        with the table they are cards for, and INDEXIN as printed with
        INDEXIN corrected, on the machine.
  ace errata
        List the corrections to the printed tables.
  ace decisions
        List the places where the report says two things, and which was taken.
  ace check FILE
        List what is wrong with a job, and what will run but is probably not
        what was meant, one per line as FILE:LINE:COLUMN: KIND: MESSAGE.
  ace format FILE
        Print the tables set out in columns.
  ace lsp
        Serve editors, speaking the Language Server Protocol on standard
        input and output.

  --errata takes `all` (the default), `none`, or names from `ace errata`.";

struct Args {
    rest: Vec<String>,
}

impl Args {
    fn opt(&mut self, name: &str) -> Option<String> {
        let i = self.rest.iter().position(|a| a == name)?;
        if i + 1 >= self.rest.len() {
            return None;
        }
        self.rest.remove(i);
        Some(self.rest.remove(i))
    }
    fn flag(&mut self, name: &str) -> bool {
        match self.rest.iter().position(|a| a == name) {
            Some(i) => {
                self.rest.remove(i);
                true
            }
            None => false,
        }
    }
}

fn main() -> ExitCode {
    match real_main() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("ace: {}", e);
            ExitCode::FAILURE
        }
    }
}

fn show_card(card: &Card, how: &str) -> Result<String, String> {
    Ok(match how {
        "numbers" => {
            let last = card.iter().rposition(|w| *w != 0).map(|i| i + 1).unwrap_or(1);
            card[..last].iter().map(|w| (*w as i32).to_string()).collect::<Vec<_>>().join(" ")
        }
        "listing" => listing(card),
        "holes" => {
            let rows = ["12", "11", " 0", " 1", " 2", " 3", " 4", " 5", " 6", " 7", " 8", " 9"];
            let mut s = String::new();
            for (row, w) in rows.iter().zip(card) {
                let holes: String = (1..=32).map(|k| if ace::word::digit(*w, k) { 'o' } else { '.' }).collect();
                s.push_str(&format!("{}  {}\n", row, holes));
            }
            s.trim_end().to_string()
        }
        other => return Err(format!("unknown --cards {}", other)),
    })
}

/// A position as a trace names it.
fn place(unit: &Unit, cd: u32) -> String {
    if cd & P17 != 0 {
        return "TS 6".to_string();
    }
    let pos = Pos::of(cd);
    unit.name_of(pos).unwrap_or_else(|| pos.to_string())
}

fn real_main() -> Result<(), String> {
    let mut args = Args { rest: std::env::args().skip(1).collect() };
    let cmd = args.rest.first().cloned().unwrap_or_default();
    if cmd == "lsp" {
        let stdin = std::io::stdin();
        return ace::lsp::serve(&mut stdin.lock(), &mut std::io::stdout());
    }
    let errata_arg = args.opt("--errata").unwrap_or("all".into());
    let errata: Vec<&str> = errata_arg.split(',').collect();
    check_errata(&errata)?;
    // FILE may be `-`: standard input is then the tables, a line of four
    // dashes, and the numbers for the reader. The playground uses this.
    let piped = if args.rest.get(1).map(String::as_str) == Some("-") {
        let mut text = String::new();
        std::io::stdin().read_to_string(&mut text).map_err(|e| e.to_string())?;
        Some(match text.split_once("\n----\n") {
            Some((tables, input)) => (tables.to_string(), input.to_string()),
            None => (text, String::new()),
        })
    } else {
        None
    };
    let source = |args: &Args| -> Result<(String, String), String> {
        if let Some((tables, _)) = &piped {
            return Ok(("-".to_string(), tables.clone()));
        }
        let path = args.rest.get(1).ok_or("which file?")?;
        Ok((path.clone(), std::fs::read_to_string(path).map_err(|e| format!("{}: {}", path, e))?))
    };
    let job = |args: &Args| -> Result<Unit, String> {
        let (name, text) = source(args)?;
        let unit = assemble(&text, &errata);
        match unit.errors().first() {
            Some(p) => Err(format!("{}:{}:{}: {}", name, p.span.line + 1, p.span.start + 1, p.message)),
            None => Ok(unit),
        }
    };
    match cmd.as_str() {
        "run" => {
            let input = match (args.opt("--input"), &piped) {
                (Some(list), _) => numbers(&list)?,
                (None, Some((_, text))) => numbers(text)?,
                (None, None) if !std::io::stdin().is_terminal() => {
                    let mut text = String::new();
                    std::io::stdin().read_to_string(&mut text).map_err(|e| e.to_string())?;
                    numbers(&text)?
                }
                (None, None) => Vec::new(),
            };
            let cards = args.opt("--cards");
            let limit = match args.opt("--limit") {
                Some(n) => n.parse::<u64>().map_err(|_| format!("--limit {} is not a number", n))?,
                None => STEP_LIMIT,
            };
            let (trace, timed, json) = (args.flag("--trace"), args.flag("--time"), args.flag("--json"));
            let cold = args.flag("--cold");
            let unit = job(&args)?;
            let how = cards.or(unit.cards.clone()).unwrap_or("numbers".into());
            show_card(&[0; 12], &how)?;
            let (mut m, deck) = if cold { ace::deck::cold(&unit, &input)? } else { (load(&unit, &input)?, 0) };
            if json {
                return run_json(&unit, m, deck, limit, &how);
            }
            let mut shown = 0;
            let stop = run(&mut m, limit, |m, e| {
                if trace {
                    println!(
                        "{:>8}  {:<12} {}  {:<26} {}",
                        e.fetched,
                        place(&unit, e.cd),
                        bits(e.ci),
                        unit.popular(e.ci),
                        e.effect
                    );
                }
                for card in &m.punched[shown..] {
                    println!("{}", show_card(card, &how).unwrap());
                }
                shown = m.punched.len();
            });
            if timed {
                if deck > 0 {
                    println!("{} cards of the deck read from cold", deck);
                }
                println!(
                    "{} instructions in {} minor cycles: {} of machine time",
                    m.steps,
                    m.time,
                    time(m.time)
                );
            }
            match stop {
                Stop::Idle(_) => Ok(()),
                stop => Err(format!("stopped at {}: {}", place(&unit, m.cd), stop)),
            }
        }
        "deck" => {
            let unit = job(&args)?;
            let deck = ace::deck::deck(&unit)?;
            let holes = args.flag("--holes");
            for (n, punched) in deck.iter().enumerate() {
                println!("card {:>3}  {}", n + 1, punched.what);
                if holes {
                    println!("{}\n", show_card(&punched.card, "holes")?);
                }
            }
            println!("{} cards", deck.len());
            Ok(())
        }
        "tables" => {
            let emit = args.opt("--emit").unwrap_or("listing".into());
            let unit = job(&args)?;
            tables(&unit, &emit)
        }
        "library" => {
            match args.rest.get(1) {
                Some(name) => match ace::library(name, &errata) {
                    Some(source) => print!("{}", source),
                    None => return Err(format!("there is no table {} in the library", name)),
                },
                None => {
                    for e in LIBRARY {
                        let whose = if e.printed { "printed in the report" } else { "described in the report; the table is ours" };
                        let unit = assemble(&ace::library(e.name, &errata).unwrap(), &errata);
                        let n = unit.table(e.name).map(|t| t.slots.len()).unwrap_or(0);
                        println!("{:<9}{:>3} minor cycles   {}", e.name, n, whose);
                    }
                }
            }
            Ok(())
        }
        "audit" => {
            print!("{}", ace::audit::report());
            Ok(())
        }
        "errata" => {
            for e in ERRATA {
                println!(
                    "{}  ({})\n    {}\n    {}\n    Kind: {}.\n    Known before: {}.\n",
                    e.id,
                    e.table,
                    e.title,
                    e.detail,
                    e.kind.describe(),
                    e.known
                );
            }
            for table in ["INDEXIN", "CALPOL", "DISCRIM"] {
                let of = |k: ace::Kind| ERRATA.iter().filter(|e| e.table == table && e.kind == k).count();
                println!(
                    "{:<8} {} slip{}, {} conflict{}, {} ours",
                    table,
                    of(ace::Kind::Slip),
                    if of(ace::Kind::Slip) == 1 { "" } else { "s" },
                    of(ace::Kind::Conflict),
                    if of(ace::Kind::Conflict) == 1 { "" } else { "s" },
                    of(ace::Kind::Ours)
                );
            }
            Ok(())
        }
        "decisions" => {
            for d in DECISIONS {
                println!("{}. {}", d.id, d.question);
                println!("    one place:   {}", d.one);
                println!("    another:     {}", d.other);
                println!("    taken here:  {}", d.taken);
                if !d.noted.is_empty() {
                    println!("    noted by:    {}", d.noted);
                }
                println!();
            }
            println!("Before the start the tool sets:");
            for (n, _, why) in STANDING {
                println!("    TS {:<3} {}", n, why);
            }
            Ok(())
        }
        "check" | "format" => {
            let (name, text) = source(&args)?;
            if cmd == "format" {
                print!("{}", ace::check::format(&text));
                return Ok(());
            }
            let problems = ace::check::check(&text);
            for d in &problems {
                let kind = if d.error { "error" } else { "warning" };
                println!("{}:{}:{}: {}: {}", name, d.span.line + 1, d.span.start + 1, kind, d.message);
            }
            match problems.iter().filter(|d| d.error).count() {
                0 => Ok(()),
                n => Err(format!("{} error{}", n, if n == 1 { "" } else { "s" })),
            }
        }
        _ => {
            println!("{}", USAGE);
            Ok(())
        }
    }
}

/// The 80 columns of an instruction-table card, as section 7 (v) lays them
/// out: popular name 1-8, detail figure 9-11, instruction 12-25, repeat of
/// destination 26-40, genuine input 41-72, job number 73-77, spare 78-80.
fn card(name: &str, fig: u32, popular: &str, pos: Pos, word: u32) -> String {
    let destination: String = bits(pos.word()).replace(',', "")[17..].to_string();
    let genuine: String = bits(word).replace(',', "");
    let brief: String = popular.chars().filter(|c| *c != ' ').take(14).collect();
    format!("{:<8.8}{:>3}{:<14}{}{}{:>5}   ", name, fig, brief, destination, genuine, 1)
}

fn tables(unit: &Unit, emit: &str) -> Result<(), String> {
    match emit {
        "listing" => {
            for t in &unit.tables {
                let whose = match ace::LIBRARY.iter().find(|e| e.name == t.name && t.library) {
                    Some(e) if e.printed => "  (library: printed in the report)",
                    Some(_) => "  (library: ours, to the report's description)",
                    None => "",
                };
                println!("{}{}", t.name, whose);
                for s in &t.slots {
                    println!(
                        "  {:>3}   {:<10} {}   {}",
                        s.fig,
                        s.pos.to_string(),
                        bits(s.word),
                        unit.popular(s.word)
                    );
                }
                println!();
            }
            for (pos, w, _) in &unit.store {
                println!("{:<18}{}   {}", pos.to_string(), bits(*w), *w as i32);
            }
            for (n, w, _) in &unit.ts {
                println!("{:<18}{}   {}", format!("TS {}", n), bits(*w), *w as i32);
            }
        }
        "cards" => {
            println!("{}", "popular  figinstruction   destination    genuine input                   job");
            for t in &unit.tables {
                for s in &t.slots {
                    println!("{}", card(&t.name, s.fig, &unit.popular(s.word), s.pos, s.word));
                }
            }
        }
        "digits" => {
            for t in &unit.tables {
                for s in &t.slots {
                    println!("{}", bits(s.word));
                }
            }
        }
        e => return Err(format!("unknown --emit {}", e)),
    }
    Ok(())
}


/// The run as JSON, for a page that steps through it.
///
/// `start` is the machine before the first instruction: the non-zero minor
/// cycles of the delay lines as [line, slot, word] and TS 1 to TS 32. Each
/// step is [began, fetched, done, cd, ci, form, ts, cells, cards]: the minor
/// cycle the search for the instruction began, the one it reached CI in, the
/// one after it finished; CD and CI; an index into `forms`, its popular form;
/// the temporary storages that changed as [n, word]; the minor cycles that
/// changed as [line, slot, word]; and how many cards had been punched.
fn run_json(unit: &Unit, mut m: ace::machine::Machine, deck: usize, limit: u64, how: &str) -> Result<(), String> {
    use std::collections::HashMap;
    use std::fmt::Write;
    let mut out = String::from("{\"start\":{\"cells\":[");
    let mut first = true;
    for (dl, line) in m.lines.iter().enumerate() {
        for (slot, w) in line.iter().enumerate() {
            if *w != 0 {
                let _ = write!(out, "{}[{},{},{}]", if first { "" } else { "," }, dl, slot, w);
                first = false;
            }
        }
    }
    let ts: Vec<String> = m.ts[1..].iter().map(|w| w.to_string()).collect();
    let reader: Vec<String> = m.reader.iter().map(|c| format!("[{}]", c.iter().map(|w| w.to_string()).collect::<Vec<_>>().join(","))).collect();
    let _ = write!(out, "],\"ts\":[{}],\"cd\":{},\"reader\":[{}]}},\"names\":[", ts.join(","), m.cd, reader.join(","));
    let mut first = true;
    for t in &unit.tables {
        for slot in &t.slots {
            let _ = write!(out, "{}[{},{},{}]", if first { "" } else { "," }, slot.pos.dl, slot.pos.slot, quote(&format!("{} {}", t.name, slot.fig)));
            first = false;
        }
    }
    out.push_str("],\"steps\":[");
    let mut forms: Vec<String> = Vec::new();
    let mut index: HashMap<u32, usize> = HashMap::new();
    let mut steps: Vec<String> = Vec::new();
    let stop = loop {
        if m.steps >= limit {
            break Stop::Limit;
        }
        let (began, ts_before) = (m.time, m.ts);
        let event = match m.step() {
            Ok(e) => e,
            Err(stop) => break stop,
        };
        let form = *index.entry(event.ci).or_insert_with(|| {
            forms.push(unit.popular(event.ci));
            forms.len() - 1
        });
        let changed: Vec<String> =
            (1..=32).filter(|n| m.ts[*n] != ts_before[*n]).map(|n| format!("[{},{}]", n, m.ts[n])).collect();
        // Only types K and P write to the delay lines.
        let mut cells = Vec::new();
        if !ace::word::digit(event.ci, 3) {
            match ace::machine::Type::of(event.ci) {
                Some(ace::machine::Type::K) => {
                    let at = Pos::of(event.ci);
                    cells.push(format!("[{},{},{}]", at.dl, at.slot, m.ts[6]));
                }
                Some(ace::machine::Type::P) => {
                    let dl = Pos::of(event.ci).dl as usize;
                    for slot in 0..12 {
                        cells.push(format!("[{},{},{}]", dl, slot, m.lines[dl][slot]));
                    }
                }
                _ => {}
            }
        }
        steps.push(format!(
            "[{},{},{},{},{},{},[{}],[{}],{}]",
            began,
            event.fetched,
            event.done,
            event.cd,
            event.ci,
            form,
            changed.join(","),
            cells.join(","),
            m.punched.len()
        ));
    };
    out.push_str(&steps.join(","));
    let forms: Vec<String> = forms.iter().map(|f| quote(f)).collect();
    let cards: Vec<String> = m.punched.iter().map(|c| quote(&show_card(c, how).unwrap())).collect();
    let raw: Vec<String> = m.punched.iter().map(|c| format!("[{}]", c.iter().map(|w| w.to_string()).collect::<Vec<_>>().join(","))).collect();
    let _ = write!(
        out,
        "],\"forms\":[{}],\"cards\":[{}],\"raw\":[{}],\"stop\":{},\"idle\":{},\"at\":{},\"time\":{},\"count\":{},\"deck\":{}}}",
        forms.join(","),
        cards.join(","),
        raw.join(","),
        quote(&stop.to_string()),
        matches!(stop, Stop::Idle(_)),
        quote(&place(unit, m.cd)),
        m.time,
        m.steps,
        deck
    );
    println!("{}", out);
    Ok(())
}

fn quote(s: &str) -> String {
    let mut out = String::new();
    ace::json::Json::Str(s.to_string()).write(&mut out);
    out
}
