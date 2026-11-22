# Time

## Waiting for a word

A minor cycle in a delay line can be read only as it comes out of the end,
once every 32 minor cycles. Reading a word costs from one minor cycle to
33, a millisecond, depending on where it is when it is asked for. Turing:
"The three most time consuming operations are multiplication, waiting for
material in long delay lines, and transfers of material."

The simulation keeps time in minor cycles. `--time` reports it. These two
jobs each fetch four numbers. In the first, every number is in the minor
cycle that comes out of its line just as the instruction asking for it
begins. In the second, every number has just gone by:

```console
$ ace run tutorial/jobs/placed.ace --time
9 instructions in 17 minor cycles: 544 microseconds of machine time
```

```console
$ ace run tutorial/jobs/unplaced.ace --time
9 instructions in 145 minor cycles: 4640 microseconds of machine time
```

The whole difference is the wait for the numbers: each one in the second job
has just gone by, so the machine waits nearly a whole turn of the line for
it. Where a word is matters as much as what it is.

## The next instruction

Instructions are in the delay lines too, and the machine has to wait for
each. Section 11 explains why the instruction after one of type A is two
minor cycles on, not one: "If the equation were the one with P18 it would
mean that the next instruction to be obeyed, after one of type A, is always
adjacent to it in time. This would mean that even with the shortest CA
operations the next instruction would have gone by before we were ready to
apply it; we should always just miss the boat."

So CD, the position of the next instruction, gets a pulse added in digit 19,
not digit 18, and the digits of a position are weighted in an unusual order
(Fig. 19): digit 19 is 1, 20 is 2, 21 is 4, 22 is 8, 18 is 16, then digit 23
upwards is the delay line. Consecutive instruction numbers are minor cycles
0, 2, 4, ... 30 of a line, sixteen of them; the odd minor cycles are left for
numbers. `ace tables` shows it: the tables start at DL 32,1, DL 32,3, ...

With transfers and logical operations taking one minor cycle, the next
instruction arrives as the last ends. Longer operations do not lose it.
Section 11 describes the control looking for the next instruction while the
arithmetic part is still busy with this one: the instruction found waits in
CI until the operation before it is over (OKCA), and then begins. An
addition takes two minor cycles, the length of the partial sum, and the next
instruction is already in CI when it ends. What costs time is a word that
has to come round. Placing words so that each arrives as it is wanted became
the art of "optimum coding" on the Pilot ACE and DEUCE.

## A trace

`--trace` prints each instruction obeyed: the minor cycle it reached CI in,
its popular name, its 32 digits, its popular form and what it changed.

```console
$ ace run tutorial/jobs/unplaced.ace --trace
       0  UNPLACED 1   0000,0101,0000,0000,0000,0010,1000,0000  DL 5,1 - TS 6              TS 6 = 0000,0000,0000,0000,0000,0000,0000,0000
       2  UNPLACED 2   0000,0011,0001,0100,0000,0000,0000,0000  TS 6 - TS 20               TS 20 = 0000,0000,0000,0000,0000,0000,0000,0000
      36  UNPLACED 3   0000,0101,0000,0000,0001,0010,1000,0000  DL 5,5 - TS 6              TS 6 = 0000,0000,0000,0000,0000,0000,0000,0000
      38  UNPLACED 4   0000,0011,0101,0100,0000,0000,0000,0000  TS 6 - TS 21               TS 21 = 0000,0000,0000,0000,0000,0000,0000,0000
      72  UNPLACED 5   0000,0101,0000,0000,0000,1010,1000,0000  DL 5,9 - TS 6              TS 6 = 0000,0000,0000,0000,0000,0000,0000,0000
      74  UNPLACED 6   0000,0011,0011,0100,0000,0000,0000,0000  TS 6 - TS 22               TS 22 = 0000,0000,0000,0000,0000,0000,0000,0000
     108  UNPLACED 7   0000,0101,0000,0000,0001,1010,1000,0000  DL 5,13 - TS 6             TS 6 = 0000,0000,0000,0000,0000,0000,0000,0000
     110  UNPLACED 8   0000,0011,0111,0100,0000,0000,0000,0000  TS 6 - TS 23               TS 23 = 0000,0000,0000,0000,0000,0000,0000,0000
```

The machine is not simulated pulse by pulse, and the circuits of Figs. 12 to
36 are not simulated at all. Instructions are, with the time each takes as
the report's circuits would take it.
