// Checks the highlighting grammar by tokenising real lines with the same
// engine VS Code uses:  node test/grammar.test.js
const assert = require("node:assert");
const fs = require("node:fs");
const path = require("node:path");
const oniguruma = require("vscode-oniguruma");
const { Registry, parseRawGrammar } = require("vscode-textmate");

const root = path.join(__dirname, "..");
const wasm = fs.readFileSync(require.resolve("vscode-oniguruma/release/onig.wasm")).buffer;

(async () => {
  await oniguruma.loadWASM(wasm);
  const registry = new Registry({
    onigLib: Promise.resolve({
      createOnigScanner: (patterns) => new oniguruma.OnigScanner(patterns),
      createOnigString: (text) => new oniguruma.OnigString(text),
    }),
    loadGrammar: async () => {
      const file = path.join(root, "syntaxes/ace.tmLanguage.json");
      return parseRawGrammar(fs.readFileSync(file, "utf8"), file);
    },
  });
  const grammar = await registry.loadGrammar("source.ace");

  // The scope of each piece of a line, without the `source.ace` every piece has.
  const scopes = (line) =>
    grammar.tokenizeLine(line, null).tokens
      .map((t) => [line.slice(t.startIndex, t.endIndex), t.scopes.slice(1).join(" ").replace(/\.ace/g, "")])
      .filter(([text]) => text.trim());
  const expect = (line, want) => assert.deepStrictEqual(scopes(line), want, line);

  expect("INDEXIN", [["INDEXIN", "entity.name.function.table"]]);
  expect("CALPOL.", [["CALPOL", "entity.name.function.table"], [".", "punctuation.terminator"]]);
  expect("ADD.", [["ADD", "entity.name.function.table"], [".", "punctuation.terminator"]]);
  // A word of the notation alone on a line is an instruction, not a table.
  expect("      OR", [["OR", "keyword.operator.logical"]]);
  expect("  1   Q, 0000,0100,0000,0000     2", [
    ["1", "entity.name.label.figure"],
    ["Q", "keyword.other.type"],
    [",", "punctuation.separator"],
    ["0000,0100,0000,0000", "constant.numeric.binary"],
    ["2", "constant.numeric"],
  ]);
  expect("  2   TS 6 - TS 2     3   # through TS 6", [
    ["2", "entity.name.label.figure"],
    ["TS", "variable.language.storage"],
    ["6", "variable.other.storage"],
    ["-", "keyword.operator.transfer"],
    ["TS", "variable.language.storage"],
    ["2", "variable.other.storage"],
    ["3", "constant.numeric"],
    ["# through TS 6", "comment.line.number-sign"],
  ]);
  expect("  3   ADD 'A'", [
    ["3", "entity.name.label.figure"],
    ["ADD", "keyword.operator.arithmetic"],
    ["'A'", "string.quoted.single"],
  ]);
  expect(" 11   B,1, INDEXIN 11", [
    ["11", "entity.name.label.figure"],
    ["B", "keyword.control.jump"],
    [",", "punctuation.separator"],
    ["1", "constant.numeric"],
    [",", "punctuation.separator"],
    ["INDEXIN", "entity.name.function.table"],
    ["11", "constant.numeric"],
  ]);
  expect("CALPOL 1. CLEAR TS 22, 23; DL 1, 14 - TS 27;", [
    ["CALPOL", "entity.name.function.table"],
    ["1", "entity.name.label.figure"],
    [".", "punctuation.terminator"],
    ["CLEAR", "keyword.operator.arithmetic"],
    ["TS", "variable.language.storage"],
    ["22", "variable.other.storage"],
    [",", "punctuation.separator"],
    ["23", "variable.other.storage"],
    [";", "punctuation.terminator"],
    ["DL", "variable.language.storage"],
    ["1", "variable.other.storage"],
    [",", "punctuation.separator"],
    ["14", "variable.other.storage"],
    ["-", "keyword.operator.transfer"],
    ["TS", "variable.language.storage"],
    ["27", "variable.other.storage"],
    [";", "punctuation.terminator"],
  ]);
  expect("DL 3,29  -2", [
    ["DL", "variable.language.storage"],
    ["3", "variable.other.storage"],
    [",", "punctuation.separator"],
    ["29", "variable.other.storage"],
    ["-2", "constant.numeric"],
  ]);
  expect("      S, POS, CANCEL, SELF, GIV, 10", [
    ["S", "keyword.other.type"],
    [",", "punctuation.separator"],
    ["POS", "variable.parameter.signal"],
    [",", "punctuation.separator"],
    ["CANCEL", "variable.parameter.signal"],
    [",", "punctuation.separator"],
    ["SELF", "variable.parameter.signal"],
    [",", "punctuation.separator"],
    ["GIV", "variable.parameter.signal"],
    [",", "punctuation.separator"],
    ["10", "constant.numeric"],
  ]);

  // Every example and every table: nothing left without a scope.
  let lines = 0;
  for (const dir of ["examples", "tables"]) {
    const folder = path.join(root, "../..", dir);
    for (const file of fs.readdirSync(folder).filter((f) => f.endsWith(".ace"))) {
      for (const line of fs.readFileSync(path.join(folder, file), "utf8").split("\n")) {
        lines++;
        for (const [text, scope] of scopes(line)) {
          assert.ok(scope, `${file}: no scope for "${text}" in: ${line}`);
        }
      }
    }
  }
  console.log(`grammar: 10 lines checked piece by piece, ${lines} lines of real tables fully scoped`);
})().catch((error) => {
  console.error(error.message);
  process.exit(1);
});
