// Runs the extension inside a real VS Code and drives it:  node test/run-vscode.js
// The first run downloads a copy of VS Code into .vscode-test/.
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { runTests } = require("@vscode/test-electron");

(async () => {
  const root = path.join(__dirname, "..");
  if (!fs.existsSync(path.join(root, "dist/extension.js"))) throw new Error("build the extension first: npm run build");
  // No settings: the extension must work with the program it carries.
  const workspace = fs.mkdtempSync(path.join(os.tmpdir(), "ace-vscode-"));
  fs.copyFileSync(path.join(root, "../../examples/fetch.ace"), path.join(workspace, "fetch.ace"));
  fs.writeFileSync(
    path.join(workspace, "job.ace"),
    "# Fetch a number with INDEXIN.\nJOB\n  1   B, BURY\n  2   B, INDEXIN\n  3   TS 28 - DL 2,1\n  5   O, DL 2\n  6   B, JOB 6\n\nTS 27   L, DL 3,5\nDL 3,5  42\n",
  );
  await runTests({
    extensionDevelopmentPath: root,
    extensionTestsPath: path.join(__dirname, "suite.js"),
    // A short user data folder: the editor's socket path must stay under 104 characters.
    launchArgs: [workspace, "--disable-extensions", "--disable-workspace-trust", `--user-data-dir=${fs.mkdtempSync("/tmp/ace-ud-")}`],
  });
})().catch((error) => {
  console.error(error.message || error);
  process.exit(1);
});
