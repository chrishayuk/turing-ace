# Turing's tables

Section 13 prints two tables in full. INDEXIN is "elementary and does not
involve subsidiaries"; CALPOL, which evaluates a polynomial, "consists
largely of such orders". INDEXIN is printed twice: in popular form, and in
Fig. 38 as the cards that would be punched for it.

## INDEXIN

"The minor cycle whose position is described in digits 17-32 of TS 27 is
transferred to TS 28." It builds an instruction of type L from the position
and obeys it from TS 6.

```console
$ ace library INDEXIN --errata none
# INDEXIN (General Description). The minor cycle whose position is
# described in digits 17-32 of TS 27 is transferred to TS 28. The
# contents of TS 2,3,4,5,6,8,9,10 get altered in the process.
#
# Transcribed from section 13 of the report, as printed.
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

Fetching the 1945 kept in DL 3,5 with it, on the machine:

```console
$ ace audit
1. The cards of Fig. 38 against the table they are cards for (INDEXIN as printed)

        1   5    10   15   20   25   30
   1      0 0110        0000010000000000  Q, 0000,0100,0000,0000  agrees
   2      0 0011 01000                    TS 6 - TS 2             agrees
   3      0 1000 110100000000             ADD 'A'                 DIFFERS in digits 13, 14: the card is S, POS, CANCEL, NOR
   4      0 1000 0000000001               ROTATE                  agrees
   5      0 0010 00100                    TS 4 - TS 6             agrees
   6      0 0011 10010                    TS 6 - TS 9             agrees
   7      0 0010 11011                    TS 27 - TS 6            agrees
   8      0 0011 01010                    TS 6 - TS 10            agrees
   9      0 0111              000         OR                      DIFFERS in digit 25: the card is AND
  10      0 0010 00100                    TS 8 - TS 6             DIFFERS in digits 12, 13: the card is TS 4 - TS 6
  11      1             1~~~~~~~~~~~~~~~  B, 1, INDEXIN 11        agrees
  12      0 0011 00111                    TS 6 - TS 28            agrees
  13      1              ~~~~~~~~~~~~~~~  B, BURY 1               agrees

  3 of the 13 cards do not say what the table says. (~ marks an instruction number written in words.)

2. INDEXIN on the machine: fetch the 1945 kept in DL 3,5, whose position is in TS 27

  the table as printed        FAILS   TS 28 = 0; stopped at INDEXIN 11 after 46 instructions: waiting for a card that is not in the reader
  the cards of Fig. 38        FAILS   TS 28 = 0; stopped at INDEXIN 11 after 46 instructions: waiting for a card that is not in the reader
  the table, all corrections  works   TS 28 = 1945; came back after 71 instructions

3. Each correction to INDEXIN left out in turn

  without `literal`        FAILS   TS 28 = 13107232; DL 3,5 now holds 13107232; came back after 71 instructions
  without `half`           FAILS   TS 28 = 0; stopped at INDEXIN 11 after 46 instructions: waiting for a card that is not in the reader
  without `return`         FAILS   TS 28 = 1945; stopped at BURY 1 after 1361 instructions: waiting for a card that is not in the reader

4. CALPOL on the machine: x^3 - 2x + 5 at x = 7, which is 334

                              once                  twice in a row
  as printed                  never stops           never stops
  all corrections             334                   334
  without `clear`             334                   1275202557  WRONG
  without `inputs`            0  WRONG              0  WRONG
  without `branch`            waits for a card      waits for a card
  without `carry`             never stops           never stops
  without `contents`          0  WRONG              0  WRONG

  INDEXIN, BURY, UNBURY and DISCRIM are corrected in every row: CALPOL calls them.

5. DISCRIM: does TS 8 contain any digit 1?

  no digits       as outlined: no            corrected: no 
  digit 1 alone   as outlined: yes           corrected: yes
  digit 32 alone  as outlined: no   WRONG    corrected: yes
  all 32 digits   as outlined: no   WRONG    corrected: yes
```

As printed it does not work, and the cards of Fig. 38 do not agree with the
table in three places. It needs three corrections, and the audit leaves each
out in turn to show that each is needed:

- `literal`: line 1 makes the code of type K, which stores, where type L,
  which fetches, is meant. Carpenter and Doran noted this in 1986.
- `half`: the code is made in the upper half of a word and turned down by
  ROTATE 16, which by section 12 moves it into TS 5. Line 5 takes TS 4.
- `return`: the table ends `B, BURY` where it should end `B, UNBURY`.
  Carpenter and Doran noted this too.

```console
$ ace errata
literal  (INDEXIN)
    INDEXIN 1 builds the code of type K, not of type L
    INDEXIN makes an instruction to fetch a minor cycle and obeys it from TS 6. Line 1 supplies the digits of its type: 0000,0100, which after the rotation are digits 5-8 = 0100. Fig. 19 gives 0100 to type K, which stores TS 6, and 0101 to type L, which fetches. As printed the table writes the instruction it has just built over the minor cycle it was sent to read. The literal becomes 0000,0101,0000,0000.
    Known before: Carpenter and Doran, 1986, note 17.

half  (INDEXIN)
    INDEXIN 5 takes the half of the partial sum the digits have left
    Type Q can only fill digits 17-32, so the type digits are made there and moved down by ROTATE 16. Section 12 describes rotation as multiplication by a power of 2: the digits go 16 places up, out of TS 4 and into digits 1-16 of TS 5. Line 5 takes TS 4. It becomes TS 5 - TS 6. (If `forwards` in the report meant towards digit 1, the line stands as printed and section 12 is what is wrong.)
    Known before: not that we found.

return  (INDEXIN)
    INDEXIN 13 buries where it should unbury
    The table ends B, BURY. BURY adds a note to the list and comes back to the instruction after the one that called it, which is past the end of the table. A subsidiary ends by taking the note off: B, UNBURY, as CALPOL does.
    Known before: Carpenter and Doran, 1986, note 18.

clear  (CALPOL)
    CALPOL 1 clears the result, not the running product
    Each turn adds the next coefficient, in TS 18, 19, to the product so far, in TS 20, 21. Before the first turn that product must be zero. The table clears TS 22, 23, which ADD is about to overwrite. On a machine just switched on TS 20, 21 are zero anyway; used a second time, CALPOL starts from the last product of the time before. It becomes CLEAR TS 20, 21.
    Known before: Carpenter and Doran analysed CALPOL in 1977; we have not seen that paper.

inputs  (CALPOL)
    CALPOL loads TS 2 and TS 3 for an AND that reads TS 9 and TS 10
    The test for the last coefficient is TS 27 & TS 29. The table puts them in TS 2 and TS 3; the logical operations of section 12 and Fig. 29 take TS 9 and TS 10, as INDEXIN knows. The two transfers go to TS 9 and TS 10.
    Known before: Carpenter and Doran, 1986, note 12, for the conflict in section 12.

branch  (CALPOL)
    CALPOL builds its two ways on with type Q, which cannot make an instruction
    DISCRIM chooses between two instructions in TS 15 and TS 16, and B, 1 obeys the one chosen. The table makes them with Q, CALPOL 40 and Q, CALPOL 37. Type Q fills digits 17-32 only, so what reaches TS 15 has an instruction number but not the digits of type B: whatever was left in the first half of TS 6 decides what it does. And CALPOL 40 is the middle of the multiplication; the way out is CALPOL 50. The two instructions, B, CALPOL 50 and B, CALPOL 37, are kept as numbers at the end of the table and fetched.
    Known before: Carpenter and Doran analysed CALPOL in 1977; we have not seen that paper.

carry  (CALPOL)
    CALPOL obeys TS 6 after DISCRIM, but the choice is in TS 24
    DISCRIM passes the chosen instruction into TS 24 and comes back through UNBURY, which has to use TS 6 to get back at all. B, 1 then obeys what UNBURY left there: the instruction that returned, which returns to B, 1 again, for ever. TS 24 - TS 6 is put in front of it.
    Known before: not that we found; it depends on how BURY and UNBURY are written.

contents  (CALPOL)
    The contents printed for DL 1,14 and DL 1,15 do not name DL 3,1
    DL 1,14 is to be the order to transfer DL 3,1 to TS 6, and DL 1,15 the digit that shows when the last coefficient has been taken. By Fig. 19 the order printed has digits 18, 22 and 23: minor cycle 18 of DL 1. The contents become the instruction L, DL 3,1 and digit 25, which is 0 in DL 3 and 1 in DL 4, where PLUSIND arrives after 32 minor cycles.
    Known before: Carpenter and Doran analysed CALPOL in 1977; we have not seen that paper.

sign  (DISCRIM)
    DISCRIM, as outlined, calls a word zero when its digit 32 is a 1
    The outline subtracts TS 8 from zero and takes TS 5 to be all ones if TS 8 had any digit 1. But digit 32 has reversed sign. A word with a 1 there is a negative number, zero minus it is positive, and TS 5 is all zeros: the same answer as for a word of 32 zeros. The corrected table adds TS 8 as well as subtracting it, and uses either TS 5.
    Known before: not that we found.
```

## CALPOL

CALPOL calculates a polynomial of degree 15 whose coefficients are in DL 3,
for the argument in TS 13, by Horner's rule. As printed it never stops. With
five corrections it does:

```ace
# cards: listing
POLY
  1   B, BURY
  2   B, CALPOL
  3   TS 25 - TS 13
  5   B, BURY
  6   B, BINDEC
  7   O, DL 10
  8   B, POLY 8

TS 13    7
DL 3,25  +1
DL 3,29  -2
DL 3,31  +5
```

```output
 000000334
```

That is x³ - 2x + 5 at x = 7. The coefficients are in pairs, highest power
first, so the last four pairs begin at DL 3,25. CALPOL works in the
two-word numbers of section 4; the library's MULTIP and ADD do whole numbers
in the first word of each pair.

Carpenter and Doran published an analysis of CALPOL's errors in 1977. It has
not been possible to read that paper for this project, so the corrections
here are not claimed to be new.

## Where the report disagrees with itself

Some of what a simulation needs, the report says twice, differently. Each
place is listed with the reading taken:

```console
$ ace decisions
1. Which digits 5-8 mean which type of operation
    one place:   Fig. 19: K 0100, L 0101, M 0010, N 0011, O 0001, P 0000, Q 0110, R 0111, S 1000, T 1001
    another:     Section 12 and Fig. 30: the tree element TRG 000 is K v L, TRG 001 is M v N
    taken here:  Fig. 19. The cards of Fig. 38 and the constant in CALPOL use its codes.

2. Where type B puts the position it leaves
    one place:   Section 11: `Pass the old CD into TS 13`
    another:     Section 12: TS 1 carries `the CD which applied just before the last instruction of type B`; BURY reads TS 1
    taken here:  TS 1. CALPOL keeps its argument in TS 13.
    noted by:    Carpenter and Doran, 1986, note 11

3. Which storages the logical operations work on
    one place:   Section 12: TS 9 and TS 10 are the inputs, TS 8 the result; so Fig. 29 and INDEXIN
    another:     Section 12, five paragraphs earlier: TS 2 and TS 3 hold the arguments `for the logical operations`; CALPOL loads TS 2 and TS 3 before AND
    taken here:  TS 9 and TS 10.
    noted by:    Carpenter and Doran, 1986, note 12

4. Which digits select a logical operation
    one place:   Section 12: `digits 18 to 23`, values 000000, 001000, 010000, 011000, 100000
    another:     Fig. 19 marks digits 23-28; Fig. 29 selects with the tree elements TRA and TRB 000, which section 11 gives to digits 23-25 and 26-28
    taken here:  Digits 23-28: the first three choose the operation, the last three are 0.

5. Which storage is the multiplicand
    one place:   Section 12: the multiplicand is TS 3, the multiplier TS 2
    another:     Fig. 32 leads TS 2 to the adders and TS 3 to MULIER; INDEXIN and DISCRIM add TS 2
    taken here:  TS 2 is added; TS 3 is the multiplier. The printed tables depend on it.

6. Where a given multiplier sits in the instruction
    one place:   Section 12: P23, `the use of digits 23, 24, 25, 26`; Fig. 19
    another:     Section 12, a page on: `the number 1 in columns 24-27`; Fig. 37
    taken here:  Digits 24-27, as the worked instructions of Fig. 37 have it.
    noted by:    Carpenter and Doran, 1986, note 13

7. Where ROTATE finds how far to go
    one place:   Section 12: digits 1-5 of TS 7, though `any time up to 63` needs six
    another:     Fig. 33 gates TS 7 at P13 to P17
    taken here:  Digits 1-5: TS 7 holds the number of places. The 64 digits of TS 4 and TS 5 turn as a closed cycle towards the more significant end.
    noted by:    Carpenter and Doran, 1986, note 14

8. What type Q does to digits 1-16 of TS 6
    one place:   Section 12: `Pass CI 17-32 into TS 6`
    another:     Section 12 and Fig. 30: in case Q, CATIM lasts from P17 for half a minor cycle; Fig. 27 stops TS 6 circulating only while Q lasts
    taken here:  Digits 1-16 stay as they were. CALPOL's `Clear TS 22, 23` in three instructions would need them cleared.

9. How minor cycles are numbered
    one place:   Fig. 19 weights digits 18-22 as 16, 1, 2, 4, 8, so that adding P19 goes to the next number
    another:     The carry from digit 22 goes into digit 23: after number 15 of a line comes number 0 of the next line, not 16
    taken here:  Positions are kept as delay line and minor cycle. Tables lie in the even minor cycles, 16 instructions to a line.

Before the start the tool sets:
    TS 31  the position of the first note BURY will make, DL 31,1
    TS 7   the 16 places INDEXIN rotates by
```

The errata can be switched off: `--errata none` runs the tables as printed.
