// Starts the language server for Turing's instruction tables: the Rust
// program compiled for WASI, speaking the Language Server Protocol on
// standard input and output.
"use strict";

const path = require("path");
const { run } = require("./wasi.js");

process.exitCode = run(path.join(__dirname, "ace.wasm"), { args: ["lsp"] });
