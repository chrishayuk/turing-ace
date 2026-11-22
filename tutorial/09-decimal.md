# Decimal output

The machine works in binary; people read decimal. Section 7 proposes doing
the conversion "as ITO", an instruction table operation, helped by the fact
that short multiplication is a single instruction. BINDEC is the table.

The report's BINDEC translates a number in its full arithmetical code, with
a decimal point and limits of error, into DL 10, "such that the content of
DL 10 can be passed out onto a card in the usual way and if the card is then
listed the digits of the numbers will then appear on the listing paper in
the usual way". A decimal digit is shown "by the minor cycle in which a
pulse occurs, and its significance by the position of it within the minor
cycle".

The library's BINDEC does whole numbers of up to nine digits, the number in
TS 13. Column 1 is the sign, a hole in row 11 for minus, and columns 2 to 10
are the digits.

```ace
SHOW
  1   B, BURY
  2   B, BINDEC
  3   O, DL 10
  4   B, SHOW 4

TS 13   -1945
```

```output
0 1 62 64 0 0 256 512 0 0 0 128
```

Shown as numbers, the card is twelve rows of binary. Listed, it is what a
tabulator would have printed. A comment `# cards: listing` at the top of a
job chooses the listing, as `--cards listing` does:

```console
$ ace run tutorial/jobs/show.ace --cards listing
-000001945
```

```console
$ ace run tutorial/jobs/show.ace --cards holes
12  ................................
11  o...............................
 0  .ooooo..........................
 1  ......o.........................
 2  ................................
 3  ................................
 4  ........o.......................
 5  .........o......................
 6  ................................
 7  ................................
 8  ................................
 9  .......o........................
```

## How it works

The number is multiplied once by 2⁶⁰ / 10⁹, which makes it a fraction of a
unit placed at digit 60 of the partial sum. Then it is multiplied by ten,
nine times, with `S, POS, CANCEL, SELF, GIV, 10`; each time the next decimal
digit comes out at the top, in digits 61 to 64. The digit says which row of
the card to punch, so BINDEC turns it into a position, builds the instruction
that fetches that row, and obeys it from TS 6.

```console
$ ace library BINDEC
# BINDEC (General Description). The number in TS 13 is translated into
# decimal form and transferred into DL 10. The notation of the decimal
# form is such that the content of DL 10 can be passed out onto a card in
# the usual way and if the card is then listed the digits of the number
# will then appear on the listing paper in the usual way. Only the first
# 12 minor cycles of DL 10 are used, and a decimal digit is represented by
# the minor cycle in which a pulse occurs, and its significance by the
# position of it within the minor cycle.
#
# The report describes BINDEC for the two minor cycles of its arithmetical
# code and prints no table. This table is ours and does whole numbers
# only: TS 13 holds an integer of at most nine decimal digits. Column 1 of
# the card is the sign and columns 2 to 10 the digits.
#
# The method is the one section 7 points to: "short multiplication is a
# CAO". The number is multiplied once by 2^60 / 10^9, which makes it a
# fraction, and then nine times by ten, each time pushing one decimal digit
# out of the top. TS 2-6, 8-10, 15-17, 24, 30 and 32 are altered. TS 7 is
# left at 16.
BINDEC
  1   NEVER
  2   TS 8 - TS 6
  3   TS 6 - DL 10,1;  TS 6 - DL 10,2;  TS 6 - DL 10,3;  TS 6 - DL 10,4
  7   TS 6 - DL 10,5;  TS 6 - DL 10,6;  TS 6 - DL 10,7;  TS 6 - DL 10,8
 11   TS 6 - DL 10,9;  TS 6 - DL 10,10; TS 6 - DL 10,11; TS 6 - DL 10,12
# The sign: TS 5 is all ones for a negative number. A hole in row 11.
 15   TS 13 - TS 2;  ADD 'A';  TS 5 - TS 6;  TS 6 - TS 10;  TS 6 - TS 3
      BINDEC 201 - TS 9;  AND;  TS 8 - DL 10,2
# The size: the number with every digit changed if it is negative, less
# the sign.
      TS 13 - TS 9;  IFF;  TS 8 - TS 10;  NOT;  TS 8 - TS 2;  ADD 'A'
      TS 3 - TS 2;  SUBTRACT;  TS 4 - TS 2
      BINDEC 202 - TS 3;  MULTIPLY 'A'
      TS 4 - TS 32;  TS 5 - TS 30
      BINDEC 203 - TS 17
# Each decimal digit. First the fraction is put back into TS 4 and TS 5
# from TS 32 and TS 30, the top half by two rotations of 16.
 60   TS 30 - TS 2;  ADD 'A';  ROTATE 16;  ROTATE 16
      TS 32 - TS 9;  BINDEC 204 - TS 10;  AND;  TS 8 - TS 2;  ADD
      BINDEC 205 - TS 10;  AND;  TS 8 - TS 2;  SUBTRACT
      S, POS, CANCEL, SELF, GIV, 10
      TS 4 - TS 32
      TS 5 - TS 9;  BINDEC 206 - TS 10;  AND;  TS 8 - TS 30
# The digit is in digits 29-32 of TS 5. Rotating 21 places brings it to
# digits 18-21, where it counts minor cycles.
      BINDEC 207 - TS 10;  AND;  TS 8 - TS 2;  ADD 'A'
      BINDEC 208 - TS 7;  ROTATE 21;  BINDEC 209 - TS 7
      TS 5 - TS 9;  BINDEC 210 - TS 10;  AND;  TS 8 - TS 2;  ADD 'A'
# Fetch the row for that digit, put the column's pulse in it, and make the
# instruction that stores it again: the same with digit 8 taken off.
      BINDEC 211 - TS 2;  ADD;  TS 4 - TS 6;  B, 1
      TS 6 - TS 9;  TS 17 - TS 10;  OR
      BINDEC 212 - TS 2;  SUBTRACT;  TS 4 - BINDEC 150
      TS 8 - TS 6
150   0000,0000,0000,0000,0000,0000,0000,0000
# The next column, and whether that was the last.
151   TS 17 - TS 2;  ADD 'A';  ADD;  TS 4 - TS 17
      TS 17 - TS 9;  BINDEC 213 - TS 10;  AND
      BINDEC 214 - TS 15;  BINDEC 215 - TS 16
      B, BURY;  B, DISCRIM;  TS 24 - TS 6;  B, 1
190   B, UNBURY
# The numbers: 1; 2^60 / 10^9; the pulse of column 2; digits 1-31;
# digit 32; digits 1-28; digits 29-32; 21; 16; digits 18-21; the
# instruction that fetches the row of the digit 0; digit 8; the pulse of
# column 11; and the two ways on.
201   +1
202   +1152921505
203   +2
204   1111,1111,1111,1111,1111,1111,1111,1110
205   0000,0000,0000,0000,0000,0000,0000,0001
206   1111,1111,1111,1111,1111,1111,1111,0000
207   0000,0000,0000,0000,0000,0000,0000,1111
208   +21
209   +16
210   0000,0000,0000,0000,0111,1000,0000,0000
211   L, DL 10,3
212   0000,0001,0000,0000,0000,0000,0000,0000
213   0000,0000,0010,0000,0000,0000,0000,0000
214   B, BINDEC 190
215   B, BINDEC 60
```
