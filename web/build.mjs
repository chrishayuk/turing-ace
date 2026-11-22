// Builds the files the playground needs:  node web/build.mjs
//   web/ace.wasm       the ace program, compiled for WASI
//   web/examples.json  the example jobs with their cards, and the names of
//                      the tables in the library
import { execFileSync, execSync } from "node:child_process";
import { copyFileSync, existsSync, readFileSync, readdirSync, writeFileSync } from "node:fs";

const root = new URL("..", import.meta.url).pathname;
execSync("cargo build --release --target wasm32-wasip1", { cwd: root, stdio: "inherit" });
execSync("cargo build --release", { cwd: root, stdio: "inherit" });
copyFileSync(`${root}target/wasm32-wasip1/release/ace.wasm`, `${root}web/ace.wasm`);

// Shown first, in this order, where they exist; the rest follow alphabetically.
const first = ["answer", "fetch", "add", "sum-to-n", "polynomial", "factorial", "fibonacci", "gcd"];

const examples = readdirSync(`${root}examples`).filter((f) => f.endsWith(".ace")).map((f) => {
  const name = f.slice(0, -4);
  const input = `${root}examples/${name}.in`;
  return {
    name,
    program: readFileSync(`${root}examples/${f}`, "utf8"),
    input: existsSync(input) ? readFileSync(input, "utf8") : "",
  };
});
examples.sort((a, b) => {
  const [i, j] = [first.indexOf(a.name), first.indexOf(b.name)];
  if (i !== -1 || j !== -1) return (i === -1 ? 99 : i) - (j === -1 ? 99 : j);
  return a.name.localeCompare(b.name);
});

// Two jobs from the tutorial that show what waiting for a delay line costs:
// the same four fetches, from minor cycles placed to arrive just in time and
// from minor cycles that have just gone by.
const timing = ["placed", "unplaced"].filter((name) => existsSync(`${root}tutorial/jobs/${name}.ace`)).map((name) => ({
  name,
  program: readFileSync(`${root}tutorial/jobs/${name}.ace`, "utf8"),
  input: "",
}));

// `ace library` lists the tables: name, size, and whose table it is.
const ace = `${root}target/release/ace`;
const library = execFileSync(ace, ["library"], { encoding: "utf8" }).trim().split("\n").map((line) => {
  const [, name, size, whose] = /^(\S+)\s+(\d+) minor cycles\s+(.*)$/.exec(line);
  return { name, size: Number(size), printed: whose.startsWith("printed") };
});

writeFileSync(`${root}web/examples.json`, JSON.stringify({ examples, timing, library }));
console.log(`web/ace.wasm and web/examples.json (${examples.length} jobs, ${timing.length} timing jobs, ${library.length} library tables) written`);
