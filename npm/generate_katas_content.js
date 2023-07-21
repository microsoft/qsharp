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

import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { basename, dirname, join, relative, sep } from "node:path";
import { fileURLToPath } from "node:url";

import { marked } from "marked";

const scriptDirPath = dirname(fileURLToPath(import.meta.url));
const katasContentPath = join(scriptDirPath, "..", "katas", "content");
const katasGeneratedContentPath = join(scriptDirPath, "src");
const contentFileNames = {
  katasIndex: "index.json",
  kataMarkdown: "index.md",
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

function getSourceId(sourcePath, basePath) {
  return relative(basePath, sourcePath).replaceAll(sep, "__");
}

function aggregateSources(paths, globalCodeSources) {
  const codeSources = [];
  for (const path of paths) {
    const id = getSourceId(path, globalCodeSources.basePath);
    if (!(id in globalCodeSources.sources)) {
      const code = tryReadFile(path, "Could not read code dependency");
      globalCodeSources.sources[id] = code;
    }
    codeSources.push(id);
  }
  return codeSources;
}

function generateExplainedSolution(path) {
  // TODO: Temporary scaffolding.
  const solutionAsMarkdown = tryReadFile(
    path,
    `Could not read solution markdown file`
  );
  const text = {
    type: "text",
    contentAsMarkdown: solutionAsMarkdown,
    contentAsHtml: marked(solutionAsMarkdown),
  };
  return {
    type: "explained-solution",
    items: [text],
  };
}

function generateExerciseSection(kataPath, properties, globalCodeSources) {
  // Validate that the data contains the required properties.
  const requiredProperties = [
    "id",
    "codePaths",
    "placeholderSourcePath",
    "solutionSourcePath",
    "solutionPath",
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
  const resolvedCodePaths = properties.codePaths.map((path) =>
    join(kataPath, path)
  );
  const sourceIds = aggregateSources(resolvedCodePaths, globalCodeSources);
  const placeholderCode = tryReadFile(
    join(kataPath, properties.placeholderSourcePath),
    `Could not read placeholder code for exercise ${properties.id}`
  );
  const explainedSolution = generateExplainedSolution(
    join(kataPath, properties.solutionPath)
  );

  return {
    type: "exercise",
    id: properties.id,
    sourceIds: sourceIds,
    placeholderCode: placeholderCode,
    explainedSolution: explainedSolution,
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
  const macroRegex = /@\[(?<type>\w+)\]\((?<json>\{.*?\})\)\n/gs;
  let latestProcessedIndex = 0;
  while (latestProcessedIndex < markdown.length) {
    const match = macroRegex.exec(markdown);
    if (match !== null) {
      // If there is something between the last processed index and the start of the match, create a text section for
      // it.
      // TODO: Should error when there is text not associated to a macro.
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
  console.log(`- ${kataId}`);
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
  const globalCodeSourcesContainer = {
    basePath: katasPath,
    sources: {},
  };
  var katas = [];
  for (const kataDir of katasDirs) {
    const kataPath = join(katasPath, kataDir);
    const kata = generateKataContent(kataPath, globalCodeSourcesContainer);
    katas.push(kata);
  }

  const globalCodeSources = [];
  for (let id in globalCodeSourcesContainer.sources) {
    globalCodeSources.push({
      id: id,
      code: globalCodeSourcesContainer.sources[id],
    });
  }
  const katasContent = {
    katas: katas,
    globalCodeSources: globalCodeSources,
  };

  // Save the JS object to a file.
  if (!existsSync(outputPath)) {
    mkdirSync(outputPath);
  }

  const contentJsPath = join(outputPath, "katas-content.generated.ts");
  writeFileSync(
    contentJsPath,
    `export default ${JSON.stringify(katasContent, undefined, 2)}`,
    "utf-8"
  );
}

generateKatasContent(katasContentPath, katasGeneratedContentPath);
