// @ts-check

/* This file creates a drop of the npm package, along with the playground as an example site.

Basically it copies the root `package.*` files along with the ./npm and ./playground directories,
but excludes files that should be installed via npm or copied from there (such as ./node_modules
and ./playground/public/libs).

It excludes all the Rust/.github/katas/samples/etc., files and puts a usage README.md at the root.
*/

import {
  copyFileSync,
  cpSync,
  mkdirSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { dirname, join } from "node:path";
import { exit } from "node:process";
import { fileURLToPath } from "node:url";

const thisDir = dirname(fileURLToPath(import.meta.url));
const targetDir = join(thisDir, "..", "qsharp-drop");

const dirStat = statSync(targetDir, { throwIfNoEntry: false });
if (dirStat) {
  console.error("Target directory already exists. Please delete first.");
  exit(1);
}

const buildStat = statSync(`${thisDir}/npm/dist/browser.js`);
if (!buildStat) {
  console.error("npm dist files not found. Build before running");
  exit(1);
}

mkdirSync(targetDir);

copyFileSync(`${thisDir}/package.json`, `${targetDir}/package.json`);
copyFileSync(`${thisDir}/package-lock.json`, `${targetDir}/package-lock.json`);

/**
 * @param {string} src The source file being copied
 */
function copyFilter(src) {
  // Don't copy over the unnecessary files
  if (src.includes("/npm/generate_")) return false;
  if (src.includes("/playground/public/libs")) return false;
  return true;
}

cpSync(`${thisDir}/npm`, `${targetDir}/npm`, {
  recursive: true,
  filter: copyFilter,
});
cpSync(`${thisDir}/playground`, `${targetDir}/playground`, {
  recursive: true,
  filter: copyFilter,
});

writeFileSync(
  `${targetDir}/README.md`,
  `# README

The npm package containing the Q# compiler is in the "./npm" directory

The sample playground site that shows it wired up is in the "./playground" directory.

To get the playground up and running:

- In the root directory run "npm install"
- In the "./playground" run "npm run build"
- To start the site, from within "./playground" run "npx serve ./public"

The main source file to show hooking up the compiler and Monaco on a web page is at "./playground/src/main.tsx"

The test files in "./npm/test/basics.js" show minimal examples of the qsharp compiler API usage.
`,
  { encoding: "utf8" },
);
