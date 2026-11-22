# Cards

The machine talks to the world through Hollerith cards. Section 7 gives it a
card reader and a card punch, each using 32 columns and all 12 rows of a
card. A row of 32 holes is exactly one minor cycle, the machine's word, so a
card is twelve words.

## Reading and punching

Type P passes the card in the reader on to the first twelve minor cycles of a
delay line. Type O passes the first twelve minor cycles of a delay line out
onto a card.

```ace
ECHO
  1   P, DL 2
  2   O, DL 2
  3   B, ECHO 3
```

```input
1 2 3 4 5 6 7 8 9 10 11 12
```

```output
1 2 3 4 5 6 7 8 9 10 11 12
```

The numbers in the input are the twelve rows of a card, as binary numbers.
`ace run` prints a punched card the same way unless it is told otherwise.

## How a job ends

The report gives the machine no instruction to stop. An instruction that
passes control to itself, `B, ECHO 3` above, is a machine with nothing
more to do, and `ace run` ends there. If the job asks for a card when the
reader is empty, it ends with a message instead:

```ace
ECHO
  1   P, DL 2
  2   P, DL 2
  3   B, ECHO 3
```

```input
1
```

```output
ace: stopped at ECHO 2: waiting for a card that is not in the reader
```

## Seeing the holes

A tabulator lists a card by printing, for each column, the row it is
punched in. BINDEC (chapter 9) writes numbers so that the listing shows
their decimal digits. `--cards holes` shows the card itself:

```console
$ ace run examples/echo.ace --input 1,2,3 --cards holes
12  o...............................
11  .o..............................
 0  oo..............................
 1  ................................
 2  ................................
 3  ................................
 4  ................................
 5  ................................
 6  ................................
 7  ................................
 8  ................................
 9  ................................
```

## Starting from cold

So far every job has been put straight into the delay lines. The machine of
the report starts with nothing in it, and reads its first orders from a
card. Section 7:

> Now let us consider what is done right at the beginning. Arrangements are
> made for setting into CI and CD a certain invariable initial order and IN.
> These state that the card is to be transferred into a particular delay
> line, and that the next order is to be taken from a particular spot, which
> will actually be in this same delay line. [...] The first few orders
> obeyed will probably be to take in a few more cards. [...] Actually it has
> been arranged that the special initial order consists of 0 throughout so
> that there is no need to set it up.

An instruction of 32 zeros is `P, DL 0`: read a card into DL 0. An empty
machine holds nothing else, so it starts by reading a card, and its next
instruction is the third row of that card. `ace deck` turns a job into cards
that load it this way. The format of the cards is ours, not the report's:
each card carries two words and the orders that put them in place, and ends
by sending the machine back to the instruction of zeros, which reads the
next card.

```console
$ ace deck examples/echo.ace
card   1  ECHO 1 into DL 32,1, ECHO 2 into DL 32,3
card   2  ECHO 3 into DL 32,5, TS 31 into TS 31
card   3  TS 7 into TS 7
card   4  start: go to ECHO 1
4 cards
```

`--cold` starts the machine empty, with the deck in the reader in front of
the job's own cards:

```console
$ ace run examples/echo.ace --input 1,2,3 --cold --time
1 2 3
4 cards of the deck read from cold
21 instructions in 112929 minor cycles: 3613.7 milliseconds of machine time
```

Nearly all of that time is the reader. The report does not say how fast its
card reader would be; the simulation takes a hundred cards a minute.
