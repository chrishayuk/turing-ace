# Choices

Section 6 gives the idea: to go one of two ways, calculate the instruction.
"Pretend that the instructions were really numbers and calculate D x
Instruction 50 + (1-D) x Instruction 33." Then obey it.

## DISCRIM

The report's table for this is DISCRIM: "If TS 8 contains any digit 1 then
TS 15 is passed into TS 24, otherwise TS 16 is passed into TS 24." TS 15 and
TS 16 hold two instructions of type B. After DISCRIM, TS 24 holds the chosen
one; move it into TS 6 and obey it with `B, 1`.

```
COUNT 61 - TS 15;  COUNT 62 - TS 16
B, BURY;  B, DISCRIM;  TS 24 - TS 6;  B, 1
...
 61   B, COUNT 10
 62   B, COUNT 50
```

The instructions are kept as words of the table, after its last
instruction, and fetched like numbers.

## A loop

Count down from the number on the card. TS 20 is the counter; AND of TS 20
with itself puts it into TS 8 for DISCRIM.

```ace
# cards: listing
COUNT
  1   P, DL 2
  2   DL 2,1 - TS 20
 10   TS 20 - TS 13;  B, BURY;  B, BINDEC;  O, DL 10
      TS 20 - TS 2;  ADD 'A';  COUNT 60 - TS 2;  SUBTRACT;  TS 4 - TS 20
      TS 20 - TS 9;  TS 20 - TS 10;  AND
      COUNT 61 - TS 15;  COUNT 62 - TS 16
      B, BURY;  B, DISCRIM;  TS 24 - TS 6;  B, 1
 50   B, COUNT 50
 60   +1
 61   B, COUNT 10
 62   B, COUNT 50
```

```input
3
```

```output
 000000003
 000000002
 000000001
```

## Is it negative?

After an addition TS 5 is the upper half of a 64-digit partial sum: all
zeros if it is positive or zero, all ones if it is negative. AND of TS 5 with
itself into TS 8 asks DISCRIM whether a - b < 0. That is how Euclid's
algorithm by subtraction decides which number to reduce:

```console
$ ace run examples/gcd.ace --input 1071,462
 000000021
```

## Factorials

```ace
# cards: listing
FACT
  1   P, DL 2
  2   DL 2,1 - TS 20
  4   FACT 60 - TS 21
 10   TS 21 - TS 2;  TS 20 - TS 3;  MULTIPLY 'A';  TS 4 - TS 21
      TS 20 - TS 2;  ADD 'A';  FACT 60 - TS 2;  SUBTRACT;  TS 4 - TS 20
      TS 20 - TS 9;  TS 20 - TS 10;  AND
      FACT 61 - TS 15;  FACT 62 - TS 16
      B, BURY;  B, DISCRIM;  TS 24 - TS 6;  B, 1
 50   TS 21 - TS 13;  B, BURY;  B, BINDEC;  O, DL 10
 56   B, FACT 56
 60   +1
 61   B, FACT 10
 62   B, FACT 50
```

```input
10
```

```output
 003628800
```
