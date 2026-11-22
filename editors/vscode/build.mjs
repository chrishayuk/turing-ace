// Builds everything the extension ships:  node build.mjs
//   server/ace.wasm     the ace program, compiled for WASI
//   dist/extension.js   the extension and its one dependency, bundled
import { execSync } from "node:child_process";
import { copyFileSync, mkdirSync } from "node:fs";
import { build } from "esbuild";

const here = new URL(".", import.meta.url).pathname;
const root = new URL("../..", import.meta.url).pathname;

execSync("cargo build --release --target wasm32-wasip1", { cwd: root, stdio: "inherit" });
copyFileSync(`${root}target/wasm32-wasip1/release/ace.wasm`, `${here}server/ace.wasm`);

mkdirSync(`${here}dist`, { recursive: true });
await build({
  entryPoints: [`${here}src/extension.js`],
  outfile: `${here}dist/extension.js`,
  bundle: true,
  platform: "node",
  format: "cjs",
  target: "node18",
  external: ["vscode"],
  minify: true,
  logLevel: "warning",
});
console.log("server/ace.wasm and dist/extension.js written");
