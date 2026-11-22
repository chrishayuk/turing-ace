# Turing's ACE for VS Code

Editor support for the instruction tables of Alan Turing's 1945 report,
*Proposed Electronic Calculator*, run on a simulation of the machine the
report describes.

- **Highlighting.** Table names, detail figures, the storages `TS 6` and
  `DL 1,14`, the type letters `B`, `Q`, `K` … `T`, the arithmetical and
  logical operations, the signals of type S and minor cycles written out
  in binary (`0000,0100,0000,0000`) each have their own colour.
- **Problems as you type.** Errors for what will not assemble, such as a
  table that does not exist, a detail figure that is not there, or a
  position the machine does not have. Warnings for what assembles but does
  not do what it says, such as a transfer into TS 1, TS 4, TS 5 or TS 8,
  which take nothing from TS 6.
- **The machine behind an instruction.** Rest the pointer on an
  instruction to see its position in the delay lines, its 32 digits and
  what it does. A transfer between two storages shows both of the
  instructions it becomes. Rest it on a table's name to see the table's
  general description, and on `TS 27` to see what that storage is for.
- **Tables.** Go to a table's definition, or to the instruction a detail
  figure names. Find where a table is used, rename it, fold it, and see the
  tables of a file in the outline. Completion offers the tables of the
  library: INDEXIN, BURY, UNBURY, DISCRIM, PLUSIND, TRANS, MULTIP, ADD,
  CALPOL and BINDEC.
- **Run it.** Cmd+Enter (Ctrl+Enter) runs the job on the machine and shows
  the cards it punches and the machine time it took. A job that reads cards
  asks for the numbers to put in the reader. The command palette also has
  *Run and Trace Every Instruction*, *Show the Machine Form* and *Audit the
  Report's Tables*.

## Install

Download `turing-ace-language-0.1.0.vsix` from the
[latest release](https://github.com/chrishayuk/turing-ace/releases/latest)
and install it:

```
code --install-extension turing-ace-language-0.1.0.vsix
```

That is all it needs. The extension carries the `ace` program compiled to
WebAssembly and runs it with the editor's own Node. To use a program you
built yourself, set `ace.path` in the settings.

*Format Document* sets the tables out in columns: detail figures on the
right of the first four columns, instructions after them, the number of
the next instruction last. It is not run on save unless you ask:

```json
"[ace]": { "editor.formatOnSave": true }
```

## Build it yourself

```
cd editors/vscode
npm install
npm run package          # builds the WebAssembly, bundles, writes the .vsix
```

## How it works

`ace lsp` is a language server, written in Rust with the rest of the tools
and with no dependencies. `server/lsp.js` runs it from `server/ace.wasm`,
and `server/run.js` runs the other commands the same way. Any editor that
speaks the Language Server Protocol can use the server; see
[`../README.md`](../README.md).

## Tests

```
npm test                 # the grammar, and the two server scripts under Node
npm run test:vscode      # the whole extension inside a real VS Code
```

The second downloads a copy of VS Code into `.vscode-test/` the first time.
It opens a job and checks the hover, go to definition, outline,
completion, a warning as you type, formatting, colours and each command.
