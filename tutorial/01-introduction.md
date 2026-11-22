# Introduction

In 1945 Alan Turing wrote *Proposed Electronic Calculator*, a report for the
National Physical Laboratory. It describes a whole computer: its memory of
mercury delay lines, its 32 temporary storages, the circuits of its control
and its arithmetic, valve by valve, and a cost, £11,200. Section 13 then
shows how it would be programmed, with two complete instruction tables and
the descriptions of nine more.

The machine was never built as described. The Pilot ACE, which ran at the
NPL in 1950, was a different and smaller design. So the tables in section 13
are programs for a computer that never existed.

This tutorial teaches the notation they are written in, on a simulation of
that computer. It starts with a single card and ends with Turing's own
tables, and with what happens when they are run.

## Installing

You need [Rust](https://rustup.rs):

```
cargo install --git https://github.com/chrishayuk/turing-ace
```

That puts `ace` in `~/.cargo/bin`. In a clone of the repository,
`cargo build --release` puts it in `target/release`.

## A first job

A job is one or more instruction tables. This one has a single table, called
ANSWER, which multiplies six by seven and punches the product on a card:

```ace
# cards: listing
ANSWER
  1   DL 1,1 - TS 2
  3   DL 1,2 - TS 3
  5   MULTIPLY 'A'
  6   TS 4 - TS 13
  8   B, BURY
  9   B, BINDEC
 10   O, DL 10
 11   B, ANSWER 11

DL 1,1   6
DL 1,2   7
```

```output
 000000042
```

Every chapter that follows explains part of it. In short: the two numbers
are kept in the delay lines and are moved into temporary storages, the
arithmetic part multiplies them, a subsidiary table called BINDEC turns the
product into decimal, and the card punch prints it. The last instruction
passes control to itself, which is how a job ends.

## What is Turing's and what is not

The machine, the types of instruction, the way instructions are numbered and
the popular form of a table are all from the report. INDEXIN and CALPOL are
transcribed from it. BURY, UNBURY, DISCRIM, PLUSIND, TRANS 45, MULTIP, ADD and
BINDEC are tables the report describes in a sentence or two and does not
print; the tables here were written to those descriptions. Chapter 11 says
where the report contradicts itself and what was done about it.

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
