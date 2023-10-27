// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/**
 * Katas Taxonomy
 *
 * A Kata is a top-level container of educational items which are used to explain a particular topic.
 *
 * This file builds the content for all the Katas. The katas ordering is conveyed by JSON file where each
 * string in the array represents a folder that contains all the data to build the kata.
 *
 * Each Kata is organized in a directory where an index.md file provides a description on how the kata must be composed.
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

function tryGetTitleFromMarkdown(markdown, errorPrefix) {
  const result = /^# (.*)/.exec(markdown);
  if (result?.length !== 2)
    throw new Error(`${errorPrefix}\nCould not get title from markdown`);
  return result[1];
}

function tryGetTitleFromSegment(segment, errorPrefix) {
  // The segment that represents the title can only be a markdown segment.
  if (segment.type !== "markdown") {
    throw new Error(
      `${errorPrefix}\n` +
        `segment is expected to be the title but found a segment of type '${segment.type}' instead`,
    );
  }

  // Check that the segment has just one line.
  const linesCount = segment.markdown.split(/\r?\n/).length;
  if (linesCount !== 1) {
    throw new Error(
      `${errorPrefix}\n` +
        `A title segment must be 1 line, but ${linesCount} lines are present\n` +
        `Hint: is the markdown missing a @[section] macro?`,
    );
  }
  const title = tryGetTitleFromMarkdown(segment.markdown, errorPrefix);

  return title;
}

function tryParseJSON(json, errorPrefix) {
  let parsed;
  try {
    parsed = JSON.parse(json);
  } catch (e) {
    throw new Error(`${errorPrefix}\n${e}`);
  }
  return parsed;
}

function tryReadFile(filePath, errorPrefix) {
  let content;
  try {
    content = readFileSync(filePath, "utf8");
  } catch (e) {
    throw new Error(`${errorPrefix}\n${e}`);
  }
  return content;
}

function identifyMissingProperties(properties, required) {
  return required.filter((property) => !Object.hasOwn(properties, property));
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

function resolveSvgSegment(properties, baseFolderPath) {
  const requiredProperties = ["path"];
  const missingProperties = identifyMissingProperties(
    properties,
    requiredProperties,
  );
  if (missingProperties.length > 0) {
    throw new Error(
      `SVG macro is missing the following properties: ${missingProperties}`,
    );
  }

  const svgPath = join(baseFolderPath, properties.path);
  const svg = tryReadFile(
    svgPath,
    `Could not read the contents of the SVG file at ${svgPath}`,
  );

  properties["svg"] = svg;
}

function resolveEmbeddedContent(segments, baseFolderPath) {
  for (const segment of segments) {
    if (segment.type === "svg") {
      resolveSvgSegment(segment.properties, baseFolderPath);
    }
  }
}

function appendToMarkdownSegment(markdownSegment, segmentToAppend) {
  if (segmentToAppend.type === "markdown") {
    markdownSegment.markdown += "\n" + segmentToAppend.markdown;
  } else if (segmentToAppend.type === "svg") {
    markdownSegment.markdown += "\n" + segmentToAppend.properties.svg;
  } else {
    throw new Error(
      `Cannot append segment of type "${segmentToAppend.type}" into markdown segment`,
    );
  }
}

function coalesceIntoSingleMarkdownSegment(startingSegment, segmentsStack) {
  const markdownSegment = { type: "markdown", markdown: "" };
  appendToMarkdownSegment(markdownSegment, startingSegment);
  const isCoalesceSupportedForSegment = (segment) =>
    segment.type === "markdown" || segment.type === "svg";
  while (
    segmentsStack.length > 0 &&
    isCoalesceSupportedForSegment(segmentsStack.at(-1))
  ) {
    const currentSegment = segmentsStack.pop();
    appendToMarkdownSegment(markdownSegment, currentSegment);
  }

  return markdownSegment;
}

function coalesceSegments(segments) {
  const coalescedSegments = [];
  const segmentsStack = segments.reverse();
  while (segmentsStack.length > 0) {
    let currentSegment = segmentsStack.pop();
    let coalescedSegment = null;
    if (currentSegment.type === "markdown" || currentSegment.type === "svg") {
      coalescedSegment = coalesceIntoSingleMarkdownSegment(
        currentSegment,
        segmentsStack,
      );
    } else {
      coalescedSegment = currentSegment;
    }

    coalescedSegments.push(coalescedSegment);
  }

  return coalescedSegments;
}

function preProcessSegments(segments, baseFolderPath) {
  resolveEmbeddedContent(segments, baseFolderPath);
  const coalescedSegments = coalesceSegments(segments);
  return coalescedSegments;
}

function parseMarkdown(markdown) {
  const segments = [];
  const macroRegex = /@\[(?<type>\w+)\]\((?<json>\{.*?\})\)\r?\n/gs;
  let latestProcessedIndex = 0;
  while (latestProcessedIndex < markdown.length) {
    const match = macroRegex.exec(markdown);
    if (match !== null) {
      // If there is something between the last processed index and the start of the match that is not just whitespace,
      // it represents a text segment.
      const delta = match.index - latestProcessedIndex;
      if (delta > 0) {
        const textSegment = tryCreateMarkdownSegment(
          markdown.substring(latestProcessedIndex, match.index),
        );
        if (textSegment !== null) {
          segments.push(textSegment);
        }
      }

      // Create a segment that corresponds to the found macro.
      const macroSegment = createMacroSegment(match);
      segments.push(macroSegment);
      latestProcessedIndex = macroRegex.lastIndex;
    } else {
      // No more matches were found, create a text segment with the remaining content.
      const textSegment = tryCreateMarkdownSegment(
        markdown.substring(latestProcessedIndex, markdown.length),
      );
      if (textSegment !== null) {
        segments.push(textSegment);
      }
      latestProcessedIndex = markdown.length;
    }
  }

  return segments;
}

function createExample(baseFolderPath, properties) {
  // Validate that the data contains the required properties.
  const requiredProperties = ["id", "codePath"];
  const missingProperties = identifyMissingProperties(
    properties,
    requiredProperties,
  );
  if (missingProperties.length > 0) {
    throw new Error(
      `Example macro is missing the following properties: ${missingProperties}`,
    );
  }

  // Generate the object using the macro properties.
  const codePath = join(baseFolderPath, properties.codePath);
  const code = tryReadFile(
    codePath,
    `Could not read the contents of the example code file at ${codePath}`,
  );
  return {
    type: "example",
    id: properties.id,
    code,
  };
}

function createTextContent(markdown) {
  const html = marked(markdown);
  return { type: "text-content", asHtml: html, asMarkdown: markdown };
}

function createSolution(baseFolderPath, properties) {
  // Validate that the data contains the required properties.
  const requiredProperties = ["id", "codePath"];
  const missingProperties = identifyMissingProperties(
    properties,
    requiredProperties,
  );
  if (missingProperties.length > 0) {
    throw new Error(
      `Solution macro is missing the following properties: ${missingProperties}`,
    );
  }

  // Generate the object using the macro properties.
  const codePath = join(baseFolderPath, properties.codePath);
  const code = tryReadFile(
    codePath,
    `Could not read the contents of the solution code file at ${codePath}`,
  );
  return {
    type: "solution",
    id: properties.id,
    code,
  };
}

function createExplainedSolution(markdownFilePath) {
  const markdown = tryReadFile(
    markdownFilePath,
    `Could not read solution markdown file at ${markdownFilePath}`,
  );

  const solutionFolderPath = dirname(markdownFilePath);
  const rawSegments = parseMarkdown(markdown);
  const segments = preProcessSegments(rawSegments, solutionFolderPath);
  const solutionItems = [];
  for (const segment of segments) {
    let solutionItem = null;
    if (segment.type === "example") {
      solutionItem = createExample(solutionFolderPath, segment.properties);
    } else if (segment.type === "solution") {
      solutionItem = createSolution(solutionFolderPath, segment.properties);
    } else if (segment.type === "markdown") {
      solutionItem = createTextContent(segment.markdown);
    }

    if (solutionItem !== null) {
      solutionItems.push(solutionItem);
    }
  }

  return {
    type: "explained-solution",
    items: solutionItems,
  };
}

function createAnswer(markdownFilePath) {
  const markdown = tryReadFile(
    markdownFilePath,
    `Could not read answer markdown file at ${markdownFilePath}`,
  );

  const answerFolderPath = dirname(markdownFilePath);
  const rawSegments = parseMarkdown(markdown);
  const segments = preProcessSegments(rawSegments, answerFolderPath);
  const items = [];
  for (const segment of segments) {
    let answerItem = null;
    if (segment.type === "example") {
      answerItem = createExample(answerFolderPath, segment.properties);
    } else if (segment.type === "markdown") {
      answerItem = createTextContent(segment.markdown);
    }

    if (answerItem !== null) {
      items.push(answerItem);
    }
  }

  return { type: "answer", items };
}

function createQuestion(kataPath, properties) {
  // Validate that the data contains the required properties.
  const requiredProperties = ["descriptionPath", "answerPath"];
  const missingProperties = identifyMissingProperties(
    properties,
    requiredProperties,
  );
  if (missingProperties.length > 0) {
    throw new Error(
      `Question macro is missing the following properties\n` +
        `${missingProperties}\n` +
        `Macro properties:\n` +
        `${JSON.stringify(properties, undefined, 2)}`,
    );
  }

  // Generate the object using the macro properties.
  const descriptionMarkdown = tryReadFile(
    join(kataPath, properties.descriptionPath),
    `Could not read descripton for question ${properties.id}`,
  );
  const description = createTextContent(descriptionMarkdown);
  const answer = createAnswer(join(kataPath, properties.answerPath));

  return {
    type: "question",
    description,
    answer,
  };
}

function createExerciseSection(kataPath, properties, globalCodeSources) {
  // Validate that the data contains the required properties.
  const requiredProperties = [
    "id",
    "title",
    "descriptionPath",
    "codePaths",
    "placeholderSourcePath",
    "solutionPath",
  ];
  const missingProperties = identifyMissingProperties(
    properties,
    requiredProperties,
  );
  if (missingProperties.length > 0) {
    throw new Error(
      `Exercise macro is missing the following properties\n` +
        `${missingProperties}\n` +
        `Macro properties:\n` +
        `${JSON.stringify(properties, undefined, 2)}`,
    );
  }

  // Generate the object using the macro properties.
  const descriptionMarkdown = tryReadFile(
    join(kataPath, properties.descriptionPath),
    `Could not read descripton for exercise ${properties.id}`,
  );
  const description = createTextContent(descriptionMarkdown);
  const resolvedCodePaths = properties.codePaths.map((path) =>
    join(kataPath, path),
  );
  const sourceIds = aggregateSources(resolvedCodePaths, globalCodeSources);
  const placeholderCode = tryReadFile(
    join(kataPath, properties.placeholderSourcePath),
    `Could not read placeholder code for exercise '${properties.id}'`,
  );
  const explainedSolution = createExplainedSolution(
    join(kataPath, properties.solutionPath),
  );

  return {
    type: "exercise",
    id: properties.id,
    title: properties.title,
    description,
    sourceIds,
    placeholderCode,
    explainedSolution,
  };
}

function createLessonSection(kataPath, properties, segmentsStack) {
  // Validate that the data contains the required properties.
  const requiredProperties = ["id", "title"];
  const missingProperties = identifyMissingProperties(
    properties,
    requiredProperties,
  );
  if (missingProperties.length > 0) {
    throw new Error(
      `Section macro is missing the following properties\n` +
        `${missingProperties}\n` +
        `Macro properties:\n` +
        `${JSON.stringify(properties, undefined, 2)}`,
    );
  }

  // Continue processing segments until another section-delimiting segment appears.
  const lessonItems = [];
  const isSectionDelimiterSegment = (segment) =>
    segment.type === "exercise" || segment.type === "section";
  while (
    segmentsStack.length > 0 &&
    !isSectionDelimiterSegment(segmentsStack.at(-1))
  ) {
    const currentSegment = segmentsStack.pop();
    let lessonItem = null;
    if (currentSegment.type === "example") {
      lessonItem = createExample(kataPath, currentSegment.properties);
    } else if (currentSegment.type === "markdown") {
      lessonItem = createTextContent(currentSegment.markdown);
    } else if (currentSegment.type === "question") {
      lessonItem = createQuestion(kataPath, currentSegment.properties);
    }

    // Check that a valid lesson item was created.
    if (lessonItem === null) {
      throw new Error(
        `Lesson item could not be generated for segment of type '${currentSegment.type}'\n` +
          `segment:\n` +
          `${JSON.stringify(currentSegment, undefined, 2)}`,
      );
    }

    lessonItems.push(lessonItem);
  }

  return {
    type: "lesson",
    id: properties.id,
    title: properties.title,
    items: lessonItems,
  };
}

function createMacroSegment(match) {
  const type = match.groups.type;
  const propertiesJson = match.groups.json;
  const properties = tryParseJSON(
    propertiesJson,
    `Invalid JSON for macro of type ${type}.\n` + `JSON: ${propertiesJson}`,
  );
  return {
    type,
    properties,
  };
}

function tryCreateMarkdownSegment(text) {
  const trimmedText = text.trim();
  if (trimmedText.length > 0) {
    return { type: "markdown", markdown: trimmedText };
  }

  return null;
}

function createKata(kataPath, id, title, segments, globalCodeSources) {
  // Validate that the kata has at least one segment.
  if (segments.length === 0) {
    throw new Error(`Kata '${id}' does not have any segments`);
  }

  // Create sections from the segments in the stack.
  // Use the array of segments as a stack to keep track of the segments that have not been processed.
  const segmentsStack = segments.reverse();
  const sections = [];
  while (segmentsStack.length > 0) {
    const currentSegment = segmentsStack.pop();
    let section = null;
    if (currentSegment.type === "exercise") {
      section = createExerciseSection(
        kataPath,
        currentSegment.properties,
        globalCodeSources,
      );
    } else if (currentSegment.type === "section") {
      section = createLessonSection(
        kataPath,
        currentSegment.properties,
        segmentsStack,
      );
    }

    // Check if a valid section was created.
    if (section === null) {
      throw new Error(
        `Unexpexted segment of type '${currentSegment.type}'\n` +
          `segment:\n` +
          `${JSON.stringify(currentSegment, undefined, 2)}\n` +
          `Hint: is the markdown missing a @[section] macro?`,
      );
    }

    sections.push(section);
  }

  return {
    id,
    title,
    sections,
  };
}

function generateKataContent(path, globalCodeSources) {
  console.log(`- Creating content for kata at: ${path}`);
  const markdownPath = join(path, contentFileNames.kataMarkdown);
  const markdown = tryReadFile(
    markdownPath,
    "Could not read the contents of the kata markdown file",
  );

  const kataId = basename(path);
  const rawSegments = parseMarkdown(markdown);

  // The first segment in the kata must be the title.
  const firstSegment = rawSegments.at(0);
  const title = tryGetTitleFromSegment(
    firstSegment,
    `Could not get title for kata '${kataId}'`,
  );

  // Do not use the first segment since it was already processed to get the kata's title.
  const segments = preProcessSegments(rawSegments.slice(1), path);
  const kata = createKata(path, kataId, title, segments, globalCodeSources);
  console.log(`-- '${kata.id}' kata was successfully created`);
  return kata;
}

function validateIdsUniqueness(katas) {
  console.log("Validating IDs uniqueness across all katas");
  const allIds = new Set();
  const assertUniqueness = (id) => {
    const idAlreadyExists = allIds.has(id);
    if (idAlreadyExists) {
      throw new Error(`"${id}" is not unique`);
    }
    allIds.add(id);
  };

  for (const kata of katas) {
    // Check kata IDs are unique.
    assertUniqueness(kata.id);
    for (const section of kata.sections) {
      // Check section IDs are unique.
      assertUniqueness(section.id);
      if (section.type === "exercise") {
        // Check IDs for examples and solutions within exercises are unique.
        section.explainedSolution.items.forEach((item) => {
          if (item.type === "example" || item.type === "solution") {
            assertUniqueness(item.id);
          }
        });
      } else if (section.type === "lesson") {
        // Check IDs for examples within lessons are unique.
        section.items.forEach((item) => {
          if (item.type === "example") {
            assertUniqueness(item.id);
          }
        });
      }
    }
  }
}

function generateKatasContent(katasPath, outputPath) {
  console.log("Generating katas content");
  const indexPath = join(katasPath, contentFileNames.katasIndex);
  const indexJson = tryReadFile(
    indexPath,
    "Could not read the contents of the katas index file",
  );
  const katasDirs = tryParseJSON(
    indexJson,
    `Invalid katas index at ${indexPath}`,
  );

  // Initialize an object where all the global code sources will be aggregated.
  const globalCodeSourcesContainer = {
    basePath: katasPath,
    sources: {},
  };

  // Generate an object for each kata and update the global code sources with the code they reference.
  var katas = [];
  for (const kataDir of katasDirs) {
    const kataPath = join(katasPath, kataDir);
    const kata = generateKataContent(kataPath, globalCodeSourcesContainer);
    katas.push(kata);
  }

  // Create the objects that will be written to a file.
  const globalCodeSources = [];
  for (let id in globalCodeSourcesContainer.sources) {
    globalCodeSources.push({
      id: id,
      code: globalCodeSourcesContainer.sources[id],
    });
  }

  // Validate the uniqueness of IDs.
  validateIdsUniqueness(katas);

  // Save the JS object to a file.
  const katasContent = {
    katas: katas,
    globalCodeSources: globalCodeSources,
  };

  if (!existsSync(outputPath)) {
    mkdirSync(outputPath);
  }

  const contentJsPath = join(outputPath, "katas-content.generated.ts");
  writeFileSync(
    contentJsPath,
    `export default ${JSON.stringify(katasContent, undefined, 2)}`,
    "utf-8",
  );
}

generateKatasContent(katasContentPath, katasGeneratedContentPath);
