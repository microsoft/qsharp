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

function tryGetTitleFromMarkdown(markdown, errorPrefix) {
  const titleRe = /#+ /;
  const lines = markdown.trim().split(/\r?\n/);
  if (lines.length === 0) {
    throw new Error(`${errorPrefix}\nCould not get title, markdown is empty`);
  }

  const firstLine = lines[0];
  const match = firstLine.match(titleRe);
  if (match === null) {
    throw new Error(
      `${errorPrefix}\nFirst line does not follow the expected title pattern: "${firstLine}"`
    );
  }

  return firstLine.replace(titleRe, "");
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

function parseMarkdown(markdown) {
  const tokens = [];
  const macroRegex = /@\[(?<type>\w+)\]\((?<json>\{.*?\})\)\n/gs;
  let latestProcessedIndex = 0;
  while (latestProcessedIndex < markdown.length) {
    const match = macroRegex.exec(markdown);
    if (match !== null) {
      // If there is something between the last processed index and the start of the match that is not just whitespace,
      // it represents a text token.
      const delta = match.index - latestProcessedIndex;
      if (delta > 0) {
        const textToken = tryCreateMarkdownToken(
          markdown.substring(latestProcessedIndex, match.index)
        );
        if (textToken !== null) {
          tokens.push(textToken);
        }
      }

      // Create a token that corresponds to the found macro.
      const macroToken = createMacroToken(match);
      tokens.push(macroToken);
      latestProcessedIndex = macroRegex.lastIndex;
    } else {
      // No more matches were found, create a text token with the remaining content.
      const textToken = tryCreateMarkdownToken(
        markdown.substring(latestProcessedIndex, markdown.length)
      );
      if (textToken !== null) {
        tokens.push(textToken);
      }
      latestProcessedIndex = markdown.length;
    }
  }

  return tokens;
}

function createExample(baseFolderPath, properties) {
  // Validate that the data contains the required properties.
  const requiredProperties = ["id", "codePath"];
  const missingProperties = identifyMissingProperties(
    properties,
    requiredProperties
  );
  if (missingProperties.length > 0) {
    throw new Error(
      `Example macro is missing the following properties: ${missingProperties}`
    );
  }

  // Generate the object using the macro properties.
  const codePath = join(baseFolderPath, properties.codePath);
  const code = tryReadFile(
    codePath,
    `Could not read the contents of the example code file at ${codePath}`
  );
  return {
    type: "example",
    id: properties.id,
    code: code,
  };
}

function createSolution(baseFolderPath, properties) {
  // Validate that the data contains the required properties.
  const requiredProperties = ["id", "codePath"];
  const missingProperties = identifyMissingProperties(
    properties,
    requiredProperties
  );
  if (missingProperties.length > 0) {
    throw new Error(
      `Solution macro is missing the following properties: ${missingProperties}`
    );
  }

  // Generate the object using the macro properties.
  const codePath = join(baseFolderPath, properties.codePath);
  const code = tryReadFile(
    codePath,
    `Could not read the contents of the solution code file at ${codePath}`
  );
  return {
    type: "solution",
    id: properties.id,
    code: code,
  };
}

function createTextContent(markdown) {
  const html = marked(markdown);
  return { type: "text-content", asHtml: html, asMarkdown: markdown };
}

function createExplainedSolution(markdownFilePath) {
  const markdown = tryReadFile(
    markdownFilePath,
    `Could not read solution markdown file at ${markdownFilePath}`
  );

  const solutionFolderPath = dirname(markdownFilePath);
  const tokens = parseMarkdown(markdown);
  const solutionItems = [];
  for (const token of tokens) {
    let solutionItem = null;
    if (token.type === "example") {
      solutionItem = createExample(solutionFolderPath, token.properties);
    } else if (token.type === "solution") {
      solutionItem = createSolution(solutionFolderPath, token.properties);
    } else if (token.type === "markdown") {
      solutionItem = createTextContent(token.markdown);
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

function createExerciseSection(kataPath, properties, globalCodeSources) {
  // Validate that the data contains the required properties.
  const requiredProperties = [
    "id",
    "descriptionPath",
    "codePaths",
    "placeholderSourcePath",
    "solutionPath",
  ];
  const missingProperties = identifyMissingProperties(
    properties,
    requiredProperties
  );
  if (missingProperties.length > 0) {
    throw new Error(
      `Exercise macro is missing the following properties\n` +
        `${missingProperties}\n` +
        `Macro properties:\n` +
        `${JSON.stringify(properties, undefined, 2)}`
    );
  }

  // Generate the object using the macro properties.
  const descriptionMarkdown = tryReadFile(
    join(kataPath, properties.descriptionPath),
    `Could not read descripton for exercise ${properties.id}`
  );
  const description = createTextContent(descriptionMarkdown);
  const title = tryGetTitleFromMarkdown(
    descriptionMarkdown,
    `Could not get title for exercise '${properties.id}'`
  );
  const resolvedCodePaths = properties.codePaths.map((path) =>
    join(kataPath, path)
  );
  const sourceIds = aggregateSources(resolvedCodePaths, globalCodeSources);
  const placeholderCode = tryReadFile(
    join(kataPath, properties.placeholderSourcePath),
    `Could not read placeholder code for exercise '${properties.id}'`
  );
  const explainedSolution = createExplainedSolution(
    join(kataPath, properties.solutionPath)
  );

  return {
    type: "exercise",
    id: properties.id,
    title: title,
    description: description,
    sourceIds: sourceIds,
    placeholderCode: placeholderCode,
    explainedSolution: explainedSolution,
  };
}

function createLessonSection(kataPath, properties, tokensStack) {
  // Validate that the data contains the required properties.
  const requiredProperties = ["id", "title"];
  const missingProperties = identifyMissingProperties(
    properties,
    requiredProperties
  );
  if (missingProperties.length > 0) {
    throw new Error(
      `Section macro is missing the following properties\n` +
        `${missingProperties}\n` +
        `Macro properties:\n` +
        `${JSON.stringify(properties, undefined, 2)}`
    );
  }

  // Continue processing tokens until another section-delimiting token appears.
  const lessonItems = [];
  const isSectionDelimiterToken = (token) =>
    token.type === "exercise" ||
    token.type === "question" ||
    token.type === "section";
  while (
    tokensStack.length > 0 &&
    !isSectionDelimiterToken(tokensStack.at(-1))
  ) {
    const currentToken = tokensStack.pop();
    let lessonItem = null;
    if (currentToken.type === "example") {
      lessonItem = createExample(kataPath, currentToken.properties);
    } else if (currentToken.type === "markdown") {
      lessonItem = createTextContent(currentToken.markdown);
    }

    // Check that a valid lesson item was created.
    if (lessonItem === null) {
      throw new Error(
        `Lesson item could not be generated for token of type '${currentToken.type}'\n` +
          `Token:\n` +
          `${JSON.stringify(currentToken, undefined, 2)}`
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

function createQuestionSection(kataPath, properties) {
  // Validate that the data contains the required properties.
  const requiredProperties = ["id", "descriptionPath", "answerPath"];
  const missingProperties = identifyMissingProperties(
    properties,
    requiredProperties
  );
  if (missingProperties.length > 0) {
    throw new Error(
      `Question macro is missing the following properties\n` +
        `${missingProperties}\n` +
        `Macro properties:\n` +
        `${JSON.stringify(properties, undefined, 2)}`
    );
  }

  // Generate the object using the macro properties.
  const descriptionMarkdown = tryReadFile(
    join(kataPath, properties.descriptionPath),
    `Could not read descripton for question ${properties.id}`
  );
  const description = createTextContent(descriptionMarkdown);
  const title = tryGetTitleFromMarkdown(
    descriptionMarkdown,
    `Could not get title for question '${properties.id}'`
  );
  const answerMarkdown = tryReadFile(
    join(kataPath, properties.descriptionPath),
    `Could not read answer for question ${properties.id}`
  );
  const answer = createTextContent(answerMarkdown);

  return {
    type: "question",
    id: properties.id,
    title: title,
    description: description,
    answer: answer,
  };
}

function createMacroToken(match) {
  const type = match.groups.type;
  const propertiesJson = match.groups.json;
  const properties = tryParseJSON(
    propertiesJson,
    `Invalid JSON for macro of type ${type}.\n` + `JSON: ${propertiesJson}`
  );
  return {
    type: type,
    properties: properties,
  };
}

function tryCreateMarkdownToken(text) {
  const trimmedText = text.trim();
  if (trimmedText.length > 0) {
    return { type: "markdown", markdown: trimmedText };
  }

  return null;
}

function tryGetTitleFromToken(token, errorPrefix) {
  // The token that represents the title can only be a markdown token.
  if (token.type !== "markdown") {
    throw new Error(
      `${errorPrefix}\n` +
        `Token is expected to be the title but found a token of type '${token.type}' instead`
    );
  }

  // Check that the token has just one line.
  const linesCount = token.markdown.split(/\r?\n/).length;
  if (linesCount !== 1) {
    throw new Error(
      `${errorPrefix}\n` +
        `A title token must be 1 line, but ${linesCount} lines are present\n` +
        `Hint: is the markdown missing a @[section] macro?`
    );
  }
  const title = tryGetTitleFromMarkdown(token.markdown, errorPrefix);

  return title;
}

function createKata(tokens, kataPath, globalCodeSources) {
  const kataId = basename(kataPath);

  // Validate that the kata markdown file is not empty.
  if (tokens.length === 0) {
    throw new Error(`Markdown for '${kataId}' kata does not have any tokens`);
  }

  // Use the array of tokens as a stack to keep track of the tokens that have not been processed.
  const tokensStack = tokens.reverse();

  // The first token in the kata must be the title.
  const firstToken = tokensStack.pop();
  const title = tryGetTitleFromToken(
    firstToken,
    `Could not get title for kata '${kataId}'`
  );

  // Create sections from the remainin tokens in the stack.
  const sections = [];
  while (tokensStack.length > 0) {
    const currentToken = tokensStack.pop();
    let section = null;
    if (currentToken.type === "exercise") {
      section = createExerciseSection(
        kataPath,
        currentToken.properties,
        globalCodeSources
      );
    } else if (currentToken.type === "question") {
      section = createQuestionSection(kataPath, currentToken.properties);
    } else if (currentToken.type === "section") {
      section = createLessonSection(
        kataPath,
        currentToken.properties,
        tokensStack
      );
    }

    // Check if a valid section was created.
    if (section === null) {
      throw new Error(
        `Unexpexted token of type '${currentToken.type}'\n` +
          `Token:\n` +
          `${JSON.stringify(currentToken, undefined, 2)}\n` +
          `Hint: is the markdown missing a @[section] macro?`
      );
    }

    sections.push(section);
  }

  return {
    id: kataId,
    title: title,
    tokens: tokens,
    sections: sections,
  };
}

function generateKataContent(path, globalCodeSources) {
  console.log(`- Creating content for kata at: ${path}`);
  const markdownPath = join(path, contentFileNames.kataMarkdown);
  const markdown = tryReadFile(
    markdownPath,
    "Could not read the contents of the kata markdown file"
  );
  const tokens = parseMarkdown(markdown);
  const kata = createKata(tokens, path, globalCodeSources);
  console.log(`-- '${kata.id}' kata was successfully created`);
  return kata;
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
