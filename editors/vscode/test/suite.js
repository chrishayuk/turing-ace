// Runs inside VS Code, started by run-vscode.js.
const assert = require("node:assert");
const path = require("node:path");
const vscode = require("vscode");

const until = async (what, test) => {
  for (let i = 0; i < 150; i++) {
    const value = await test();
    if (value) return value;
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  throw new Error(`timed out waiting for ${what}`);
};

exports.run = async () => {
  const folder = vscode.workspace.workspaceFolders[0].uri.fsPath;
  const uri = vscode.Uri.file(path.join(folder, "job.ace"));
  const document = await vscode.workspace.openTextDocument(uri);
  assert.strictEqual(document.languageId, "ace");
  const editor = await vscode.window.showTextDocument(document);

  // Resting on a transfer shows the two instructions it becomes. Waiting for
  // the hover also waits for the server to start.
  const shown = await until("hover", async () => {
    const hovers = await vscode.commands.executeCommand("vscode.executeHoverProvider", uri, new vscode.Position(4, 12));
    const text = hovers.flatMap((h) => h.contents).map((c) => c.value ?? c).join("\n");
    return text.includes("On the machine:") ? text : null;
  });
  assert.ok(shown.includes("two stages"), shown);
  assert.strictEqual(vscode.languages.getDiagnostics(uri).length, 0);

  // JOB 6 leads to the table's instruction 6.
  const places = await vscode.commands.executeCommand("vscode.executeDefinitionProvider", uri, new vscode.Position(6, 11));
  assert.strictEqual((places[0].range ?? places[0].targetRange).start.line, 6);

  const outline = await vscode.commands.executeCommand("vscode.executeDocumentSymbolProvider", uri);
  assert.deepStrictEqual(outline.map((s) => s.name), ["JOB"]);

  // The tables of the library come from the server and the snippets from
  // the extension; snippets are loaded a moment after the file opens.
  const labels = await until("completions", async () => {
    const offered = await vscode.commands.executeCommand("vscode.executeCompletionItemProvider", uri, new vscode.Position(7, 0));
    const found = offered.items.map((i) => i.label.label ?? i.label);
    return found.includes("INDEXIN") && found.includes("table") ? found : null;
  });
  assert.ok(labels.includes("DISCRIM"));

  // A transfer into TS 4 runs, but does nothing: a warning on that line.
  await editor.edit((edit) => edit.insert(new vscode.Position(7, 0), "  7   TS 6 - TS 4\n"));
  const warned = await until("the warning", () => {
    const found = vscode.languages.getDiagnostics(uri);
    return found.length === 1 ? found : null;
  });
  assert.strictEqual(warned[0].range.start.line, 7);
  assert.strictEqual(warned[0].severity, vscode.DiagnosticSeverity.Warning);
  await vscode.commands.executeCommand("workbench.action.files.revert");

  // Formatting sets the table out in columns.
  await vscode.commands.executeCommand("editor.action.formatDocument");
  assert.ok(editor.document.getText().includes("   3   TS 28 - DL 2,1\n"), editor.document.getText());
  await vscode.commands.executeCommand("workbench.action.files.revert");

  // The run commands, with the program the extension carries.
  assert.strictEqual(await vscode.commands.executeCommand("ace.run").then((s) => s.split("\n")[0]), "42");
  const trace = await vscode.commands.executeCommand("ace.trace");
  assert.ok(trace.includes("INDEXIN 11") && trace.includes("B, 1, INDEXIN 11"), trace.slice(0, 400));
  const listing = await vscode.commands.executeCommand("ace.tables");
  assert.ok(listing.includes("JOB") && listing.includes("DL 3,5"), listing);
  const audit = await vscode.commands.executeCommand("ace.audit");
  assert.ok(audit.includes("3 of the 13 cards"), audit);

  // Semantic colours come from the server.
  const colours = await vscode.commands.executeCommand("vscode.provideDocumentSemanticTokens", uri);
  assert.ok(colours.data.length >= 5 * 10, "semantic tokens");

  console.log("extension: hover, definition, outline, completion, warnings, formatting, colours and the run commands verified in VS Code");
};
