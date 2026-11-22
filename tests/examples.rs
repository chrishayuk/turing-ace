//! Every program in examples/ runs and prints what its .out file says. A
//! program reads its .in file, if it has one, as cards.

use ace::assemble::assemble;
use ace::machine::{listing, Stop};
use ace::{load, numbers, run, STEP_LIMIT};
use std::fs;

fn punch(card: &ace::machine::Card, how: &str) -> String {
    match how {
        "listing" => listing(card),
        _ => {
            let last = card.iter().rposition(|w| *w != 0).map(|i| i + 1).unwrap_or(1);
            card[..last].iter().map(|w| (*w as i32).to_string()).collect::<Vec<_>>().join(" ")
        }
    }
}

#[test]
fn every_example_prints_its_out_file() {
    let mut names: Vec<String> = fs::read_dir("examples")
        .unwrap()
        .filter_map(|e| e.ok()?.file_name().into_string().ok())
        .filter(|n| n.ends_with(".ace"))
        .collect();
    names.sort();
    assert!(!names.is_empty());
    let mut failed = Vec::new();
    for name in &names {
        let stem = name.trim_end_matches(".ace");
        let text = fs::read_to_string(format!("examples/{}", name)).unwrap();
        let input = fs::read_to_string(format!("examples/{}.in", stem)).unwrap_or_default();
        let expected = fs::read_to_string(format!("examples/{}.out", stem))
            .unwrap_or_else(|_| panic!("examples/{}.out is missing", stem));
        let unit = assemble(&text, &["all"]);
        assert!(unit.problems.is_empty(), "{}: {:?}", name, unit.problems);
        let how = unit.cards.clone().unwrap_or("numbers".into());
        let mut m = load(&unit, &numbers(&input).unwrap()).unwrap();
        let stop = run(&mut m, STEP_LIMIT, |_, _| {});
        let mut out: String = m.punched.iter().map(|c| punch(c, &how) + "\n").collect();
        if !matches!(stop, Stop::Idle(_)) {
            out.push_str(&format!("stopped: {}\n", stop));
        }
        if out != expected {
            failed.push(format!("{}\n--- expected\n{}--- got\n{}", name, expected, out));
        }
    }
    assert!(failed.is_empty(), "{}", failed.join("\n"));
}

/// Loaded from cold, from the deck of cards alone, every example punches the
/// same cards as when it is put straight into the machine.
#[test]
fn every_example_runs_the_same_from_cold() {
    let mut failed = Vec::new();
    for entry in fs::read_dir("examples").unwrap() {
        let name = entry.unwrap().file_name().into_string().unwrap();
        let Some(stem) = name.strip_suffix(".ace") else { continue };
        let text = fs::read_to_string(format!("examples/{}", name)).unwrap();
        let input = numbers(&fs::read_to_string(format!("examples/{}.in", stem)).unwrap_or_default()).unwrap();
        let unit = assemble(&text, &["all"]);
        let mut warm = load(&unit, &input).unwrap();
        let warm_stop = run(&mut warm, STEP_LIMIT, |_, _| {});
        let (mut cold, cards) = ace::deck::cold(&unit, &input).unwrap();
        let cold_stop = run(&mut cold, STEP_LIMIT, |_, _| {});
        if warm.punched != cold.punched || warm_stop.to_string() != cold_stop.to_string() || cards == 0 {
            failed.push(format!("{}: warm {} cards ({}), cold {} cards ({})", stem, warm.punched.len(), warm_stop, cold.punched.len(), cold_stop));
        }
    }
    assert!(failed.is_empty(), "{}", failed.join("\n"));
}
