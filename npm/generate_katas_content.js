// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// @ts-check

import {
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  writeFileSync,
} from "node:fs";
import { basename, dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { inspect } from "node:util";

import { marked } from "marked";

const scriptDirPath = dirname(fileURLToPath(import.meta.url));
const katasContentPath = join(scriptDirPath, "..", "katas", "content");
const katasGeneratedContentPath = join(scriptDirPath, "src");
const contentFileNames = {
  index: "index.md",
  qsharpExample: "example.qs",
  qsharpPlaceholder: "placeholder.qs",
  qsharpSolution: "solution.qs",
  qsharpVerify: "verify.qs",
  textSolution: "solution.md",
};

function getTitleFromMarkdown(markdown) {
  const titleRe = /#+ /;
  const lines = markdown.split(/\r?\n/);
  if (lines.length === 0) {
    throw new Error("Could not parse title, markdown is empty");
  }

  const firstLine = lines[0];
  const match = firstLine.match(titleRe);
  if (match === null) {
    throw new Error(
      `First line does not follow the expected title pattern: ${firstLine}`
    );
  }

  return firstLine.replace(titleRe, "");
}

function buildExampleContent(id, path) {
  const source = readFileSync(
    join(path, contentFileNames.qsharpExample),
    "utf8"
  );
  const contentAsMarkdown = readFileSync(
    join(path, contentFileNames.index),
    "utf8"
  );
  const contentAsHtml = marked.parse(contentAsMarkdown);
  const title = getTitleFromMarkdown(contentAsMarkdown);
  return {
    type: "example",
    id: id,
    title: title,
    source: source,
    contentAsMarkdown: contentAsMarkdown,
    contentAsHtml: contentAsHtml,
  };
}

function buildExerciseContent(id, path) {
  const placeholderSource = readFileSync(
    join(path, contentFileNames.qsharpPlaceholder),
    "utf8"
  );
  const referenceSource = readFileSync(
    join(path, contentFileNames.qsharpSolution),
    "utf8"
  );
  const verificationSource = readFileSync(
    join(path, contentFileNames.qsharpVerify),
    "utf8"
  );
  const contentAsMarkdown = readFileSync(
    join(path, contentFileNames.index),
    "utf8"
  );
  const contentAsHtml = marked.parse(contentAsMarkdown);
  const solutionAsMarkdown = readFileSync(
    join(path, contentFileNames.textSolution),
    "utf8"
  );
  const solutionAsHtml = marked.parse(solutionAsMarkdown);
  const title = getTitleFromMarkdown(contentAsMarkdown);
  return {
    type: "exercise",
    id: id,
    title: title,
    placeholderImplementation: placeholderSource,
    referenceImplementation: referenceSource,
    verificationImplementation: verificationSource,
    contentAsMarkdown: contentAsMarkdown,
    contentAsHtml: contentAsHtml,
    solutionAsMarkdown: solutionAsMarkdown,
    solutionAsHtml: solutionAsHtml,
  };
}

function buildReadingContent(id, path) {
  const contentAsMarkdown = readFileSync(
    join(path, contentFileNames.index),
    "utf8"
  );
  const contentAsHtml = marked.parse(contentAsMarkdown);
  const title = getTitleFromMarkdown(contentAsMarkdown);
  return {
    type: "reading",
    id: id,
    title: title,
    contentAsMarkdown: contentAsMarkdown,
    contentAsHtml: contentAsHtml,
  };
}

function symmetricDifference(setA, setB) {
  const difference = new Set(setA);
  for (const elem of setB) {
    if (difference.has(elem)) {
      difference.delete(elem);
    } else {
      difference.add(elem);
    }
  }
  return difference;
}

function getItemType(path) {
  const itemTypeFileSets = {
    reading: new Set([contentFileNames.index]),
    example: new Set([contentFileNames.index, contentFileNames.qsharpExample]),
    exercise: new Set([
      contentFileNames.index,
      contentFileNames.qsharpPlaceholder,
      contentFileNames.qsharpSolution,
      contentFileNames.qsharpVerify,
      contentFileNames.textSolution,
    ]),
  };

  const itemFiles = new Set(readdirSync(path));
  return Object.keys(itemTypeFileSets).find(function (key) {
    const fileSet = itemTypeFileSets[key];
    return symmetricDifference(fileSet, itemFiles).size === 0;
  });
}

function buildItemContent(path) {
  const itemId = `${basename(dirname(path))}__${basename(path)}`;
  const itemType = getItemType(path);
  if (itemType === "example") {
    return buildExampleContent(itemId, path);
  } else if (itemType === "exercise") {
    return buildExerciseContent(itemId, path);
  } else if (itemType === "reading") {
    return buildReadingContent(itemId, path);
  }

  throw new Error(`Unknown module type ${itemType}`);
}

function buildKataContent(path) {
  const kataId = basename(path);
  const itemsJson = readFileSync(join(path, "items.json"), "utf8");
  const items = JSON.parse(itemsJson);
  let itemsContent = [];
  for (const item of items) {
    const itemDir = join(path, item);
    const itemContent = buildItemContent(itemDir);
    itemsContent.push(itemContent);
  }

  const contentAsMarkdown = readFileSync(
    join(path, contentFileNames.index),
    "utf8"
  );
  const contentAsHtml = marked.parse(contentAsMarkdown);
  const title = getTitleFromMarkdown(contentAsMarkdown);
  return {
    id: kataId,
    title: title,
    contentAsMarkdown: contentAsMarkdown,
    contentAsHtml: contentAsHtml,
    items: itemsContent,
  };
}

function buildKatasContentJs(katasPath, outputPath) {
  console.log("Building katas content");
  const katasJson = readFileSync(join(katasPath, "katas.json"), "utf8");
  const katasDirs = JSON.parse(katasJson);
  var katasContent = [];
  for (const kataDir of katasDirs) {
    const kataPath = join(katasPath, kataDir);
    var kataContent = buildKataContent(kataPath);
    katasContent.push(kataContent);
  }

  if (!existsSync(outputPath)) {
    mkdirSync(outputPath);
  }

  const contentJsPath = join(outputPath, "katas-content.generated.ts");
  writeFileSync(
    contentJsPath,
    "export const katas = " + inspect(katasContent, { depth: null }),
    "utf-8"
  );
}

buildKatasContentJs(katasContentPath, katasGeneratedContentPath);
