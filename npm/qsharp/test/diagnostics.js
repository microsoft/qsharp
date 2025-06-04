// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//@ts-check

import assert from "node:assert/strict";
import { test } from "node:test";
import { QdkDiagnostics } from "../dist/diagnostics.js";
import { log } from "../dist/log.js";
import {
  getCompiler,
  getCompilerWorker,
  getProjectLoader,
} from "../dist/main.js";

/** @type {import("../dist/log.js").TelemetryEvent[]} */
const telemetryEvents = [];
log.setLogLevel("warn");
log.setTelemetryCollector((event) => telemetryEvents.push(event));

/**
 * @returns {import ("../dist/compiler/compiler.js").ProgramConfig}
 */
function getInvalidQirProgramConfig() {
  /** @type {[string, string][]} */
  const sources = [
    ["test.qs", `namespace Test { function Main() : Int { return 1; } }`],
  ];
  return {
    sources,
    languageFeatures: [],
    profile: "base",
  };
}

test("getQir throws QdkDiagnostics", async () => {
  const compiler = getCompiler();
  const invalidConfig = getInvalidQirProgramConfig();
  await assert.rejects(
    () => compiler.getQir(invalidConfig),
    (err) => {
      assert(err instanceof QdkDiagnostics, "Error should be QdkDiagnostics");
      assert(err.diagnostics.length > 0, "diagnostics should not be empty");
      assert.match(err.message, /cannot use an integer value as an output/);
      return true;
    },
    "getQir should throw on invalid input",
  );
});

test("getQir throws QdkDiagnostics - worker", async () => {
  const compiler = getCompilerWorker();
  const invalidConfig = getInvalidQirProgramConfig();
  try {
    await assert.rejects(
      () => compiler.getQir(invalidConfig),
      (err) => {
        assert(err instanceof QdkDiagnostics, "Error should be QdkDiagnostics");
        assert(err.diagnostics.length > 0, "diagnostics should not be empty");
        assert.match(err.message, /cannot use an integer value as an output/);
        return true;
      },
      "getQir should throw on invalid input",
    );
  } finally {
    compiler.terminate();
  }
});

// Minimal IProjectHost implementation for testing
const dummyHost = {
  readFile: async () => null,
  listDirectory: async () => [],
  resolvePath: async (a, b) => b,
  fetchGithub: async () => "",
  findManifestDirectory: async () => null,
};

test("loadQSharpProject throws QdkDiagnostics", async () => {
  const loader = getProjectLoader(dummyHost);
  await assert.rejects(
    () => loader.loadQSharpProject("/not/a/real/dir"),
    (err) => {
      assert(err instanceof QdkDiagnostics, "Error should be QdkDiagnostics");
      assert(err.diagnostics.length > 0, "diagnostics should not be empty");
      assert.match(err.message, /Failed to parse manifest/i);
      return true;
    },
    "loadQSharpProject should throw on invalid directory",
  );
  loader.dispose();
});
