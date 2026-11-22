# Logical operations

Type R works digit by digit on TS 9 and TS 10 and puts the result in TS 8.
Section 4 lists five, in Turing's symbols:

| popular form | result in TS 8 |
|--------------|----------------|
| `AND` | TS 9 & TS 10 |
| `OR` | TS 9 v TS 10 |
| `NOT` | ~TS 10 |
| `IFF` | (TS 9 & TS 10) v (~TS 9 & ~TS 10) |
| `NEVER` | 0 |

Each row of this card is one of them, applied to 1100 and 1010:

```ace
LOGIC
  1   DL 1,1 - TS 9
  3   DL 1,2 - TS 10
  5   AND;   TS 8 - DL 2,1
  8   OR;    TS 8 - DL 2,2
 11   NOT;   TS 8 - DL 2,3
 14   IFF;   TS 8 - DL 2,4
 17   NEVER; TS 8 - DL 2,5
 20   O, DL 2
 21   B, LOGIC 21

DL 1,1  1100,0000,0000,0000,0000,0000,0000,0000
DL 1,2  1010,0000,0000,0000,0000,0000,0000,0000
```

```output
1 7 -6 -7
```

Written first digit first, AND gives 1000, which is 1; OR gives 1110, which
is 7; NOT gives every digit of 1010 changed, which as a number is -6.

Several instructions can go on one line with semicolons between them, as
CALPOL is printed. The detail figure in front is that of the first.

## Clearing

`CLEAR TS 22, 23` is CALPOL's "Clear TS 22, 23": NEVER into TS 8, TS 8 into
TS 6, and TS 6 into each storage named. It is three instructions or more,
and it leaves TS 6 and TS 8 at zero.

## Masks

AND with a word of chosen digits keeps those digits and clears the rest. OR
puts digits in. INDEXIN builds an instruction this way: the code of the type
from one word and the position from another, joined by OR.
