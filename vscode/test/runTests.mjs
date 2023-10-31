// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// @ts-check

// Adapted from the sample at
// https://github.com/microsoft/vscode-test-web/blob/3f0f858ab15cb65ef3c19564b0f5a6910ea9414e/sample/src/web/test/runTest.ts
//
// This script is run using Node.js in the dev environment. It will
// download the latest Insiders build of VS Code for the Web and launch
// it in a headless instance of Chromium to run the integration test suite.

import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { runTests } from "@vscode/test-web";

const thisDir = dirname(fileURLToPath(import.meta.url));
// The folder containing the Extension Manifest package.json
const extensionDevelopmentPath = join(thisDir, "..");
const attachArgName = "--waitForDebugger=";
const waitForDebugger = process.argv.find((arg) =>
  arg.startsWith(attachArgName),
);
const verboseArgName = "--verbose";
const verbose = process.argv.includes(verboseArgName);

try {
  // Language service tests
  await runSuite(
    join(thisDir, "out", "language-service", "index"),
    join(thisDir, "suites", "language-service", "test-workspace"),
  );

  // Debugger tests
  await runSuite(
    join(thisDir, "out", "debugger", "index"),
    join(thisDir, "..", "..", "samples"),
  );
} catch (err) {
  console.error("Failed to run tests", err);
  process.exit(1);
}

/**
 * @param {string} extensionTestsPath - The path to module with the test runner and tests
 * @param {string} workspacePath - The path to the workspace to be opened in VS Code
 */
async function runSuite(extensionTestsPath, workspacePath) {
  // Start a web server that serves VS Code in a browser, run the tests
  await runTests({
    headless: true, // pass false to see VS Code UI
    browserType: "chromium",
    extensionDevelopmentPath,
    extensionTestsPath,
    folderPath: workspacePath,
    quality: "stable",
    printServerLog: verbose,
    verbose,
    waitForDebugger: waitForDebugger
      ? Number(waitForDebugger.slice(attachArgName.length))
      : undefined,
  });
}
