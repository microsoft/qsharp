// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// Adapted from the sample at
// https://github.com/microsoft/vscode-test-web/blob/3f0f858ab15cb65ef3c19564b0f5a6910ea9414e/sample/src/web/test/suite/index.ts
//
// This script runs in the VS Code for the Web extension host. Test discovery
// is performed by the call to `require()` below. If new test suites are added,
// `require()` calls for those new files should be added here. Glob imports are
// not supported in the current build environment, where the tests are bundled
// into a single file using esbuild.

// imports mocha for the browser, defining the `mocha` global.
require("mocha/mocha");

export function run(): Promise<void> {
  return new Promise((c, e) => {
    mocha.setup({
      ui: "tdd",
      reporter: undefined,
    });

    // Load the test suites. This needs to come
    // after the call to mocha.setup() so that the
    // suite() global is defined by mocha.
    require("./extension.test");

    try {
      // Run the mocha test
      mocha.run((failures) => {
        if (failures > 0) {
          e(new Error(`${failures} tests failed.`));
        } else {
          c();
        }
      });
    } catch (err) {
      console.error(err);
      e(err);
    }
  });
}
