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
