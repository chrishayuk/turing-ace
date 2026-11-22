# turing-ace

The Automatic Computing Engine as Alan Turing proposed it in 1945: the
machine, the notation of its instruction tables, and the tables themselves,
run.

Turing's report *Proposed Electronic Calculator*, written for the National
Physical Laboratory in 1945, describes a complete computer: mercury delay
lines, 32 temporary storages, the logical control and the arithmetic part
down to the valves, and a cost of £11,200. Section 13 shows how it would be
programmed, with two instruction tables printed in full, INDEXIN and CALPOL,
and the descriptions of nine more. The machine was never built as described;
the Pilot ACE of 1950 was a different design. Here it is simulated, and the
tables are run on it.

```
$ ace audit
...
2. INDEXIN on the machine: fetch the 1945 kept in DL 3,5, whose position is in TS 27

  the table as printed        FAILS   TS 28 = 0; stopped at INDEXIN 11 after 46 instructions: waiting for a card that is not in the reader
  the cards of Fig. 38        FAILS   TS 28 = 0; stopped at INDEXIN 11 after 46 instructions: waiting for a card that is not in the reader
  the table, all corrections  works   TS 28 = 1945; came back after 71 instructions
```

## The notation

A job is a text of instruction tables in what the report calls *popular
form*. This is INDEXIN as section 13 prints it:

```
INDEXIN
  1   Q, 0000,0100,0000,0000     2
  2   TS 6 - TS 2                3
  3   ADD 'A'                    4
  4   ROTATE 16                  5
  5   TS 4 - TS 6                6
  6   TS 6 - TS 9                7
  7   TS 27 - TS 6               8
  8   TS 6 - TS 10               9
  9   OR                        10
 10   TS 8 - TS 6               11
 11   B,1, INDEXIN 11
 12   TS 6 - TS 28              13
 13   B, BURY
```

The first column is the detail figure, the last the number of the next
instruction. `TS 6 - TS 2` moves the content of TS 6 into TS 2. `B, X` passes
control to X; `B, 1` carries out the instruction in TS 6. CALPOL's style, several
instructions to a line with semicolons, is read too.

`ace` assembles a job the way section 7 describes doing it with cards and a
collator: the tables a job uses are sorted by name and detail figure,
numbered consecutively in the binary scale, and every reference filled in.
That gives *machine form*, one 32-digit minor cycle per instruction, laid
out in the delay lines with consecutive instructions two minor cycles apart,
as section 11 requires.

## Install

You need [Rust](https://rustup.rs):

```
cargo install --git https://github.com/chrishayuk/turing-ace
```

That puts `ace` in `~/.cargo/bin`. Then:

```
ace run examples/answer.ace
ace run examples/gcd.ace --input 1071,462
ace run examples/polynomial.ace --time
```

For Visual Studio Code, build the extension in
[`editors/vscode`](editors/vscode) and install it (you need Node.js):

```
cd editors/vscode && npm install && npm run package
code --install-extension turing-ace-language-0.1.0.vsix
```

Open any `.ace` file for colours, hover help showing the machine form of
each instruction, problems as you type, and commands to run the job, show
its machine form and run the audit. The extension carries the program
compiled to WebAssembly. For Neovim, Helix and other editors see
[`editors/README.md`](editors/README.md).

Or try it with nothing installed, in the [playground](https://chrishayuk.com/demos/turing-ace).

## Using it

```
ace run FILE [--input N,N,...] [--cards numbers|listing|holes] [--trace] [--time]
ace deck FILE [--holes]
ace tables FILE [--emit listing|cards|digits]
ace library [NAME]
ace audit
ace errata
ace decisions
ace check FILE
ace format FILE
ace lsp
cargo test
```

`run` loads the job and runs it until an instruction passes control to
itself. Numbers given with `--input` go onto cards in the reader, twelve to
a card; punched cards come out as twelve numbers, as the line a tabulator
would list, or as rows of holes. `--trace` prints every instruction obeyed
with the minor cycle it was found in, and `--time` the machine time.

`deck` turns a job into the cards that load it into an empty machine, and
`run --cold` starts from them. The report says the machine starts with an
initial order of "0 throughout", which reads a card into DL 0, and that "the
first few orders obeyed will probably be to take in a few more cards". Each
card of the deck carries two words and the orders that put them in place;
that format is ours. Loading takes far longer than running:

```
$ ace run examples/answer.ace --cold --time
 000000042
127 cards of the deck read from cold
2430 instructions in 2441105 minor cycles: 78.1 seconds of machine time
```

`tables` shows the machine form, and `--emit cards` the 80-column instruction
cards of section 7: popular name, detail figure, instruction, repeat of
destination, genuine input and job number.

## The machine

What is simulated is sections 11 and 12 and Fig. 19, at the level of whole
instructions: 200 delay lines of 32 minor cycles, 32 temporary storages,
CD and CI, the card reader and punch, instructions of type A and B, and the
ten types of operation K to T. Valves and pulses are not simulated. Time is
kept in minor cycles, because a word in a delay line can only be read as it
comes round, and that is most of what the machine spends its time on:

```
$ ace run tutorial/jobs/placed.ace --time
8 instructions in 16 minor cycles: 512 microseconds of machine time
$ ace run tutorial/jobs/unplaced.ace --time
8 instructions in 144 minor cycles: 4608 microseconds of machine time
```

Both jobs fetch four numbers. In the second each number has just gone by, so
the machine waits for it, and while it waits the next instruction goes by
too.

## Examples

`examples/` holds 16 jobs, from reading and punching a card to factorials,
Fibonacci numbers, Euclid's algorithm by subtraction, an instruction built
and obeyed in TS 6, and CALPOL working out x³ - 2x + 5. Each has an `.out`
file with what it punches and, where it reads cards, an `.in` file.
`cargo test` runs them all.

## The library

| table | |
|-------|---|
| INDEXIN | printed in section 13 |
| CALPOL | printed in section 13 |
| BURY, UNBURY | described in sections 6 and 13; the tables are ours |
| DISCRIM | described and outlined in section 13; the table is ours |
| PLUSIND, TRANS 45 | described in section 13; the tables are ours |
| MULTIP, ADD | described in section 13; ours do whole numbers only |
| BINDEC | described in section 13; ours does whole numbers only |

A table a job names and does not contain comes from the library. Every
library table carries the report's general description as its comment, and
those that are ours say so. `ace library NAME` prints one.

## What running it found

INDEXIN does not work as printed. It needs three corrections, and leaving
out any one of them stops it working:

| erratum | problem | known before? |
|---|---|---|
| `literal` | Line 1 makes the code of type K, which stores, where type L, which fetches, is meant | Carpenter and Doran (1986), note 17 |
| `half` | ROTATE 16 moves the code into TS 5; line 5 takes TS 4 | not that we found |
| `return` | The table ends `B, BURY` instead of `B, UNBURY` | Carpenter and Doran (1986), note 18 |

Fig. 38 prints INDEXIN a second time, as the 13 cards that would be punched
for it. Three of them do not agree with the table: the card for ADD 'A' also
sets NOR, the card for OR is the code for AND, and the card for
`TS 8 - TS 6` is `TS 4 - TS 6`. Run as the cards have it, INDEXIN fails the
same way.

CALPOL as printed never stops. With five corrections it gives the right
value, called once or twice in a row:

| erratum | problem |
|---|---|
| `clear` | Line 1 clears TS 22, 23, the result, not TS 20, 21, the running product: a second call starts from the last product |
| `inputs` | AND is given its arguments in TS 2 and TS 3; the logical operations read TS 9 and TS 10 |
| `branch` | The two ways on are made with type Q, which fills only digits 17-32 and so cannot make an instruction; and CALPOL 40 is not the exit |
| `carry` | After DISCRIM the chosen instruction is in TS 24, not TS 6 (this one depends on how BURY and UNBURY are written) |
| `contents` | The words printed for DL 1,14 and DL 1,15 do not name DL 3,1 |

Carpenter and Doran published an analysis of CALPOL's errors in 1977 (*The
other Turing machine*). This project has not been able to read that paper,
so none of the CALPOL corrections is claimed to be new. In their 1986
edition they wrote that section 13 "contains numerous coding errors".

DISCRIM is described and outlined, not printed. Followed step by step, the
outline answers "no digit 1" for any word whose digit 32 is 1: subtracting a
negative number from zero gives a positive one. The library's DISCRIM adds
TS 8 as well as subtracting it (`sign`). `ace errata` lists all nine
corrections, and `--errata none` runs the tables as printed.

## Where the report disagrees with itself

Some things the report says twice, differently: which digits select a
logical operation, which storages the logical operations read, which is the
multiplicand, where a given multiplier sits, where ROTATE finds its amount,
where type B leaves the position it came from, how minor cycles are numbered.
`ace decisions` lists each with both readings and the one taken. Several
were noted by Carpenter and Doran in 1986; those are credited.

The report never sets TS 31, where BURY keeps its list, or TS 7, which
INDEXIN's `ROTATE 16` needs to hold 16. The tool sets both before a job
starts and says so.

## Tutorial

`tutorial/` is a 12-chapter tutorial, starting at
[`01-introduction.md`](tutorial/01-introduction.md): cards, storage,
arithmetic, logic, control, subsidiaries, choices, decimal output, time,
Turing's tables, and a reference. Every job and command in it is followed by
what it prints, and

```
python3 tutorial/check_tutorial.py
```

runs them all and compares.

## Playground

`web/` runs the same program in a browser, compiled unchanged for WASI
(`wasm32-wasip1`). It opens on the machine room (`web/console.js`): the job
as a deck in the card reader's hopper, a starting switch, the mercury delay
lines with the job going round in them, CD, CI and the temporary storages as
lamps, and the punch and the tabulator's listing. Close the switch and the
empty machine reads its first card with the order of zeros, loads the rest,
and runs. It can be watched slowed down or at the machine's own speed, at
which loading a job takes over a minute. Every detail it shows is quoted
from the report at the top of the file, and what is not from the report is
said so.

```
node web/build.mjs                  # builds web/ace.wasm and web/examples.json
node web/test.mjs                   # runs every example through the wasm build
python3 -m http.server -d web 8000  # then open http://localhost:8000
```

## Sources

The report and its figures are not in this repository.

* A. M. Turing, *Proposed Electronic Calculator*, National Physical
  Laboratory, 1945. Transcription:
  <https://en.wikisource.org/wiki/Proposed_Electronic_Calculator>.
  Facsimile and figures: the Turing Archive for the History of Computing,
  <https://www.alanturing.net/ace>.
* B. E. Carpenter and R. W. Doran (eds.), *A. M. Turing's ACE Report of 1946
  and Other Papers*, MIT Press, 1986.
* B. E. Carpenter and R. W. Doran, "The other Turing machine", *The Computer
  Journal* 20(3), 1977, pp. 269-279.

## Earlier work

There are emulators of the Pilot ACE, starting with Donald Davies's. We
found no executable reconstruction of the machine of the 1945 report, which
is not to say there is none.

## Licence

Apache License 2.0 - see [LICENSE](LICENSE).
