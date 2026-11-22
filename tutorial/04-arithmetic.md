# Arithmetic

The arithmetic part is a single process with switches. The *partial sum* is
TS 4 and TS 5 in series, 64 digits. A multiple of TS 2 is added to it or
taken from it. Which multiple, and whether the partial sum is cleared first,
are signals set by digits of the instruction (Fig. 19 and Fig. 37).

## Adding

`ADD 'A'` is the report's name for "add TS 2 into TS 4 cancelling the partial
sums": the partial sum becomes TS 2. `ADD` adds TS 2 to what is there.

```ace
# cards: listing
SUM
  1   P, DL 2
  2   DL 2,1 - TS 2
  4   ADD 'A'
  5   DL 2,2 - TS 2
  7   ADD
  8   TS 4 - TS 13
 10   B, BURY
 11   B, BINDEC
 12   O, DL 10
 13   B, SUM 13
```

```input
1071 462
```

```output
 000001533
```

`SUBTRACT 'A'` and `SUBTRACT` are the same without the signal POS: TS 2 is
taken away. Subtracting from a cancelled sum makes a number negative:

```ace
# cards: listing
NEGATE
  1   P, DL 2
  2   DL 2,1 - TS 2
  4   SUBTRACT 'A'
  5   TS 4 - TS 13
  7   B, BURY
  8   B, BINDEC
  9   O, DL 10
 10   B, NEGATE 10
```

```input
42
```

```output
-000000042
```

## Multiplying

`MULTIPLY 'A'` is long multiplication: TS 2 times TS 3, the whole 32 digits
of each, into the cancelled partial sum. It repeats the fundamental process
eight times, four digits of the multiplier at a time (AUTO), and takes 16
minor cycles where an addition takes 2.

```ace
# cards: listing
PRODUCT
  1   P, DL 2
  2   DL 2,1 - TS 2
  4   DL 2,2 - TS 3
  6   MULTIPLY 'A'
  7   TS 4 - TS 13
  9   B, BURY
 10   B, BINDEC
 11   O, DL 10
 12   B, PRODUCT 12
```

```input
1945 1946
```

```output
 003784970
```

## The signals

Every arithmetical instruction can be written as type S with the signals it
sets. These are the names Fig. 19 gives digits 10 to 20:

| signal | digit | means |
|--------|-------|-------|
| POS | 10 | add (without it, subtract) |
| CANCEL | 11 | clear the partial sum first |
| SELF | 12 | the multiplicand is the partial sum, not TS 2 |
| NOR | 13 | four digits of TS 3 are the multiplier |
| GIV | 14 | the multiplier is given in digits 24-27 of the instruction |
| ERR | 15 | four digits of TS 11 are the multiplier |
| AUTO | 18 | repeat eight times: long multiplication |
| ROTATE | 19 | rotate the partial sum |

Short multiplication by a number less than 16 is GIV with the number last.
Fig. 37 works "multiply TS 3 by 6" and "multiply the partial sum by 6":

```ace
# cards: listing
SHORT
  1   P, DL 2
  2   DL 2,1 - TS 2
  4   S, POS, CANCEL, GIV, 6
  5   S, POS, CANCEL, SELF, GIV, 6
  6   TS 4 - TS 13
  8   B, BURY
  9   B, BINDEC
 10   O, DL 10
 11   B, SHORT 11
```

```input
7
```

```output
 000000252
```

## Rotating

`ROTATE` turns the 64 digits of the partial sum towards the more significant
end by the number of places in TS 7. The digits that leave the top come back
at the bottom. TS 7 holds 16 when a job starts, because INDEXIN needs it and
nothing in the report sets it (chapter 11). Rotating by 16 twice moves the
lower half of the partial sum into the upper:

```ace
HALVES
  1   DL 1,1 - TS 2
  3   ADD 'A'
  4   ROTATE 16
  5   ROTATE 16
  6   TS 5 - DL 2,1
  8   TS 4 - DL 2,2
 10   O, DL 2
 11   B, HALVES 11

DL 1,1   1945
```

```output
1945
```

There is no division. Section 12 leaves it to an instruction table, by the
recurrence u(n+1) = u(n)(2 - a·u(n)).
