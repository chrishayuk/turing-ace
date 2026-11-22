#!/usr/bin/env python3
"""Check that everything the tutorial shows is what the tools really print.

An ```ace block is a complete job. It must be followed by an ```output
block, optionally with an ```input block in between: the numbers on the
cards in the reader. The job is run with `ace run` and what it punches is
compared.

A ```console block starts with a line `$ ace ...`. The command is run from
the top folder of the repository and the rest of the block compared with
what it prints.

    python3 check_tutorial.py            check
    python3 check_tutorial.py --write    rewrite the expected text from real runs

The program is taken from $ACE, or from ../target.
"""
import os
import pathlib
import re
import shlex
import subprocess
import sys
import tempfile

HERE = pathlib.Path(__file__).resolve().parent
ROOT = HERE.parent
BLOCK = re.compile(r"```(\w+)\n(.*?)```", re.S)


def binary():
    if os.environ.get("ACE"):
        return os.environ["ACE"]
    for build in ("release", "debug"):
        path = ROOT / "target" / build / "ace"
        if path.exists():
            return str(path)
    sys.exit("no program found: run `cargo build` or set $ACE")


def call(args, stdin=""):
    result = subprocess.run(args, input=stdin, capture_output=True, text=True,
                            timeout=60, cwd=ROOT)
    return (result.stdout + result.stderr).rstrip("\n")


def run_program(ace, program, stdin):
    with tempfile.NamedTemporaryFile("w", suffix=".ace", delete=False) as f:
        f.write(program)
    try:
        return call([ace, "run", f.name], stdin).replace(f.name, "job.ace")
    finally:
        os.unlink(f.name)


def main():
    write = "--write" in sys.argv
    ace = binary()
    checked = failed = 0
    for path in sorted(HERE.glob("*.md")):
        text = path.read_text()
        blocks = list(BLOCK.finditer(text))
        edits = []

        def compare(block, start, actual, what):
            nonlocal failed
            expected = text[start:block.end(2)].rstrip("\n")
            if actual == expected:
                return
            if write:
                edits.append((start, block.end(2), actual + "\n"))
            else:
                failed += 1
                print(f"{path.name}: {what}\n--- tutorial says\n{expected}\n"
                      f"--- the program prints\n{actual}\n")

        for i, block in enumerate(blocks):
            kind, body = block.group(1), block.group(2)
            if kind == "console":
                command, _, _ = body.partition("\n")
                if not command.startswith("$ ace"):
                    continue
                checked += 1
                args = [ace] + shlex.split(command[len("$ ace"):])
                compare(block, block.start(2) + len(command) + 1, call(args), command)
            elif kind == "ace":
                stdin, j = "", i + 1
                if j < len(blocks) and blocks[j].group(1) == "input":
                    stdin, j = blocks[j].group(2), j + 1
                if j >= len(blocks) or blocks[j].group(1) != "output":
                    print(f"{path.name}: job has no output block:\n{body}")
                    failed += 1
                    continue
                checked += 1
                actual = run_program(ace, body, stdin)
                compare(blocks[j], blocks[j].start(2), actual, f"output differs for:\n{body}")
        for start, end, new in reversed(edits):
            text = text[:start] + new + text[end:]
        if edits:
            path.write_text(text)
            print(f"{path.name}: rewrote {len(edits)} block(s)")
    print(f"{checked} blocks checked, {failed} failed")
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()
