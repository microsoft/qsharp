// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//@ts-check

import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { basename, dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { inspect } from "node:util";

import { marked } from "marked"

import { katas } from '../katas/content/katas.js';

const thisDir = dirname(fileURLToPath(import.meta.url));
const katasContentDir = join(thisDir, "..", "katas", "content")
const katasGeneratedContentDir = join(thisDir, "src");

function getTitleFromMarkdown(markdown) {
    const titleRe = /#+ /;
    const lines = markdown.split(/\r?\n/);
    if (lines.length === 0) {
        throw new Error("Could not parse title, markdown is empty");
    }

    const firstLine = lines[0];
    const match = firstLine.match(titleRe);
    if (match === null) {
        throw new Error(`First line does not follow the expected title pattern: ${firstLine}`);
    }

    return firstLine.replace(titleRe, "");
}

function buildExerciseContent(id, exerciseDir) {
    const placeholderSource = readFileSync(join(exerciseDir, "placeholder.qs"), 'utf8');
    const referenceSource = readFileSync(join(exerciseDir, "reference.qs"), 'utf8');
    const verificationSource = readFileSync(join(exerciseDir, "verify.qs"), 'utf8');
    const contentAsMarkdown = readFileSync(join(exerciseDir, "content.md"), 'utf8');
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
        contentAsHtml: contentAsHtml
    };
}

function buildItemContent(item, kataDir) {
    const itemDir = join(kataDir, item.directory);
    const itemId = `${basename(kataDir)}__${item.directory}`;
    if (item.type === "exercise") {
        return buildExerciseContent(itemId, itemDir);
    }

    throw new Error(`Unknown module type ${item.type}`);
}

function buildKataContent(kata, katasDir) {
    const kataDir = join(katasDir, kata.directory)
    let itemsContent = [];
    for (const item of kata.items) {
        const moduleContent = buildItemContent(item, kataDir);
        itemsContent.push(moduleContent);
    }

    const contentAsMarkdown = readFileSync(join(kataDir, "content.md"), 'utf8');
    const contentAsHtml = marked.parse(contentAsMarkdown);
    const title = getTitleFromMarkdown(contentAsMarkdown);
    return {
        id: kata.directory,
        title: title,
        contentAsMarkdown: contentAsMarkdown,
        contentAsHtml: contentAsHtml,
        items: itemsContent
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
    writeFileSync(contentJsPath, 'export const katas = ' + inspect(katasContent, { depth: null }), 'utf-8');
}

buildKatasContentJs(katasContentDir, katasGeneratedContentDir);
