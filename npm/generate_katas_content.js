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

import { katas } from "../katas/content/katas.js";

const thisDir = dirname(fileURLToPath(import.meta.url));
const katasContentDir = join(thisDir, "..", "katas", "content");
const katasGeneratedContentDir = join(thisDir, "src");
const katasContentFileNames = {
  index: "content.md",
  qsharpExample: "example.qs",
  qsharpPlaceholder: "placeholder.qs",
  qsharpSolution: "reference.qs",
  qsharpVerify: "verify.qs",
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

function buildExampleContent(id, directory) {
  const source = readFileSync(join(directory, "example.qs"), "utf8");
  const contentAsMarkdown = readFileSync(join(directory, "content.md"), "utf8");
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

function buildExerciseContent(id, directory) {
  const placeholderSource = readFileSync(
    join(directory, "placeholder.qs"),
    "utf8"
  );
  const referenceSource = readFileSync(join(directory, "reference.qs"), "utf8");
  const verificationSource = readFileSync(join(directory, "verify.qs"), "utf8");
  const contentAsMarkdown = readFileSync(join(directory, "content.md"), "utf8");
  const contentAsHtml = marked.parse(contentAsMarkdown);
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
  };
}

function buildReadingContent(id, directory) {
  const contentAsMarkdown = readFileSync(join(directory, "content.md"), "utf8");
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

function getItemType(itemDir) {
  const itemTypeFileSets = {
    reading: new Set([katasContentFileNames.index]),
    example: new Set([
      katasContentFileNames.index,
      katasContentFileNames.qsharpExample,
    ]),
    exercise: new Set([
      katasContentFileNames.index,
      katasContentFileNames.qsharpPlaceholder,
      katasContentFileNames.qsharpSolution,
      katasContentFileNames.qsharpVerify,
    ]),
  };

  const itemFiles = new Set(readdirSync(itemDir));
  return Object.keys(itemTypeFileSets).find(function (key) {
    const fileSet = itemTypeFileSets[key];
    return symmetricDifference(fileSet, itemFiles).size === 0;
  });
}

function buildItemContent(itemDir) {
  const itemId = `${basename(dirname(itemDir))}__${basename(itemDir)}`;
  const itemType = getItemType(itemDir);
  if (itemType === "example") {
    return buildExampleContent(itemId, itemDir);
  } else if (itemType === "exercise") {
    return buildExerciseContent(itemId, itemDir);
  } else if (itemType === "reading") {
    return buildReadingContent(itemId, itemDir);
  }

  throw new Error(`Unknown module type ${itemType}`);
}

function buildKataContent(kata, katasDir) {
  const kataDir = join(katasDir, kata.directory);
  const itemsJson = readFileSync(join(kataDir, "items.json"), "utf8");
  const items = JSON.parse(itemsJson);
  let itemsContent = [];
  for (const item of items) {
    const itemDir = join(kataDir, item);
    const itemContent = buildItemContent(itemDir);
    itemsContent.push(itemContent);
  }

  const contentAsMarkdown = readFileSync(join(kataDir, "content.md"), "utf8");
  const contentAsHtml = marked.parse(contentAsMarkdown);
  const title = getTitleFromMarkdown(contentAsMarkdown);
  return {
    id: kata.directory,
    title: title,
    contentAsMarkdown: contentAsMarkdown,
    contentAsHtml: contentAsHtml,
    items: itemsContent,
  };
}

function buildKatasContentJs(katasDir, outDir) {
  console.log("Building katas content");
  var katasContent = [];
  for (const kata of katas) {
    var kataContent = buildKataContent(kata, katasDir);
    katasContent.push(kataContent);
  }

  if (!existsSync(outDir)) {
    mkdirSync(outDir);
  }

  const contentJsPath = join(outDir, "katas-content.generated.ts");
  writeFileSync(
    contentJsPath,
    "export const katas = " + inspect(katasContent, { depth: null }),
    "utf-8"
  );
}

buildKatasContentJs(katasContentDir, katasGeneratedContentDir);
