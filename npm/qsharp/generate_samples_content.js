// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// @ts-check

import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import qSharpSampleList from "../../samples/samples.mjs";
import openQasmSampleList from "../../samples/OpenQASM/samples.mjs";

const thisDir = dirname(fileURLToPath(import.meta.url));
const qSharpSampleDir = join(thisDir, "..", "..", "samples");
const openQasmSampleDir = join(thisDir, "..", "..", "samples", "OpenQASM");

const tsDir = join(thisDir, "src");
const qSharpGeneratedTsPath = join(tsDir, "samples.generated.ts");
const openQasmGeneratedTsPath = join(tsDir, "openqasm-samples.generated.ts");

embedSampleContentsInTsFile(
  qSharpSampleList,
  qSharpSampleDir,
  qSharpGeneratedTsPath,
);
embedSampleContentsInTsFile(
  openQasmSampleList,
  openQasmSampleDir,
  openQasmGeneratedTsPath,
);

/**
 * @param {any[]} sampleList
 * @param {string} sampleDir
 * @param {import("fs").PathOrFileDescriptor} generatedTsFileName
 */
function embedSampleContentsInTsFile(
  sampleList,
  sampleDir,
  generatedTsFileName,
) {
  const result = sampleList.map((sample) => {
    const samplePath = join(sampleDir, sample.file);
    const sampleText = readFileSync(samplePath, "utf8");
    return {
      title: sample.title,
      shots: sample.shots,
      code: sampleText,
      omitFromTests: sample.omitFromTests,
    };
  });

  writeFileSync(
    generatedTsFileName,
    `export default ${JSON.stringify(result, undefined, 2)}`,
    "utf-8",
  );
}
