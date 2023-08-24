// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// imports mocha for the browser, defining the `mocha` global.
require("mocha/mocha");
import defineExtensionTests from "./extension.test";

export function run(): Promise<void> {
  return new Promise((c, e) => {
    mocha.setup({
      ui: "tdd",
      reporter: undefined,
    });

    defineExtensionTests();

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
