# Reference

## A job

A text of tables. A table is its name on a line of its own, in capitals,
then its instructions. The first table in the text is where the job starts.
Tables named in a job and not in it come from the library. `#` begins a
comment; comments straight above a table are its general description, and
editors show them.

A line that names a storage and then gives a content sets it before the
start: `DL 3,5  1945`, `TS 27  DL 3,5`, `DL 1,14  L, DL 3,1`.

## Instructions

| popular form | type | does |
|--------------|------|------|
| `B, X 7` | B | go on at X 7; the position left goes to TS 1 |
| `B, 1` | B | carry out the instruction in TS 6 |
| `B, 1, X 7` | B | the same, then go on after X 7 if it was of type A |
| `TS 6 - DL 3,5` or `K, DL 3,5` | K | TS 6 into a minor cycle |
| `DL 3,5 - TS 6` or `L, DL 3,5` | L | a minor cycle into TS 6 |
| `TS 9 - TS 6` or `M, TS 9` | M | a TS into TS 6 |
| `TS 6 - TS 9` or `N, TS 9` | N | TS 6 into a TS |
| `X - Y` | | any transfer: two instructions, through TS 6 |
| `O, DL 10` | O | punch the first 12 minor cycles of a line |
| `P, DL 2` | P | read a card into the first 12 minor cycles of a line |
| `Q, 0000,0100,0000,0000` or `Q, X 7` | Q | digits 17-32 into TS 6 |
| `AND` `OR` `NOT` `IFF` `NEVER` or `R, 001` | R | TS 9 with TS 10 into TS 8 |
| `ADD` `ADD 'A'` `SUBTRACT` `SUBTRACT 'A'` | S | add or subtract TS 2 |
| `MULTIPLY` `MULTIPLY 'A'` | S | add TS 2 times TS 3 |
| `ROTATE 16` | S | rotate the partial sum by the places in TS 7 |
| `S, POS, CANCEL, GIV, 6` | S | the arithmetic by its signals |
| `T, 5` | T | stimulate the valve element FLEX 5 |
| `CLEAR TS 22, 23` | | NEVER, TS 8 - TS 6, and TS 6 into each |
| `0000,0101,...` (32 digits), `+7`, `-2` | | a word of the table |

`'A'` may be written with the report's curly quotes. A dash may be `-`, `–`
or `→`.

## The library

```console
$ ace library
INDEXIN   13 minor cycles   printed in the report
BURY      36 minor cycles   described in the report; the table is ours
UNBURY    23 minor cycles   described in the report; the table is ours
DISCRIM   33 minor cycles   described in the report; the table is ours
PLUSIND   10 minor cycles   described in the report; the table is ours
TRANS      5 minor cycles   described in the report; the table is ours
BINDEC   147 minor cycles   described in the report; the table is ours
CALPOL    53 minor cycles   printed in the report
MULTIP    11 minor cycles   described in the report; the table is ours
ADD       12 minor cycles   described in the report; the table is ours
```

## Commands

```console
$ ace help
ace: the Automatic Computing Engine as Alan Turing proposed it in 1945

usage:
  ace run FILE [--input N,N,...] [--cards numbers|listing|holes] [--trace]
          [--time] [--cold] [--json] [--errata LIST] [--limit N]
        Run a job: the instruction tables in FILE and those it uses from the
        library. It starts at the first table in the file and ends when an
        instruction passes control to itself.
        --input puts numbers on cards in the reader, twelve to a card.
        Without it they are read from standard input.
        --cards chooses how punched cards are shown: as twelve numbers, as
        the line a tabulator would list, or as rows of holes.
        --trace prints every instruction obeyed, with the minor cycle it
        was found in. --time prints the machine time taken.
        --cold starts with nothing in the machine but the deck in the
        reader: see `ace deck`.
        --json prints the whole run as one JSON object instead: the
        machine before the start, every instruction obeyed with what it
        changed, and the cards. The playground steps through it.
        FILE may be `-`: standard input is then the tables, a line `----`,
        and the numbers.
  ace deck FILE [--holes] [--errata LIST]
        List the deck of cards that loads the job into an empty machine:
        what each card puts where, and with --holes the holes themselves.
        `ace run FILE --cold` starts from that deck. Section 7 of the report:
        the initial order `consists of 0 throughout`, which reads a card into
        DL 0; each card of the deck carries its own orders.
  ace tables FILE [--emit listing|cards|digits] [--errata LIST]
        Show a job in machine form: every instruction with its position,
        its 32 digits and its popular form; or as the 80-column instruction
        cards of section 7; or as the digits alone.
  ace library [NAME]
        List the tables in the library, or print one.
  ace audit
        Compare what the report prints with itself: the cards of Fig. 38
        with the table they are cards for, and INDEXIN as printed with
        INDEXIN corrected, on the machine.
  ace errata
        List the corrections to the printed tables.
  ace decisions
        List the places where the report says two things, and which was taken.
  ace check FILE
        List what is wrong with a job, and what will run but is probably not
        what was meant, one per line as FILE:LINE:COLUMN: KIND: MESSAGE.
  ace format FILE
        Print the tables set out in columns.
  ace lsp
        Serve editors, speaking the Language Server Protocol on standard
        input and output.

  --errata takes `all` (the default), `none`, or names from `ace errata`.
```

## Sources

- A. M. Turing, *Proposed Electronic Calculator*, National Physical
  Laboratory, 1945. Transcription on Wikisource; facsimile and figures at
  AlanTuring.net (the Turing Archive for the History of Computing).
- B. E. Carpenter and R. W. Doran (eds.), *A. M. Turing's ACE Report of
  1946 and Other Papers*, MIT Press, 1986.
- B. E. Carpenter and R. W. Doran, "The other Turing machine", *The Computer
  Journal* 20(3), 1977, pp. 269-279.
