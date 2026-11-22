# Turing's instruction tables in your editor

Two pieces give an editor an understanding of the notation:

- **`ace lsp`**, a language server. It is a command of the `ace` program
  itself, so there is nothing else to build, and it works with any editor
  that speaks the Language Server Protocol.
- **A Visual Studio Code extension** in [`vscode`](vscode), which carries
  the program and adds syntax highlighting, snippets and run commands. Its
  [README](vscode/README.md) lists the features.

Both are built on [`src/check.rs`](../src/check.rs), the same code behind
`ace check`, and that is built on the assembler the `run` command uses. A
problem the editor shows is one the machine would meet.

## The language server

```sh
cargo install --git https://github.com/chrishayuk/turing-ace
```

or, in a clone of this repository, `cargo build --release`. The server is
then `ace lsp`. It talks on standard input and output.

It provides diagnostics, hover, go to definition, references, document
highlights, rename, completion, document symbols, folding ranges,
formatting and semantic tokens. Semantic tokens mean an editor with no
grammar for the notation still gets highlighting.

Hover is where the machine shows through. On an instruction it gives the
position the instruction will have in the delay lines, its 32 digits,
what it does and how long it takes; on a table's name, the table's general
description and, for a table of the report, the corrections applied to
it; on a temporary storage, the duty section 12 of the report gives it.

### Neovim (0.11 or later)

```lua
vim.filetype.add({ extension = { ace = "ace" } })
vim.lsp.config("ace", {
  cmd = { "/path/to/ace", "lsp" },
  filetypes = { "ace" },
})
vim.lsp.enable("ace")
```

### Helix

In `languages.toml`:

```toml
[language-server.ace-lsp]
command = "/path/to/ace"
args = ["lsp"]

[[language]]
name = "ace"
scope = "source.ace"
file-types = ["ace"]
comment-token = "#"
language-servers = ["ace-lsp"]
```

These two configurations follow each editor's documented format but have
not been run here. The server itself is tested over standard input and
output by `tests/lsp.rs` and `vscode/test/server.test.js`.

## Checking a job from the command line

```sh
ace check job.ace
```

prints one line per problem, as `file:line:column: kind: message`, and
exits with status 1 if any is an error. `ace format job.ace` prints the
tables set out in columns.

## In a web page

The playground runs the same server in a browser worker, from the same
`ace.wasm` the extension carries.
