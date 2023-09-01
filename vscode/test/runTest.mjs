// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// Adapted from the sample at
// https://github.com/microsoft/vscode-test-web/blob/3f0f858ab15cb65ef3c19564b0f5a6910ea9414e/sample/src/web/test/runTest.ts

// @ts-check

import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { runTests } from "@vscode/test-web";

const thisDir = dirname(fileURLToPath(import.meta.url));

async function main() {
  try {
    // The folder containing the Extension Manifest package.json
    const extensionDevelopmentPath = join(thisDir, "..");

    // The path to module with the test runner and tests
    const extensionTestsPath = join(thisDir, "./out/index");

    const folderPath = join(thisDir, "test-workspace");

    const attachArgName = "--waitForDebugger=";
    const waitForDebugger = process.argv.find((arg) =>
      arg.startsWith(attachArgName)
    );

    // Start a web server that serves VSCode in a browser, run the tests
    await runTests({
      headless: true, // pass false to see VS Code UI
      browserType: "chromium",
      extensionDevelopmentPath,
      extensionTestsPath,
      folderPath,
      waitForDebugger: waitForDebugger
        ? Number(waitForDebugger.slice(attachArgName.length))
        : undefined,
    });
  } catch (err) {
    console.error("Failed to run tests", err);
    process.exit(1);
  }
}

main();
