// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// @ts-check

import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import samples from "../samples/samples.mjs";

const thisDir = dirname(fileURLToPath(import.meta.url));
const sampleDir = join(thisDir, "..", "samples");
const sampleGeneratedDir = join(thisDir, "src");

const result = samples.map((sample) => {
  const samplePath = join(sampleDir, sample.file);
  const sampleText = readFileSync(samplePath, "utf8");
  return {
    title: sample.title,
    shots: sample.shots,
    code: sampleText,
  };
});

const contentPath = join(sampleGeneratedDir, "samples.generated.ts");
writeFileSync(
  contentPath,
  `export default ${JSON.stringify(result, undefined, 2)}`,
  "utf-8",
);
