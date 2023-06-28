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
import { basename, dirname, join, relative, sep } from "node:path";
import { fileURLToPath } from "node:url";
import { inspect } from "node:util";

import { marked } from "marked";

const scriptDirPath = dirname(fileURLToPath(import.meta.url));
const katasContentPath = join(scriptDirPath, "..", "katas", "content");
const katasGeneratedContentPath = join(scriptDirPath, "src");
const contentFileNames = {
  index: "index.md",
  katasIndex: "index.json",
  kataMarkdown: "index.md",
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

function tryParseJSON(json, errorPrefix) {
  let parsed;
  try {
    parsed = JSON.parse(json);
  } catch (e) {
    throw new Error(`${errorPrefix}: ${e}`);
  }
  return parsed;
}

function tryReadFile(filePath, errorPrefix) {
  let content;
  try {
    content = readFileSync(filePath, "utf8");
  } catch (e) {
    throw new Error(`${errorPrefix}: ${e}`);
  }
  return content;
}

function getMissingProperties(properties, required) {
  return required.filter((property) => !Object.hasOwn(properties, property));
}

function generateExampleSection(kataPath, properties) {
  // Validate that the data contains the required properties.
  const requiredProperties = ["id", "codePath"];
  const missingProperties = getMissingProperties(
    properties,
    requiredProperties
  );
  if (missingProperties.length > 0) {
    throw new Error(
      `Example macro is missing the following properties: ${missingProperties}`
    );
  }

  // Generate the object using the macro properties.
  const codePath = join(kataPath, properties.codePath);
  const code = tryReadFile(
    codePath,
    "Could not read the contents of the example code file"
  );
  return {
    type: "example",
    id: properties.id,
    code: code,
  };
}

function getCodeDependencyId(codeDependencyPath, basePath) {
  return relative(basePath, codeDependencyPath).replace(sep, "__");
}

function generateCodeDependencies(paths, globalCodeSources) {
  const codeDependencies = [];
  for (const path of paths) {
    const id = getCodeDependencyId(path, globalCodeSources.basePath);
    if (!(id in globalCodeSources.sources)) {
      const code = tryReadFile(path, "Could not read code dependency");
      globalCodeSources.sources[id] = code;
      codeDependencies.push(id);
    }
  }
  return codeDependencies;
}

function generateExerciseSection(kataPath, properties, globalCodeSources) {
  // Validate that the data contains the required properties.
  const requiredProperties = [
    "id",
    "codeDependenciesPaths",
    "verificationSourcePath",
    "placeholderSourcePath",
    "solutionSourcePath",
    "solutionDescriptionPath",
  ];
  const missingProperties = getMissingProperties(
    properties,
    requiredProperties
  );
  if (missingProperties.length > 0) {
    throw new Error(
      `Exercise macro is missing the following properties: ${missingProperties}`
    );
  }

  // Generate the object using the macro properties.
  const resolvedCodeDependenciesPaths = properties.codeDependenciesPaths.map(
    (path) => join(kataPath, path)
  );
  const codeDependencies = generateCodeDependencies(
    resolvedCodeDependenciesPaths,
    globalCodeSources
  );
  const verificationCode = tryReadFile(
    join(kataPath, properties.verificationSourcePath),
    `Could not read verification code for exercise ${properties.id}`
  );
  const placeholderCode = tryReadFile(
    join(kataPath, properties.placeholderSourcePath),
    `Could not read placeholder code for exercise ${properties.id}`
  );
  const solutionCode = tryReadFile(
    join(kataPath, properties.solutionSourcePath),
    `Could not read solution code for exercise ${properties.id}`
  );
  const solutionDescriptionAsMarkdown = tryReadFile(
    join(kataPath, properties.solutionDescriptionPath),
    `Could not read solution description for exercise ${properties.id}`
  );
  const solutionDescriptionAsHtml = marked.parse(solutionDescriptionAsMarkdown);
  return {
    type: "exercise",
    id: properties.id,
    codeDependencies: codeDependencies,
    verificationCode: verificationCode,
    placeholderCode: placeholderCode,
    solutionCode: solutionCode,
    solutionDescriptionAsMarkdown: solutionDescriptionAsMarkdown,
    solutionDescriptionAsHtml: solutionDescriptionAsHtml,
  };
}

function generateMacroSection(kataPath, match, globalCodeSources) {
  const type = match.groups.type;
  const propertiesJson = match.groups.json;
  const properties = tryParseJSON(
    propertiesJson,
    `Invalid JSON for ${type} macro.`
  );
  if (type === "example") {
    return generateExampleSection(kataPath, properties);
  } else if (type === "exercise") {
    return generateExerciseSection(kataPath, properties, globalCodeSources);
  }

  throw new Error(`Unknown macro type ${type}`);
}

function generateTextSection(markdown) {
  const html = marked.parse(markdown);
  return {
    type: "text",
    contentAsMarkdown: markdown,
    contentAsHtml: html,
  };
}

function generateSections(kataPath, markdown, globalCodeSources) {
  const sections = [];
  const macroRegex = /@\[(?<type>\w+)\]\((?<json>[^@]+)\)\s+/g;
  let latestProcessedIndex = 0;
  while (latestProcessedIndex < markdown.length) {
    const match = macroRegex.exec(markdown);
    if (match !== null) {
      // If there is something between the last processed index and the start of the match, create a text section for
      // it.
      const delta = match.index - latestProcessedIndex;
      if (delta > 0) {
        const textSection = generateTextSection(
          markdown.substring(latestProcessedIndex, match.index)
        );
        sections.push(textSection);
      }

      // Create a section object that corresponds to the found macro.
      const macroSection = generateMacroSection(
        kataPath,
        match,
        globalCodeSources
      );
      sections.push(macroSection);
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

function generateKataContent(path, globalCodeSources) {
  const kataId = basename(path);
  const markdownPath = join(path, contentFileNames.kataMarkdown);
  const markdown = tryReadFile(
    markdownPath,
    "Could not read the contents of the kata markdown file"
  );
  const title = getTitleFromMarkdown(markdown);
  const sections = generateSections(path, markdown, globalCodeSources);
  return {
    id: kataId,
    title: title,
    sections: sections,
  };
}

function generateKatasContent(katasPath, outputPath) {
  console.log("Generating katas content");
  const indexPath = join(katasPath, contentFileNames.katasIndex);
  const indexJson = tryReadFile(
    indexPath,
    "Could not read the contents of the katas index file"
  );
  const katasDirs = tryParseJSON(
    indexJson,
    `Invalid katas index at ${indexPath}`
  );
  const globalCodeSources = {
    basePath: katasPath,
    sources: {},
  };
  var katas = [];
  for (const kataDir of katasDirs) {
    const kataPath = join(katasPath, kataDir);
    const kata = generateKataContent(kataPath, globalCodeSources);
    katas.push(kata);
  }

  const codeDependencies = [];
  for (let name in globalCodeSources.sources) {
    codeDependencies.push({
      name: name,
      contents: globalCodeSources.sources[name],
    });
  }
  const katasContent = {
    katas: katas,
    codeDependencies: codeDependencies,
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
