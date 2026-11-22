#!/usr/bin/env python3
"""Build dataset/ace.jsonl, a chat-format training set about the instruction
tables of Turing's 1945 ACE report and the machine they run on.

Every job in an answer is run, and the cards shown are what it actually
punched, so the dataset cannot drift from the tools. Run from this folder
after building them:

    python3 build_dataset.py
"""
import json
import os
import pathlib
import subprocess
import sys
import tempfile

from check_tutorial import binary

HERE = pathlib.Path(__file__).resolve().parent
EXAMPLES = HERE.parent / "examples"
SYSTEM = ("You are a knowledgeable assistant that provides detailed information "
          "about the Automatic Computing Engine of Alan Turing's 1945 report "
          "'Proposed Electronic Calculator' and the instruction tables written for it.")
ACE = binary()
rows = []


def ace(*args, stdin=""):
    result = subprocess.run([ACE, *args], input=stdin, capture_output=True, text=True,
                            timeout=120, cwd=HERE.parent)
    return result.returncode, (result.stdout + result.stderr).rstrip("\n")


def run(src, stdin=""):
    with tempfile.NamedTemporaryFile("w", suffix=".ace", delete=False) as f:
        f.write(src + "\n")
    try:
        code, out = ace("run", f.name, stdin=stdin)
    finally:
        os.unlink(f.name)
    if code != 0:
        sys.exit(f"job failed:\n{src}\n{out}")
    return out


def job(src, stdin=""):
    """A job, its input cards if any, and what it really punched, as markdown."""
    src = src.strip("\n")
    text = f"```ace\n{src}\n```\n\n"
    if stdin.strip():
        text += f"With a card holding `{' '.join(stdin.split())}` in the reader it punches:\n\n"
    else:
        text += "It punches:\n\n"
    return text + f"```\n{run(src, stdin)}\n```"


def command(*args):
    code, out = ace(*args)
    return f"```\n$ ace {' '.join(args)}\n{out}\n```"


def qa(question, answer):
    rows.append({"messages": [
        {"role": "system", "content": SYSTEM},
        {"role": "user", "content": question},
        {"role": "assistant", "content": answer.strip()},
    ]})


SHOW = "B, BURY;  B, BINDEC;  O, DL 10"

# ---------------------------------------------------------------- the report

qa("What is the ACE?",
   "The Automatic Computing Engine, the computer Alan Turing proposed to the "
   "National Physical Laboratory in his 1945 report 'Proposed Electronic "
   "Calculator'. The report describes the whole machine: a memory of mercury "
   "delay lines, 32 temporary storages, a logical control that interprets "
   "instructions, an arithmetic part, a card reader and a card punch, with circuit "
   "diagrams and an estimated cost of £11,200. Its section 13 shows how it would be "
   "programmed, with instruction tables written in a notation the report calls "
   "popular form.")

qa("Was Turing's ACE ever built?",
   "Not as the 1945 report describes it. The Pilot ACE, which ran at the National "
   "Physical Laboratory in 1950, was a smaller and different design, and so were "
   "DEUCE and the later full-scale ACE. The instruction tables in the report were "
   "written for a machine that never existed, which is why running them on a "
   "simulation of the report's machine is the only way to try them.")

qa("What is a minor cycle?",
   "The word of the ACE: 32 binary digits. A delay line holds 1024 pulses, a "
   "microsecond apart, and the report divides them into 32 minor cycles. A minor "
   "cycle holds one number or one instruction. The 1024 digits of a whole line are "
   "a major cycle. `DL 3,5` is the fifth minor cycle of delay line 3.")

qa("What is a delay line?",
   "A tube of mercury, or possibly water, about five feet long with a quartz crystal "
   "at each end. Pulses put in at one end come out 1.024 milliseconds later, are "
   "amplified, reshaped, and put back in. Each line holds 1024 digits. A digit can "
   "only be used as it comes out of the end, so a word has to be waited for: up to "
   "a millisecond.")

qa("How are the digits of a word numbered on the ACE?",
   "1 to 32, in the order they come out of the delay line. Digit 1 is the least "
   "significant, and digit 32 has reversed sign, so a word of 32 ones is -1. The "
   "report prints words digit 1 first, in groups of four: "
   "`0000,0101,0000,0000,0100,0110,0000,0000`.")

qa("What are the temporary storages of the ACE?",
   "TS 1 to TS 32, delay lines one minor cycle long, so their content is always "
   "available. Several have duties: TS 1 receives the position left by an "
   "instruction of type B; TS 2 and TS 3 are the arguments of the arithmetic; TS 4 "
   "and TS 5 together are the 64-digit partial sum; TS 6 is the shunting station "
   "every transfer passes through; TS 7 says how far to rotate; TS 8 is the result "
   "of the logical operations, and TS 9 and TS 10 their inputs.")

qa("What are instructions of type A and type B?",
   "Digit 3 of an instruction tells them apart. Type A asks the arithmetic part to "
   "do something, and the next instruction is the next in sequence. Type B does "
   "nothing but name the next instruction, and passes the old position into TS 1. "
   "The report's own example: 'Instruction 491. A. Multiply the content of TS 23 by "
   "the content of TS 24 and store the result in TS 25. Then proceed to carry out "
   "the next instruction (i.e. No. 492). Instruction 492. B. Proceed with "
   "instruction 301.'")

qa("What are the types of operation K to T?",
   "The ten things an instruction of type A can ask for, chosen by digits 5-8 "
   "(Fig. 19): K, TS 6 into a minor cycle; L, a minor cycle into TS 6; M, a TS into "
   "TS 6; N, TS 6 into a TS; O, the first 12 minor cycles of a delay line onto a "
   "card; P, a card into a delay line; Q, digits 17-32 of the instruction into "
   "TS 6; R, a logical operation; S, arithmetic; T, stimulate a valve element.")

qa("What are the three forms of an instruction in Turing's report?",
   "Machine form, permanent form and popular form. Machine form is the instruction "
   "as the machine takes it, one minor cycle. Permanent form is a card kept in a "
   "library; the same permanent instruction becomes different machine forms in "
   "different jobs, because instructions are renumbered for each job. Popular form "
   "is the table printed on paper for people to read, much abbreviated: "
   "`TS 6 - TS 2`, `ADD 'A'`, `B, BURY`. The report adds a fourth thing, the "
   "general description of a table: what it does, where its inputs and results "
   "are, and which storages it alters.")

qa("How were instructions numbered for a job on the ACE?",
   "Section 7 of the report: take the cards of every table the job uses, copy them, "
   "sort them into the order of popular name and detail figure, and number them "
   "consecutively in the binary scale. Then every instruction that names another "
   "has that number filled into its genuine input, 'a straightforward sorting and "
   "collating process'. `ace` does the same:\n\n"
   + command("tables", "examples/echo.ace"))

qa("What are the columns of an ACE instruction card?",
   "Section 7 proposes: popular name of the table, columns 1-8; detail figure "
   "(popular), 9-11; the instruction in popular form, 12-25; repeat of "
   "destination, 26-40; genuine input, the 32 digits the machine takes, 41-72; "
   "job number, 73-77; spare, 78-80. `ace tables FILE --emit cards` prints a job "
   "that way.")

# ---------------------------------------------------------------- the notation

qa("What does a simple ACE job look like?",
   "A table name on a line of its own, then instructions with their detail "
   "figures. This one reads a card and punches it again:\n\n"
   + job(open(EXAMPLES / "echo.ace").read(), "1 2 3"))

qa("How do I move a word between storages on the ACE?",
   "Write `from - to`. Everything passes through TS 6, so a transfer that does not "
   "start or end at TS 6 is two instructions, which is why the detail figures "
   "below go 1, 3, 5:\n\n"
   + job("FETCH\n  1   DL 3,5 - TS 20\n  3   TS 20 - DL 2,1\n  5   O, DL 2\n"
         "  6   B, FETCH 6\n\nDL 3,5  1945"))

qa("How does an ACE job stop?",
   "The report gives the machine no stop instruction. A job ends with an "
   "instruction of type B that passes control to itself, such as `B, ECHO 3` at "
   "detail figure 3 of ECHO; `ace run` treats that as the end. A job that asks for "
   "a card when the reader is empty also stops.")

qa("How do I put a number into the machine before an ACE job starts?",
   "A line naming a storage and giving it a content: `DL 3,5  1945`, `TS 13  -7`, "
   "a word in binary `DL 1,15  0000,0000,0000,0000,0000,0100,0000,0000`, an "
   "instruction `DL 1,14  L, DL 3,1`, or a position `TS 27  DL 3,5`. CALPOL in the "
   "report says 'Before starting we require special contents in DL 1, 14 and "
   "DL 1, 15'.")

qa("How do I add two numbers on the ACE?",
   "Put the first in TS 2 and use `ADD 'A'`, which the report explains as 'Add TS 2 "
   "into TS 4 cancelling the partial sums'. Put the second in TS 2 and use `ADD`. "
   "The sum is in TS 4.\n\n"
   + job(f"# cards: listing\nSUM\n  1   P, DL 2\n  2   DL 2,1 - TS 2\n  4   ADD 'A'\n"
         f"  5   DL 2,2 - TS 2\n  7   ADD\n  8   TS 4 - TS 13\n 10   {SHOW}\n"
         f" 15   B, SUM 15", "19 23"))

qa("How do I multiply on the ACE?",
   "`MULTIPLY 'A'` multiplies TS 2 by TS 3 into the cancelled partial sum, TS 4 "
   "and TS 5. It takes 16 minor cycles. For a multiplier under 16, short "
   "multiplication is one quick instruction with the multiplier written in it: "
   "`S, POS, CANCEL, GIV, 6` is six times TS 2.\n\n"
   + job(f"# cards: listing\nPRODUCT\n  1   P, DL 2\n  2   DL 2,1 - TS 2\n"
         f"  4   DL 2,2 - TS 3\n  6   MULTIPLY 'A'\n  7   TS 4 - TS 13\n  9   {SHOW}\n"
         f" 14   B, PRODUCT 14", "6 7"))

qa("Does the ACE have a division instruction?",
   "No. Section 12: 'Division is an ITO and will probably be carried out by means of "
   "the recurrence relation u(n+1) = u(n)(2 - a u(n))', that is, by an instruction "
   "table. A job can also divide by repeated subtraction, as Euclid's algorithm "
   "does.")

qa("What are the signals of an arithmetic instruction on the ACE?",
   "Type S is one process with switches, set by digits of the instruction: POS "
   "(add rather than subtract), CANCEL (clear the partial sum first), SELF (the "
   "multiplicand is the partial sum), NOR (four digits of TS 3 are the "
   "multiplier), GIV (the multiplier is in digits 24-27 of the instruction), ERR "
   "(it is in TS 11), DIFF, AUTO (repeat eight times: long multiplication) and "
   "ROTATE. Any arithmetic can be written by its signals: `S, POS, CANCEL, GIV, 1` "
   "is `ADD 'A'`.")

qa("What logical operations does the ACE have?",
   "Five, digit by digit on TS 9 and TS 10 into TS 8: `AND`, `OR`, `NOT` (of "
   "TS 10), `IFF` (1 where the digits agree) and `NEVER` (0).\n\n"
   + job(open(EXAMPLES / "logic.ace").read()))

qa("How does a program choose between two paths on the ACE?",
   "By calculating the instruction to obey. The report's table DISCRIM passes TS 15 "
   "into TS 24 if TS 8 has any digit 1, and TS 16 otherwise. Keep two instructions "
   "of type B in TS 15 and TS 16, call DISCRIM, move TS 24 into TS 6, and obey it "
   "with `B, 1`.\n\n"
   + job(open(EXAMPLES / "countdown.ace").read(), "3"))

qa("What does B, 1 mean in an ACE instruction table?",
   "Type B with a 1 in column 17: carry out the instruction in TS 6. If it is of "
   "type A, the machine then goes on at the instruction after the B. It is how a "
   "program obeys an instruction it has built. `B, 1, X` names an instruction "
   "after it as well, as INDEXIN 11 does.\n\n"
   + job(open(EXAMPLES / "from-ts6.ace").read()))

qa("What are BURY and UNBURY?",
   "Turing's tables for subroutine calls, which the report calls subsidiary "
   "operations. A note of where the main table left off is buried in a list kept "
   "in a delay line, 'with the most recent last', whose position is in TS 31. "
   "UNBURY takes the most recent note off and continues there. A call is two "
   "instructions, `B, BURY;  B, X`, and a subsidiary ends with `B, UNBURY`. The "
   "report describes both tables but prints neither; the library's are written to "
   "its description.")

qa("How do I write a subroutine on the ACE?",
   "Write a table that ends `B, UNBURY` and call it with `B, BURY;  B, NAME`.\n\n"
   + job("# cards: listing\nMAIN\n  1   B, BURY\n  2   B, TWICE\n  3   B, BURY\n"
         "  4   B, TWICE\n  5   B, BURY\n  6   B, BINDEC\n  7   O, DL 10\n  8   B, MAIN 8\n\n"
         "TWICE\n  1   TS 13 - TS 2\n  3   S, POS, CANCEL, GIV, 2\n  4   TS 4 - TS 13\n"
         "  6   B, UNBURY\n\nTS 13   21"))

qa("How does the ACE print a decimal number?",
   "With BINDEC, which the report describes: the number is translated into DL 10 so "
   "that, punched and listed on a tabulator, its decimal digits appear 'in the "
   "usual way'. Each digit is a hole in the row for that digit. The library's "
   "BINDEC does whole numbers of up to nine digits, in TS 13.\n\n"
   + job("SHOW\n  1   B, BURY\n  2   B, BINDEC\n  3   O, DL 10\n  4   B, SHOW 4\n\n"
         "TS 13   -1945"))

# ---------------------------------------------------------------- time

qa("Why are consecutive ACE instructions two minor cycles apart?",
   "So that the next instruction has not gone by when the machine is ready for it. "
   "Section 11: 'If the equation were the one with P18 it would mean that the next "
   "instruction to be obeyed, after one of type A, is always adjacent to it in "
   "time. This would mean that even with the shortest CA operations the next "
   "instruction would have gone by before we were ready to apply it; we should "
   "always just miss the boat.' So the position of the next instruction is found "
   "by adding a pulse in digit 19, and the digits of a position are weighted "
   "16, 1, 2, 4, 8 so that consecutive numbers are two minor cycles apart.")

qa("Does it matter where a number is kept on the ACE?",
   "Very much. A word in a delay line can only be read as it comes round, and while "
   "the machine waits, the next instruction may go by too. These two jobs fetch "
   "four numbers each; in the first every number arrives just as it is wanted:\n\n"
   + command("run", "tutorial/jobs/placed.ace", "--time") + "\n\n"
   + command("run", "tutorial/jobs/unplaced.ace", "--time"))

# ---------------------------------------------------------------- Turing's tables

qa("What is INDEXIN?",
   "The first instruction table printed in the report, 'used for finding a minor "
   "cycle whose position has been written down in a particular place': the minor "
   "cycle whose position is in digits 17-32 of TS 27 is transferred to TS 28. It "
   "builds an instruction of type L from the position and obeys it from TS 6. As "
   "printed:\n\n" + command("library", "INDEXIN", "--errata", "none"))

qa("Does Turing's INDEXIN work as printed?",
   "No. Run on a simulation of the report's machine it fetches nothing and runs "
   "off into reading cards. It needs three corrections: line 1 makes the code of "
   "type K (store) instead of type L (fetch), noted by Carpenter and Doran in "
   "1986; line 5 takes TS 4 where ROTATE 16 has moved the code into TS 5; and the "
   "table ends `B, BURY` instead of `B, UNBURY`, also noted by Carpenter and "
   "Doran. Leaving out any one of the three stops it working.")

qa("What is wrong with Fig. 38 of Turing's report?",
   "Fig. 38 prints INDEXIN a second time, as the 13 instruction cards that would "
   "be punched for it. Three do not agree with the table: the card for ADD 'A' also "
   "sets NOR, the card for OR has the code of AND, and the card for `TS 8 - TS 6` "
   "is `TS 4 - TS 6`. Run as the cards have it, INDEXIN fails as the printed table "
   "does.")

qa("What is CALPOL?",
   "The second instruction table printed in the report: it calculates a polynomial "
   "of degree 15 whose coefficients are in DL 3, for the argument in TS 13, 14, by "
   "repeated multiplication and addition, calling INDEXIN, PLUSIND, DISCRIM, "
   "TRANS 45, MULTIP and ADD as subsidiaries. As printed it never stops; with five "
   "corrections it works. Carpenter and Doran analysed its errors in 1977.\n\n"
   + job(open(EXAMPLES / "polynomial.ace").read()))

qa("Where does Turing's report contradict itself?",
   "In several details a simulation needs. It names TS 13 and TS 1 as where type B "
   "leaves the old position; TS 9 and 10, and TS 2 and 3, as the inputs of the "
   "logical operations; digits 23-26 and 24-27 for a given multiplier; digits 1-5 "
   "and 13-17 of TS 7 for the rotation. Carpenter and Doran's 1986 edition notes "
   "several of these.\n\n" + command("decisions"))

# ---------------------------------------------------------------- the examples

TASKS = {
    "echo": "Write an ACE job that reads a card and punches it again.",
    "blank": "What does a completely blank instruction do on the ACE?",
    "answer": "Write an ACE job that multiplies six by seven and punches the answer in decimal.",
    "add": "Write an ACE job that reads two numbers and punches their sum.",
    "multiply": "Write an ACE job that multiplies two numbers from a card.",
    "negative": "How do I make a number negative on the ACE?",
    "short-multiplication": "Show me short multiplication on the ACE.",
    "logic": "Show me every logical operation of the ACE.",
    "from-ts6": "Write an ACE job that builds an instruction and obeys it.",
    "fetch": "Write an ACE job that uses INDEXIN.",
    "countdown": "Write an ACE job that counts down.",
    "sum-to-n": "Write an ACE job that adds up 1 to n.",
    "factorial": "Write an ACE job that computes a factorial.",
    "fibonacci": "Write an ACE job that punches Fibonacci numbers.",
    "gcd": "Write Euclid's algorithm for the ACE.",
    "polynomial": "Write an ACE job that uses Turing's CALPOL.",
}
missing = {p.stem for p in EXAMPLES.glob("*.ace")} ^ set(TASKS)
if missing:
    sys.exit(f"examples and TASKS differ: {sorted(missing)}")
for name, question in TASKS.items():
    src = (EXAMPLES / f"{name}.ace").read_text()
    stdin_file = EXAMPLES / f"{name}.in"
    stdin = stdin_file.read_text() if stdin_file.exists() else ""
    qa(question, job(src, stdin))

out = HERE / "dataset" / "ace.jsonl"
out.parent.mkdir(exist_ok=True)
with out.open("w") as f:
    for row in rows:
        f.write(json.dumps(row, ensure_ascii=False) + "\n")
print(f"wrote {len(rows)} examples to {out}")
