// Runs every example through the WebAssembly build and the browser's WASI
// host, and compares what it punches with the .out file.   node web/test.mjs
import { readFileSync, readdirSync, existsSync } from "node:fs";
import { run, stdinFor } from "./wasi.js";
import { ask } from "./lsp.js";

const root = new URL("..", import.meta.url).pathname;
const module = new WebAssembly.Module(readFileSync(root + "web/ace.wasm"));

function ace(args, program = "", input = "") {
  let out = "";
  let err = "";
  const decoder = new TextDecoder();
  const code = run(module, stdinFor(program, input), (fd, bytes) => {
    const text = decoder.decode(bytes, { stream: true });
    if (fd === 1) out += text;
    else err += text;
  }, args);
  return { code, out, err };
}

let checked = 0;
let failed = 0;
const fail = (what) => {
  failed++;
  console.log(`FAIL ${what}`);
};

const names = readdirSync(root + "examples").filter((f) => f.endsWith(".ace")).sort();
for (const file of names) {
  const name = file.slice(0, -4);
  const program = readFileSync(`${root}examples/${file}`, "utf8");
  const inFile = `${root}examples/${name}.in`;
  const input = existsSync(inFile) ? readFileSync(inFile, "utf8") : "";
  const expected = readFileSync(`${root}examples/${name}.out`, "utf8");
  const { code, out } = ace(["run", "-"], program, input);
  if (code !== 0 || out !== expected) fail(`${name} (exit ${code})\n--- expected\n${expected}--- got\n${out}`);
  // The step view's JSON agrees with the plain run.
  const json = ace(["run", "-", "--json"], program, input);
  const result = JSON.parse(json.out);
  const cards = result.cards.map((c) => c + "\n").join("");
  if (json.code !== 0 || !result.idle || cards !== expected) fail(`${name} --json`);
  checked++;
}

// The page's list holds every example and the library.
const listed = JSON.parse(readFileSync(root + "web/examples.json", "utf8"));
if (listed.examples.length !== checked) fail(`examples.json lists ${listed.examples.length} jobs, not ${checked}`);
if (!listed.library.some((t) => t.name === "INDEXIN" && t.printed)) fail("examples.json: INDEXIN missing from the library");
// The two timing jobs: the same fetches, one placed in time and one not.
const cycles = (job) => Number(/in (\d+) minor cycles/.exec(ace(["run", "-", "--time"], job.program).out)?.[1]);
const [placed, unplaced] = ["placed", "unplaced"].map((n) => listed.timing.find((t) => t.name === n));
if (!placed || !unplaced || !(cycles(placed) < cycles(unplaced))) fail("the timing jobs");

// An error comes back as a message and a non-zero exit code.
const refused = ace(["run", "-"], "JOB\n  1  TS 6 = TS 2\n");
if (refused.code !== 1 || !refused.err.includes("`=` is not used")) fail(`error reporting: ${refused.err}`);
const looping = ace(["run", "-", "--limit", "1000"], "JOB\n  1  B, JOB 2\n  2  B, JOB 1\n");
if (looping.code !== 1 || !looping.err.includes("step limit")) fail(`the step limit: ${looping.err}`);

// The other commands the page offers.
const fetchJob = listed.examples.find((e) => e.name === "fetch").program;
const listing = ace(["tables", "-"], fetchJob);
if (listing.code !== 0 || !listing.out.includes("INDEXIN  (library: printed in the report)")) fail("tables");
const cards = ace(["tables", "-", "--emit", "cards"], fetchJob);
if (cards.code !== 0 || !cards.out.split("\n")[1] || cards.out.split("\n")[1].length !== 80) fail("tables --emit cards");
const audit = ace(["audit"]);
if (audit.code !== 0 || !audit.out.includes("3 of the 13 cards")) fail("audit");
const printed = ace(["library", "INDEXIN", "--errata", "none"]);
if (!printed.out.includes("Q, 0000,0100,0000,0000")) fail("library as printed");
const corrected = ace(["library", "INDEXIN"]);
if (!corrected.out.includes("Q, 0000,0101,0000,0000")) fail("library corrected");
const asPrinted = ace(["run", "-", "--errata", "none", "--limit", "5000"], fetchJob);
if (asPrinted.code !== 1) fail("fetch with INDEXIN as printed should not finish");

// The language server, as the page calls it.
const bad = "JOB\n  1   TS 4 - TS 6\n  3   TS 6 - TS 8\n  4   B, JOB 4\n";
const answer = ask(module, bad, [
  { method: "textDocument/semanticTokens/full" },
  { method: "textDocument/hover", params: { position: { line: 1, character: 11 } } },
  { method: "textDocument/hover", params: { position: { line: 3, character: 10 } } },
]);
if (answer.diagnostics.length !== 1 || !answer.diagnostics[0].message.startsWith("TS 8 takes nothing")) fail(`language server: problems ${JSON.stringify(answer.diagnostics)}`);
if (!(answer.results[0].data.length > 20) || answer.results[0].data.length % 5) fail("language server: colours");
if (!answer.results[1]?.contents.value.includes("On the machine:")) fail(`language server: hover on an instruction ${JSON.stringify(answer.results[1])}`);
if (!answer.results[2]?.contents.value.includes("**JOB**")) fail("language server: hover on a table");
const clean = ask(module, fetchJob, [{ method: "textDocument/completion", params: { position: { line: 1, character: 0 } } }]);
if (clean.diagnostics.length !== 0 || !clean.results[0].some((item) => item.label === "INDEXIN")) fail("language server: completion");

// The machine room (console.js) plays `ace run --json --cold`: from an empty
// machine, the deck and then the job, punching what a warm run punches.
{
  const { listing } = await import("./console.js");
  for (const name of ["answer", "echo", "gcd"]) {
    const program = readFileSync(`${root}examples/${name}.ace`, "utf8");
    const inFile = `${root}examples/${name}.in`;
    const input = existsSync(inFile) ? readFileSync(inFile, "utf8") : "";
    const warm = JSON.parse(ace(["run", "-", "--json"], program, input).out);
    const cold = JSON.parse(ace(["run", "-", "--json", "--cold"], program, input).out);
    if (!cold.idle || cold.deck < 2 || cold.start.cells.length !== 0) fail(`${name} from cold: ${cold.stop}, deck ${cold.deck}`);
    if (JSON.stringify(cold.raw) !== JSON.stringify(warm.raw)) fail(`${name} from cold punches something else`);
    if (cold.start.reader.length < cold.deck) fail(`${name} from cold: the deck is not in the reader`);
  }
  const card = Array(12).fill(0);
  card[1] = 1;      // row 11, column 1: minus
  card[2] = 2;      // row 0, column 2
  card[6] = 4;      // row 4, column 3
  if (listing(card) !== "-04") fail(`console listing ${JSON.stringify(listing(card))}`);
}

console.log(`${checked} examples checked through WebAssembly, ${failed} failed`);
process.exit(failed ? 1 : 0);
