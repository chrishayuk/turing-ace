# Storage

## Minor cycles and delay lines

A delay line is a tube of mercury about five feet long. A pulse put in at one
end comes out at the other 1.024 milliseconds later, and is put back in.
1024 pulses, a microsecond apart, circulate in each line. The report divides
them into 32 *minor cycles* of 32 digits, and a minor cycle is the unit the
machine works in: a number, or an instruction.

`DL 3,5` is the fifth minor cycle of delay line 3. The report allows for
several hundred lines; the simulation has 200.

A line in a job that names a storage and gives it a content puts it there
before the job starts. The content can be a number with or without a sign,
32 binary digits in groups of four, an instruction, or a position:

```
DL 3,5   1945
DL 3,6   -1
DL 3,7   1010,0000,0000,0000,0000,0000,0000,0000
DL 3,8   L, DL 3,5
TS 27    DL 3,5
```

## Digits

The report numbers the digits of a minor cycle 1 to 32, in the order they
come out of the line. Digit 1 is the least significant: the 32 digits above
are 5. Digit 32 counts as minus 2³¹, so a word of 32 ones is -1.

## Temporary storages

A word in a delay line can only be reached when it comes round. The 32
*temporary storages*, TS 1 to TS 32, are delay lines one minor cycle long,
always available. Some have duties (section 12):

| TS | duty |
|----|------|
| 1 | where type B leaves the position it came from |
| 2, 3 | what the arithmetic adds, and the multiplier |
| 4, 5 | the result of the arithmetic, 64 digits, TS 4 the lower half |
| 6 | the shunting station: every transfer goes through it |
| 7 | how far ROTATE turns the partial sum |
| 8 | the result of the logical operations |
| 9, 10 | the inputs of the logical operations |

## Transfers

Only four types of instruction move words, and all of them use TS 6:

| type | does |
|------|------|
| K | TS 6 into a minor cycle of a delay line |
| L | a minor cycle into TS 6 |
| M | a TS into TS 6 |
| N | TS 6 into a TS |

`TS 27 - TS 6` is written for M, and the popular form writes any transfer
`from - to`. When neither end is TS 6 the transfer is two instructions, which
is why the instruction after `DL 1,1 - TS 2` in chapter 1 is numbered 3:

```console
$ ace tables examples/fetch.ace --emit listing
BURY  (library: ours, to the report's description)
    1   DL 32,1    0000,0010,0111,1100,0000,0000,0000,0000   TS 31 - TS 6
    2   DL 32,3    0000,0011,0010,0000,0000,0000,0000,0000   TS 6 - TS 2
    3   DL 32,5    0000,1000,0110,0100,0000,0001,0000,0000   ADD 'A'
    4   DL 32,7    0000,0101,0000,0000,0000,0001,0001,0000   BURY 40 - TS 6
    5   DL 32,9    0000,0011,0010,0000,0000,0000,0000,0000   TS 6 - TS 2
    6   DL 32,11   0000,1000,0100,0100,0000,0001,0000,0000   ADD
    7   DL 32,13   0000,0010,0001,0000,0000,0000,0000,0000   TS 4 - TS 6
    8   DL 32,15   0000,0100,0000,0000,0011,1100,0001,0000   TS 6 - BURY 16
    9   DL 32,17   0000,0010,0100,0000,0000,0000,0000,0000   TS 1 - TS 6
   10   DL 32,19   0000,0011,0010,0000,0000,0000,0000,0000   TS 6 - TS 2
   11   DL 32,21   0000,1000,0110,0100,0000,0001,0000,0000   ADD 'A'
   12   DL 32,23   0000,0101,0000,0000,0010,0001,0001,0000   BURY 41 - TS 6
   13   DL 32,25   0000,0011,0010,0000,0000,0000,0000,0000   TS 6 - TS 2
   14   DL 32,27   0000,1000,0100,0100,0000,0001,0000,0000   ADD
   15   DL 32,29   0000,0010,0001,0000,0000,0000,0000,0000   TS 4 - TS 6
   16   DL 32,31   0000,0000,0000,0000,0000,0000,0000,0000   P, DL 0
   17   DL 33,1    0000,0010,0111,1100,0000,0000,0000,0000   TS 31 - TS 6
   18   DL 33,3    0000,0011,0010,0000,0000,0000,0000,0000   TS 6 - TS 2
   19   DL 33,5    0000,1000,0110,0100,0000,0001,0000,0000   ADD 'A'
   20   DL 33,7    0000,0101,0000,0000,0001,0001,0001,0000   BURY 42 - TS 6
   21   DL 33,9    0000,0011,0010,0000,0000,0000,0000,0000   TS 6 - TS 2
   22   DL 33,11   0000,1000,0100,0100,0000,0001,0000,0000   ADD
   23   DL 33,13   0000,0010,0001,0000,0000,0000,0000,0000   TS 4 - TS 6
   24   DL 33,15   0000,0011,0111,1100,0000,0000,0000,0000   TS 6 - TS 31
   25   DL 33,17   0000,0010,0100,0000,0000,0000,0000,0000   TS 1 - TS 6
   26   DL 33,19   0000,0011,0010,0000,0000,0000,0000,0000   TS 6 - TS 2
   27   DL 33,21   0000,1000,0110,0100,0000,0001,0000,0000   ADD 'A'
   28   DL 33,23   0000,0101,0000,0000,0011,0001,0001,0000   BURY 43 - TS 6
   29   DL 33,25   0000,0011,0010,0000,0000,0000,0000,0000   TS 6 - TS 2
   30   DL 33,27   0000,1000,0100,0100,0000,0001,0000,0000   ADD
   31   DL 33,29   0000,0010,0001,0000,0000,0000,0000,0000   TS 4 - TS 6
   32   DL 33,31   0110,0000,0000,0000,1011,1110,0001,0000   B, 1, BURY 32
   40   DL 34,1    0000,0100,0000,0000,0000,0000,0000,0000   TS 6 - DL 0,1
   41   DL 34,3    0000,0000,0000,0000,0010,0000,0000,0000   P, DL 0
   42   DL 34,5    0000,0000,0000,0000,0100,0000,0000,0000   P, DL 0
   43   DL 34,7    0110,0000,0000,0000,0010,0000,0000,0000   B, DL 0,3

FETCH
    1   DL 34,9    0110,0000,0000,0000,0000,0000,0001,0000   B, BURY 1
    2   DL 34,11   0110,0000,0000,0000,0001,0101,0001,0000   B, INDEXIN 1
    3   DL 34,13   0000,0010,0001,1100,0000,0000,0000,0000   TS 28 - TS 6
    4   DL 34,15   0000,0100,0000,0000,0000,0001,0000,0000   TS 6 - DL 2,1
    5   DL 34,17   0000,0001,0000,0000,0000,0001,0000,0000   O, DL 2
    6   DL 34,19   0110,0000,0000,0000,0010,0101,0001,0000   B, FETCH 6

INDEXIN  (library: printed in the report)
    1   DL 34,21   0000,0110,0000,0000,0000,0101,0000,0000   Q, 0000,0101,0000,0000
    2   DL 34,23   0000,0011,0010,0000,0000,0000,0000,0000   TS 6 - TS 2
    3   DL 34,25   0000,1000,0110,0100,0000,0001,0000,0000   ADD 'A'
    4   DL 34,27   0000,1000,0000,0000,0010,0000,0000,0000   ROTATE
    5   DL 34,29   0000,0010,0101,0000,0000,0000,0000,0000   TS 5 - TS 6
    6   DL 34,31   0000,0011,0100,1000,0000,0000,0000,0000   TS 6 - TS 9
    7   DL 35,1    0000,0010,0110,1100,0000,0000,0000,0000   TS 27 - TS 6
    8   DL 35,3    0000,0011,0010,1000,0000,0000,0000,0000   TS 6 - TS 10
    9   DL 35,5    0000,0111,0000,0000,0000,0000,1000,0000   OR
   10   DL 35,7    0000,0010,0000,1000,0000,0000,0000,0000   TS 8 - TS 6
   11   DL 35,9    0110,0000,0000,0000,1000,1011,0001,0000   B, 1, INDEXIN 11
   12   DL 35,11   0000,0011,0001,1100,0000,0000,0000,0000   TS 6 - TS 28
   13   DL 35,13   0110,0000,0000,0000,0011,1011,0001,0000   B, UNBURY 1

UNBURY  (library: ours, to the report's description)
    1   DL 35,15   0000,0010,0111,1100,0000,0000,0000,0000   TS 31 - TS 6
    2   DL 35,17   0000,0011,0010,0000,0000,0000,0000,0000   TS 6 - TS 2
    3   DL 35,19   0000,1000,0110,0100,0000,0001,0000,0000   ADD 'A'
    4   DL 35,21   0000,0101,0000,0000,0011,0100,1001,0000   UNBURY 40 - TS 6
    5   DL 35,23   0000,0011,0010,0000,0000,0000,0000,0000   TS 6 - TS 2
    6   DL 35,25   0000,1000,0000,0100,0000,0001,0000,0000   SUBTRACT
    7   DL 35,27   0000,0010,0001,0000,0000,0000,0000,0000   TS 4 - TS 6
    8   DL 35,29   0000,0011,0111,1100,0000,0000,0000,0000   TS 6 - TS 31
    9   DL 35,31   0000,0101,0000,0000,0000,1100,1001,0000   UNBURY 41 - TS 6
   10   DL 36,1    0000,0011,0010,0000,0000,0000,0000,0000   TS 6 - TS 2
   11   DL 36,3    0000,1000,0100,0100,0000,0001,0000,0000   ADD
   12   DL 36,5    0000,0010,0001,0000,0000,0000,0000,0000   TS 4 - TS 6
   13   DL 36,7    0110,0000,0000,0000,1011,0000,1001,0000   B, 1, UNBURY 13
   14   DL 36,9    0000,0011,0010,0000,0000,0000,0000,0000   TS 6 - TS 2
   15   DL 36,11   0000,1000,0110,0100,0000,0001,0000,0000   ADD 'A'
   16   DL 36,13   0000,0101,0000,0000,0010,1100,1001,0000   UNBURY 42 - TS 6
   17   DL 36,15   0000,0011,0010,0000,0000,0000,0000,0000   TS 6 - TS 2
   18   DL 36,17   0000,1000,0100,0100,0000,0001,0000,0000   ADD
   19   DL 36,19   0000,0010,0001,0000,0000,0000,0000,0000   TS 4 - TS 6
   20   DL 36,21   0110,0000,0000,0000,1001,0100,1001,0000   B, 1, UNBURY 20
   40   DL 36,23   0000,0000,0000,0000,0100,0000,0000,0000   P, DL 0
   41   DL 36,25   0000,0101,0000,0000,0000,0000,0000,0000   DL 0,1 - TS 6
   42   DL 36,27   0110,0000,0000,0000,0010,0000,0000,0000   B, DL 0,3

DL 3,5            1001,1001,1110,0000,0000,0000,0000,0000   1945
TS 27             0000,0101,0000,0000,0001,0011,0000,0000   13107360
```

That listing is the *machine form* of the job: where each instruction is, its
32 digits, and the popular form read back from them.

TS 1, 4, 5 and 8 cannot be filled from TS 6. They have "no input connections
except those shown on other diagrams". `ace check` says so:

```console
$ ace check tutorial/jobs/ts4.ace
tutorial/jobs/ts4.ace:2:7: warning: TS 4 takes nothing from TS 6: TS 1 is filled by type B, TS 4 and TS 5 by the arithmetic, TS 8 by the logical operations
```
