# Fig. 38, read twice

The claim that three of the thirteen cards of Fig. 38 disagree with the
INDEXIN table rests on reading a hand-drawn figure digit by digit. The
reading in `src/audit.rs` (`FIG_38`) was made by the agent that wrote this
repository. To check it, the figure was transcribed again, blind, by a
different model in a separate session: OpenAI's Codex CLI (codex-cli
0.166.0, `codex exec`, read-only, no saved session) on 22 November 2026. It
was given the image of the figure from the Turing Archive for the History of
Computing (www.alanturing.net/ace), twice enlarged, and the instructions
below, and nothing else: not the table, not our reading, not what the figure
is for. The image is not reproduced here.

## The instructions

```
You are transcribing a hand-drawn figure from a 1945 typescript report, for a research comparison. The image fig.png (and an enlarged copy fig-2x.png) is a grid: a header row of column numbers 1 to 32, then 13 numbered rows. In each row some columns contain a hand-written digit 0 or 1; other columns are blank. Some rows instead have an arrow spanning a range of columns with words written over it.

Transcribe exactly what is drawn, nothing inferred. For each row 1-13 output one line:
  ROW n: <32 characters, one per column 1..32: '0' or '1' where that digit is written, '.' where the column is blank, '~' where the column is covered by a labelled arrow> | arrow label if any, else -
Then list any digit you are unsure of, as "UNSURE row r column c: why".
Align each digit to its column by its horizontal position under the header numbers; check carefully, column by column. Do not use any outside knowledge of what the figure is meant to contain.
Return only the transcription and the UNSURE list.
```

## Its answer

`.` is a blank column, `~` a column under a labelled arrow.

```
ROW 1: ..0.0110........0000010000000000 | -
ROW 2: ..0.0011.01000.................. | -
ROW 3: ..0.1000.110100000000........... | -
ROW 4: ..0.1000.0000000001............. | -
ROW 5: ..0.0010.00100.................. | -
ROW 6: ..0.0011.10010.................. | -
ROW 7: ..0.0010.11011.................. | -
ROW 8: ..0.0011.01010.................. | -
ROW 9: ..0.0111..............000....... | -
ROW 10: ..0.0010.00100.................. | -
ROW 11: ..1.............1~~~~~~~~~~~~~~~ | INDEXIN II
ROW 12: ..0.0011.00111.................. | -
ROW 13: ..1..............~~~~~~~~~~~~~~~ | BURY

UNSURE: none.
```

## Against our reading

All thirteen rows have the same digits in the same columns, and the same
two labelled arrows. It reported no digit it was unsure of. In particular
it reads card 3 with digit 13 set and digit 14 clear (NOR, not GIV), card 9
with 000 in digits 23-25 (AND, not OR), and card 10 with 00100 in digits
10-14 (TS 4, not TS 8): the three cards `ace audit` reports.

Two readings by two models agree; neither is a person's. A human check
against the figure is still the stronger test.
