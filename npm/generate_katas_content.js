// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/**
 * Katas Taxonomy
 *
 * A Kata is a top-level container of educational items (exercises and examples) which are used to explain a particular
 * topic.
 *
 * This file builds the content for all the Katas. The katas ordering is conveyed by the katas.json file where each
 * string in the array represents a folder that contains all the data to build the kata.
 *
 * Each Kata is organized in a directory where an index.md file, an items.json file, and multiple sub-directories are
 * present. Each sub-directory represents an item within the Kata and its specific content depends on the type of item
 * it represents.
 */

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

function generateTextSection(markdown) {
  const html = marked.parse(markdown);
  return {
    type: "text",
    contentAsMarkdown: markdown,
    contentAsHtml: html,
  };
}

function generateSections(markdown) {
  const sections = [];
  const macroRegex = /@\[\w+\]\([^@]+\)\s+/g;
  let latestProcessedIndex = 0;
  while (latestProcessedIndex < markdown.length) {
    const matchArray = macroRegex.exec(markdown);
    if (matchArray !== null) {
      // If there is something between the last processed index and the start of the match, create a text section for
      // it.
      const delta = matchArray.index - latestProcessedIndex;
      if (delta > 0) {
        const textSection = generateTextSection(
          markdown.substring(latestProcessedIndex, matchArray.index)
        );
        sections.push(textSection);
      }

      // TODO: Create macro section.
      sections.push(matchArray[0]);
      latestProcessedIndex = macroRegex.lastIndex;
    } else {
      // No more matches were found, create a text section with the remaining content.
      const textSection = generateTextSection(
        markdown.substring(latestProcessedIndex, markdown.length)
      );
      sections.push(textSection);
      latestProcessedIndex = markdown.length;
    }
  }

  return sections;
}

function generateKataContent(path) {
  const kataId = basename(path);
  const indexFilePath = join(path, contentFileNames.index);
  if (!existsSync(indexFilePath)) {
    throw new Error(
      `${contentFileNames.index} file not found for kata at the ${path} directory`
    );
  }

  const katasMarkdown = readFileSync(indexFilePath, "utf8");
  const sections = generateSections(katasMarkdown);
  return {
    id: kataId,
    sections: sections,
  };
}

function generateKatasContent(katasPath, outputPath) {
  console.log("Generating katas content");
  const indexJson = readFileSync(join(katasPath, "index.json"), "utf8");
  const katasDirs = JSON.parse(indexJson);
  var katas = [];
  for (const kataDir of katasDirs) {
    const kataPath = join(katasPath, kataDir);
    const kata = generateKataContent(kataPath);
    katas.push(kata);
  }

  const katasContent = {
    katas: katas,
    codeDependencies: [],
  };

  // Save the JS object to a file.
  if (!existsSync(outputPath)) {
    mkdirSync(outputPath);
  }

  const contentJsPath = join(outputPath, "katas-content.new.generated.ts");
  writeFileSync(
    contentJsPath,
    "export const katasContent = " + inspect(katasContent, { depth: null }),
    "utf-8"
  );
}

generateKatasContent(katasContentPath, katasGeneratedContentPath);
