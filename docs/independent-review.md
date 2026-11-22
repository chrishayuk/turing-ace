# Independent review

On 22 November 2026 this repository was reviewed by a second model, OpenAI's
Codex CLI (codex-cli 0.166.0, `codex exec` in its own git worktree), working
independently of the agent that wrote it. It is not a person. It was asked to
build and run everything, to check the simulation against the report's
sections 11, 12 and 13, to try to break each correction, and to check the
README against the tool and the report. It ran every command without a
failure: 22 tests, the tutorial (36 blocks), the audit, the errata, the
decisions and a cold run. Separately, it transcribed Fig. 38 blind
([`fig38-read-twice.md`](fig38-read-twice.md)).

What it found, and what was done (commits `8db268f` and `22bcc46`):

1. **Timing ignored section 11's overlap. Accepted, fixed.** The control
   looks for the next instruction while the arithmetic part finishes the
   current one (OKLK, OKCI, OKCA). The simulation fetched only afterwards,
   so an addition appeared to make the next instruction go by. It now
   overlaps them (`Machine::step`); what instructions do to the storages
   still happens in order. The tutorial, README and hover text that
   explained the old behaviour were rewritten. The placed and unplaced jobs
   now take 17 and 145 minor cycles; the difference is the wait for the
   numbers alone.
2. **Long multiplication and the signals have undocumented rules. Accepted
   in part.** AUTO is simulated as one signed multiplication, which is what
   eight repetitions on four digits amount to; that, DIFF without TS 12, and
   the priority of multiplier sources are now listed among the conventions
   in `ace decisions`. The unsigned top group of digits was a defect: a
   group including digit 32 is now signed.
3. **INDEXIN ORs all 32 digits of TS 27. Accepted, documented.** Its
   description speaks only of digits 17-32. Listed as a convention, and in
   the README.
4. **`carry` is not necessary with another UNBURY. Accepted.** Its kind was
   already "ours"; its wording claimed UNBURY "has to use TS 6", which was
   wrong. Reworded: an UNBURY that returns through a delay line would not
   need it.
5. **The audit shows particular runs, not general necessity; "never
   stops" means only "did not stop within the limit". Accepted.** The
   necessity claim for INDEXIN is now stated for TS 27 holding a position
   only, as its description says. The reviewer's case, INDEXIN without
   `literal` working when TS 27's lower digits supply the code, led to a
   finding of its own: CALPOL calls INDEXIN with a whole order of type L in
   TS 27, so Turing's own use would not have shown that slip (CALPOL gives
   334 with every correction but `literal`). "Never stops" is now proved:
   the audit keeps the machine's whole state every 997 instructions and
   reports a loop when a state comes round again.
6. **`ROTATE n` discards n. Accepted.** The machine rotates by TS 7, as
   decision 7 says; `ace check` now warns when n is not 16.
7. **BURY and UNBURY resolve the report's descriptions in ways not listed.
   Accepted, documented** among the conventions.
8. **Other behaviour beyond the decisions** (self-branch as the end, unused
   codes, unlisted logical selectors, TS field 0, storages starting at zero,
   card timing, FLEX, rotation with arithmetic). **Accepted, documented**:
   `ace decisions` now ends with fifteen conventions where the report says
   nothing. A self-branch now updates TS 1 and the count before the run
   ends.
9. **Verdicts on the nine errata.** It agreed with every kind given, and
   with every diagnosis. On `branch` it noted that the outline also names
   CALPOL 40 and 31, so the exit number is partly a numbering conflict; the
   README says the exit is our reading.
10. **README claims. Accepted.** "Numbered consecutively" now explains that
    Fig. 19's numbers are not consecutive in the delay lines; the card for
    ADD 'A' "sets NOR where the table's instruction sets GIV"; CALPOL's
    success is stated for the polynomial tried, with whole-number MULTIP and
    ADD. It could not check Fig. 38 itself (the figure is not in the
    repository) or the attributions to Carpenter and Doran.
11. **Everything ran.** No failure.

Two readings of Fig. 38 by two models agree; neither is a person's, and the
Carpenter and Doran papers of 1977 and 1986 have still not been read for
this project.
