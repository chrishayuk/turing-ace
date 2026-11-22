// The VS Code side of Turing's instruction tables. The work is done by the
// ace program, which the extension carries compiled to WebAssembly and runs
// with the editor's own Node: `ace lsp` is the language server, and the
// commands run the same program on the file being edited.
"use strict";

const vscode = require("vscode");
const path = require("path");
const { execFile } = require("child_process");
const { LanguageClient, TransportKind } = require("vscode-languageclient/node");

/** @type {LanguageClient | undefined} */
let client;
let output;
// The numbers last put in the card reader for each file, offered again.
const lastInput = new Map();

const custom = () => (vscode.workspace.getConfiguration("ace").get("path") || "").trim();

async function startServer(context) {
  const program = custom();
  const builtIn = { module: context.asAbsolutePath(path.join("server", "lsp.js")), transport: TransportKind.stdio };
  const serverOptions = program ? { command: program, args: ["lsp"] } : { run: builtIn, debug: builtIn };
  client = new LanguageClient("ace", "Turing's ACE", serverOptions, { documentSelector: [{ language: "ace" }] });
  try {
    await client.start();
  } catch (error) {
    client = undefined;
    vscode.window.showWarningMessage(`ACE: the language server did not start (${error.message}). Highlighting and snippets still work.`);
  }
}

async function restartServer(context) {
  const old = client;
  client = undefined;
  if (old) await old.stop().catch(() => {});
  await startServer(context);
}

/** Runs `ace COMMAND FILE OPTIONS` and resolves with everything it printed. */
function ace(context, file, command, options) {
  const program = custom();
  const fileArgs = command === "audit" ? [] : [file];
  const [exe, args, env] = program
    ? [program, [command, ...fileArgs, ...options], process.env]
    : [process.execPath, [context.asAbsolutePath(path.join("server", "run.js")), file, command, ...options], { ...process.env, ELECTRON_RUN_AS_NODE: "1" }];
  return new Promise((resolve) => {
    const child = execFile(exe, args, { env, timeout: 120000, maxBuffer: 64 * 1024 * 1024 }, (error, stdout, stderr) => {
      const missing = error && error.code === "ENOENT" ? `could not run "${exe}": check the ace.path setting\n` : "";
      resolve(stdout + stderr + missing);
    });
    // Close standard input: the program must not wait there for cards.
    child.stdin.end();
  });
}

/** Runs a command on the file in the editor and shows what it prints. */
async function run(context, command, options, { input = false } = {}) {
  const editor = vscode.window.activeTextEditor;
  if (!editor || editor.document.languageId !== "ace") {
    vscode.window.showInformationMessage("Open an instruction table (.ace) to run it.");
    return undefined;
  }
  const document = editor.document;
  if (document.isUntitled) {
    vscode.window.showInformationMessage("Save the file first.");
    return undefined;
  }
  if (document.isDirty) await document.save();
  const file = document.uri.fsPath;
  const all = [...options];
  // A job with an instruction of type P reads cards: ask for the numbers.
  const code = document.getText().replace(/#.*$/gm, "");
  if (input && /(^|[\s;.])P\s*,\s*DL/m.test(code)) {
    const numbers = await vscode.window.showInputBox({
      prompt: "Numbers for the card reader, twelve to a card, separated by spaces or commas",
      value: lastInput.get(file) || "",
    });
    if (numbers === undefined) return undefined;
    lastInput.set(file, numbers);
    all.push("--input", numbers.trim().split(/[\s,]+/).filter(Boolean).join(","));
  }
  output.clear();
  output.show(true);
  output.appendLine(`ace ${[command, ...(command === "audit" ? [] : [path.basename(file)]), ...all].join(" ")}`);
  const printed = await ace(context, file, command, all);
  output.append(printed);
  return printed;
}

async function activate(context) {
  output = vscode.window.createOutputChannel("Turing's ACE");
  const command = (name, ...how) => vscode.commands.registerCommand(name, () => run(context, ...how));
  context.subscriptions.push(
    output,
    command("ace.run", "run", ["--time"], { input: true }),
    command("ace.trace", "run", ["--trace", "--time"], { input: true }),
    command("ace.tables", "tables", []),
    command("ace.audit", "audit", []),
    vscode.commands.registerCommand("ace.restart", () => restartServer(context)),
    vscode.workspace.onDidChangeConfiguration((event) => {
      if (event.affectsConfiguration("ace.path")) restartServer(context);
    }),
  );
  await startServer(context);
}

function deactivate() {
  return client ? client.stop() : undefined;
}

module.exports = { activate, deactivate };
