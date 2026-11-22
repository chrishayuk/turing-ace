//! A language server for the instruction tables of the 1945 report, speaking
//! the Language Server Protocol over standard input and output. `ace lsp`
//! runs it.
//!
//! It keeps the text of each open file and answers from `check.rs`: problems
//! as the text changes, what an instruction becomes on the machine and what
//! a table is for, where a table begins and where it is used, the outline of
//! tables, completion, and a formatter that sets the tables out in columns.

use crate::check;
use crate::json::{self, obj, Json};
use std::collections::HashMap;
use std::io::{BufRead, Write};

fn read_message(input: &mut impl BufRead) -> Option<Json> {
    let mut length = None;
    loop {
        let mut line = String::new();
        if input.read_line(&mut line).ok()? == 0 {
            return None;
        }
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            if name.eq_ignore_ascii_case("content-length") {
                length = value.trim().parse::<usize>().ok();
            }
        }
    }
    let mut body = vec![0; length?];
    input.read_exact(&mut body).ok()?;
    json::parse(&String::from_utf8_lossy(&body)).ok()
}

fn send(output: &mut impl Write, message: Json) {
    let mut body = String::new();
    message.write(&mut body);
    let _ = write!(output, "Content-Length: {}\r\n\r\n{}", body.len(), body);
    let _ = output.flush();
}

fn position(line: usize, character: usize) -> Json {
    obj([("line", line.into()), ("character", character.into())])
}

fn range(line: usize, start: usize, end: usize) -> Json {
    obj([("start", position(line, start)), ("end", position(line, end))])
}

fn diagnostics(uri: &str, text: &str) -> Json {
    let items: Vec<Json> = check::check(text)
        .into_iter()
        .map(|d| {
            obj([
                ("range", range(d.span.line, d.span.start, d.span.end)),
                ("severity", if d.error { 1 } else { 2 }.into()),
                ("source", "ace".into()),
                ("message", d.message.into()),
            ])
        })
        .collect();
    obj([
        ("jsonrpc", "2.0".into()),
        ("method", "textDocument/publishDiagnostics".into()),
        ("params", obj([("uri", uri.into()), ("diagnostics", items.into())])),
    ])
}

fn capabilities() -> Json {
    obj([
        ("capabilities", obj([
            // 1: the whole text is sent on every change.
            ("textDocumentSync", 1.into()),
            ("hoverProvider", true.into()),
            ("definitionProvider", true.into()),
            ("referencesProvider", true.into()),
            ("documentSymbolProvider", true.into()),
            ("documentFormattingProvider", true.into()),
            ("documentHighlightProvider", true.into()),
            ("renameProvider", true.into()),
            ("foldingRangeProvider", true.into()),
            ("semanticTokensProvider", obj([
                ("legend", obj([
                    ("tokenTypes", check::TOKEN_TYPES.iter().map(|t| Json::from(*t)).collect::<Vec<_>>().into()),
                    ("tokenModifiers", Vec::new().into()),
                ])),
                ("full", true.into()),
            ])),
            ("completionProvider", obj([("resolveProvider", false.into())])),
        ])),
        ("serverInfo", obj([("name", "ace".into()), ("version", env!("CARGO_PKG_VERSION").into())])),
    ])
}

fn completion(text: &str) -> Json {
    check::completions(text)
        .into_iter()
        .enumerate()
        .map(|(i, (label, kind, detail))| {
            obj([
                ("label", label.into()),
                ("kind", kind.into()),
                ("detail", detail.into()),
                ("sortText", format!("{}{:03}", if kind == 3 { 0 } else { 1 }, i).into()),
            ])
        })
        .collect::<Vec<_>>()
        .into()
}

fn symbols(text: &str) -> Json {
    let width = |line: usize| text.lines().nth(line).map(|l| l.encode_utf16().count()).unwrap_or(0);
    check::tables(text)
        .into_iter()
        .map(|(name, first, last)| {
            obj([
                ("name", name.into()),
                ("kind", 12.into()), // Function
                ("range", obj([("start", position(first, 0)), ("end", position(last, width(last)))])),
                ("selectionRange", range(first, 0, width(first))),
            ])
        })
        .collect::<Vec<_>>()
        .into()
}

/// Tokens as the protocol wants them: five numbers each, positions relative
/// to the token before.
fn semantic_tokens(text: &str) -> Json {
    let mut data = Vec::new();
    let (mut last_line, mut last_start) = (0, 0);
    for (line, start, length, kind) in check::semantic(text) {
        let delta = if line == last_line { start - last_start } else { start };
        data.extend([line - last_line, delta, length, kind, 0].map(Json::from));
        (last_line, last_start) = (line, start);
    }
    obj([("data", data.into())])
}

fn formatting(text: &str) -> Json {
    let new = check::format(text);
    if new == text {
        return Json::Arr(Vec::new());
    }
    // One edit that replaces the whole text.
    let lines = text.split('\n').count();
    let last = text.rsplit('\n').next().unwrap_or("").encode_utf16().count();
    vec![obj([
        ("range", obj([("start", position(0, 0)), ("end", position(lines - 1, last))])),
        ("newText", new.into()),
    ])]
    .into()
}

/// Answer one request. `None` means the method is not one this server knows.
fn answer(method: &str, params: &Json, docs: &HashMap<String, String>) -> Option<Json> {
    let uri = params.get("textDocument").get("uri").str().unwrap_or("");
    let text = docs.get(uri).map(String::as_str).unwrap_or("");
    let line = params.get("position").get("line").num().unwrap_or(0);
    let col = params.get("position").get("character").num().unwrap_or(0);
    let location = |(l, a, b): check::Place| obj([("uri", uri.into()), ("range", range(l, a, b))]);
    Some(match method {
        "initialize" => capabilities(),
        "shutdown" => Json::Null,
        "textDocument/hover" => match check::hover(text, line, col) {
            Some((value, (l, a, b))) => obj([
                ("contents", obj([("kind", "markdown".into()), ("value", value.into())])),
                ("range", range(l, a, b)),
            ]),
            None => Json::Null,
        },
        "textDocument/documentHighlight" => check::references(text, line, col)
            .into_iter()
            .map(|(l, a, b)| obj([("range", range(l, a, b))]))
            .collect::<Vec<_>>()
            .into(),
        "textDocument/rename" => {
            let new = params.get("newName").str().unwrap_or("");
            let places = check::references(text, line, col);
            if places.is_empty() || !crate::notation::is_name(new) {
                Json::Null
            } else {
                let edits: Vec<Json> = places
                    .into_iter()
                    .map(|(l, a, b)| obj([("range", range(l, a, b)), ("newText", new.into())]))
                    .collect();
                obj([("changes", Json::Obj(vec![(uri.to_string(), edits.into())]))])
            }
        }
        "textDocument/foldingRange" => check::tables(text)
            .into_iter()
            .filter(|(_, first, last)| last > first)
            .map(|(_, first, last)| obj([("startLine", first.into()), ("endLine", last.into())]))
            .collect::<Vec<_>>()
            .into(),
        "textDocument/semanticTokens/full" => semantic_tokens(text),
        "textDocument/definition" => {
            check::definition(text, line, col).map(location).unwrap_or(Json::Null)
        }
        "textDocument/references" => {
            check::references(text, line, col).into_iter().map(location).collect::<Vec<_>>().into()
        }
        "textDocument/documentSymbol" => symbols(text),
        "textDocument/completion" => completion(text),
        "textDocument/formatting" => formatting(text),
        _ => return None,
    })
}

/// Serve until the client says `exit` or closes the input.
pub fn serve(input: &mut impl BufRead, output: &mut impl Write) -> Result<(), String> {
    let mut docs: HashMap<String, String> = HashMap::new();
    let mut shutting_down = false;
    while let Some(message) = read_message(input) {
        let method = message.get("method").str().unwrap_or("").to_string();
        let params = message.get("params");
        let id = message.get("id").clone();
        let uri = params.get("textDocument").get("uri").str().unwrap_or("").to_string();
        match method.as_str() {
            "exit" => return if shutting_down { Ok(()) } else { Err("exit without shutdown".into()) },
            "textDocument/didOpen" => {
                let text = params.get("textDocument").get("text").str().unwrap_or("").to_string();
                send(output, diagnostics(&uri, &text));
                docs.insert(uri, text);
            }
            "textDocument/didChange" => {
                if let Some(text) = params.get("contentChanges").items().last().and_then(|c| c.get("text").str()) {
                    send(output, diagnostics(&uri, text));
                    docs.insert(uri, text.to_string());
                }
            }
            "textDocument/didClose" => {
                docs.remove(&uri);
                send(output, diagnostics(&uri, ""));
            }
            // A notification this server has no use for.
            _ if id == Json::Null => {}
            _ => {
                shutting_down |= method == "shutdown";
                let reply = match answer(&method, params, &docs) {
                    Some(result) => obj([("jsonrpc", "2.0".into()), ("id", id), ("result", result)]),
                    None => obj([
                        ("jsonrpc", "2.0".into()),
                        ("id", id),
                        ("error", obj([
                            ("code", Json::Num(-32601.0)),
                            ("message", format!("method not found: {}", method).into()),
                        ])),
                    ]),
                };
                send(output, reply);
            }
        }
    }
    Ok(())
}
