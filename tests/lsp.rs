//! Talks to `ace lsp` as an editor would, over its standard input and output.

use ace::json::{self, obj, Json};
use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

struct Client {
    child: Child,
    to: ChildStdin,
    from: BufReader<ChildStdout>,
    next: usize,
}

const URI: &str = "file:///tmp/fetch.ace";

impl Client {
    fn start() -> Client {
        let mut child = Command::new(env!("CARGO_BIN_EXE_ace"))
            .arg("lsp")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let to = child.stdin.take().unwrap();
        let from = BufReader::new(child.stdout.take().unwrap());
        Client { child, to, from, next: 1 }
    }

    fn send(&mut self, message: Json) {
        let mut body = String::new();
        message.write(&mut body);
        write!(self.to, "Content-Length: {}\r\n\r\n{}", body.len(), body).unwrap();
        self.to.flush().unwrap();
    }

    fn read(&mut self) -> Json {
        let mut length = 0;
        loop {
            let mut line = String::new();
            assert!(self.from.read_line(&mut line).unwrap() > 0, "the server closed its output");
            match line.trim_end().split_once(": ") {
                Some(("Content-Length", n)) => length = n.parse().unwrap(),
                _ if line.trim_end().is_empty() => break,
                _ => {}
            }
        }
        let mut body = vec![0; length];
        self.from.read_exact(&mut body).unwrap();
        json::parse(std::str::from_utf8(&body).unwrap()).unwrap()
    }

    fn notify(&mut self, method: &str, params: Json) {
        self.send(obj([("jsonrpc", "2.0".into()), ("method", method.into()), ("params", params)]));
    }

    fn request(&mut self, method: &str, params: Json) -> Json {
        let id = self.next;
        self.next += 1;
        self.send(obj([
            ("jsonrpc", "2.0".into()),
            ("id", id.into()),
            ("method", method.into()),
            ("params", params),
        ]));
        let reply = self.read();
        assert_eq!(reply.get("id").num(), Some(id), "{:?}", reply);
        reply
    }

    fn at(line: usize, character: usize) -> Json {
        obj([
            ("textDocument", obj([("uri", URI.into())])),
            ("position", obj([("line", line.into()), ("character", character.into())])),
        ])
    }
}

#[test]
fn an_editor_session() {
    let mut c = Client::start();
    let init = c.request("initialize", obj([("capabilities", obj([]))]));
    let caps = init.get("result").get("capabilities");
    assert_eq!(caps.get("hoverProvider"), &Json::Bool(true));
    assert_eq!(caps.get("textDocumentSync").num(), Some(1));
    assert_eq!(init.get("result").get("serverInfo").get("name").str(), Some("ace"));
    c.notify("initialized", obj([]));

    // A clean job has no problems.
    let text = std::fs::read_to_string("examples/fetch.ace").unwrap();
    c.notify(
        "textDocument/didOpen",
        obj([("textDocument", obj([("uri", URI.into()), ("text", text.as_str().into())]))]),
    );
    let published = c.read();
    assert_eq!(published.get("method").str(), Some("textDocument/publishDiagnostics"));
    assert_eq!(published.get("params").get("diagnostics").items().len(), 0, "{:?}", published);

    // The lines of fetch.ace, counting from 0.
    let lines: Vec<&str> = text.lines().collect();
    let line_of = |s: &str| lines.iter().position(|l| l.contains(s)).unwrap();
    let (call, transfer, stop) = (line_of("B, INDEXIN"), line_of("TS 28 - DL 2,1"), line_of("B, FETCH 6"));
    let column = |line: usize, s: &str| lines[line].find(s).unwrap();

    // Resting on a transfer between storages shows the two instructions it becomes.
    let hover = c.request("textDocument/hover", Client::at(transfer, column(transfer, "- DL")));
    let shown = hover.get("result").get("contents").get("value").str().unwrap();
    assert!(shown.contains("two stages") && shown.contains("On the machine:"), "{}", shown);
    assert!(shown.contains("Type M: pass the content of TS 28 into TS 6."), "{}", shown);

    // On a table of the library: its general description.
    let hover = c.request("textDocument/hover", Client::at(call, column(call, "INDEXIN") + 1));
    let shown = hover.get("result").get("contents").get("value").str().unwrap();
    assert!(shown.contains("**INDEXIN**") && shown.contains("General Description"), "{}", shown);

    // FETCH 6 leads to instruction 6 of FETCH, on the same line.
    let def = c.request("textDocument/definition", Client::at(stop, column(stop, "FETCH") + 1));
    assert_eq!(def.get("result").get("range").get("start").get("line").num(), Some(stop));
    let refs = c.request("textDocument/references", Client::at(stop, column(stop, "FETCH") + 1));
    assert_eq!(refs.get("result").items().len(), 2);

    // Renaming a table changes its head and every use.
    let mut rename = Client::at(stop, column(stop, "FETCH") + 1);
    if let Json::Obj(fields) = &mut rename {
        fields.push(("newName".into(), "GET".into()));
    }
    let renamed = c.request("textDocument/rename", rename);
    assert_eq!(renamed.get("result").get("changes").get(URI).items().len(), 2);
    let marked = c.request("textDocument/documentHighlight", Client::at(stop, column(stop, "FETCH")));
    assert_eq!(marked.get("result").items().len(), 2);
    let folds = c.request("textDocument/foldingRange", Client::at(0, 0));
    assert_eq!(folds.get("result").items()[0].get("endLine").num(), Some(stop));

    // The first token is the comment on line 0: kind 0, the whole line long.
    let colours = c.request("textDocument/semanticTokens/full", Client::at(0, 0));
    let data: Vec<usize> = colours.get("result").get("data").items().iter().map(|n| n.num().unwrap()).collect();
    assert_eq!(data.len() % 5, 0);
    assert_eq!(&data[..5], [0, 0, lines[0].chars().count(), 0, 0]);

    let outline = c.request("textDocument/documentSymbol", Client::at(0, 0));
    let names: Vec<&str> = outline.get("result").items().iter().map(|s| s.get("name").str().unwrap()).collect();
    assert_eq!(names, ["FETCH"]);

    let offered = c.request("textDocument/completion", Client::at(0, 0));
    let labels: Vec<&str> = offered.get("result").items().iter().filter_map(|i| i.get("label").str()).collect();
    for want in ["INDEXIN", "DISCRIM", "CALPOL", "FETCH", "ADD 'A'", "OR"] {
        assert!(labels.contains(&want), "{} not offered: {:?}", want, labels);
    }

    // Formatting sets the table out in columns.
    let edits = c.request("textDocument/formatting", Client::at(0, 0));
    let new = edits.get("result").items()[0].get("newText").str().unwrap();
    assert!(new.contains("\n   3   TS 28 - DL 2,1\n"), "{}", new);
    assert!(new.contains("\nTS 27    DL 3,5\n"), "{}", new);

    // A change that names a table nobody has is an error on that line, and a
    // transfer into TS 4 is a warning.
    c.notify(
        "textDocument/didChange",
        obj([
            ("textDocument", obj([("uri", URI.into())])),
            ("contentChanges", vec![obj([("text", "JOB\n  1   TS 6 - TS 4\n  2   B, NOWHERE\n".into())])].into()),
        ]),
    );
    let published = c.read();
    let problems = published.get("params").get("diagnostics").items();
    assert_eq!(problems.len(), 2, "{:?}", problems);
    assert_eq!(problems[0].get("severity").num(), Some(2));
    assert_eq!(problems[0].get("range").get("start").get("line").num(), Some(1));
    assert_eq!(problems[1].get("severity").num(), Some(1));
    assert_eq!(problems[1].get("range").get("start").get("line").num(), Some(2));
    assert!(problems[1].get("message").str().unwrap().contains("NOWHERE"));

    let unknown = c.request("textDocument/typeDefinition", Client::at(0, 0));
    assert_eq!(unknown.get("error").get("message").str(), Some("method not found: textDocument/typeDefinition"));

    assert_eq!(c.request("shutdown", Json::Null).get("result"), &Json::Null);
    c.notify("exit", Json::Null);
    assert!(c.child.wait().unwrap().success());
}
