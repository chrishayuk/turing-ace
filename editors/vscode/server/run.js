// Runs one ace command on a file:  node run.js FILE run --input 1071,462
// The program compiled for WASI has no file system here, so the file is read
// by this script and handed over on standard input, where `-` stands for it.
"use strict";

const fs = require("fs");
const path = require("path");
const { run } = require("./wasi.js");

const [file, command, ...rest] = process.argv.slice(2);
if (!file || !command) {
  console.error("usage: run.js FILE COMMAND [OPTIONS]");
  process.exit(2);
}
let tables;
try {
  tables = fs.readFileSync(file, "utf8");
} catch (error) {
  console.error(`ace: ${file}: ${error.message}`);
  process.exit(2);
}
// After the tables comes a line of four dashes and then the numbers for the
// card reader, which arrive with --input.
const args = command === "audit" ? [command, ...rest] : [command, "-", ...rest];
process.exitCode = run(path.join(__dirname, "ace.wasm"), {
  args,
  prefix: Buffer.from(tables + "\n----\n"),
});
