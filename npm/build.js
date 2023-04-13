// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import {readFileSync, writeFileSync}  from "node:fs";
import {dirname, join} from "node:path";
import {fileURLToPath} from "node:url";
import {inspect} from "node:util";

import {parse} from "marked"

import {katasMetadata} from "../katas/content/dist/metadata.js"

const thisDir = dirname(fileURLToPath(import.meta.url));
const katasContentDir = join(thisDir, "..", "katas", "content")
const katasContentJs = join(thisDir, "dist", "katas-content.js");
const katasContentDeclarationTs = join(thisDir, "dist", "katas-content.d.ts");

function buildExercise(exerciseMetadata, moduleDir) {
    const exerciseDir = join(moduleDir, exerciseMetadata.directory);
    const placeholderSource = readFileSync(join(exerciseDir, "placeholder.qs"), 'utf8');
    const referenceSource = readFileSync(join(exerciseDir, "reference.qs"), 'utf8');
    const verificationSource = readFileSync(join(exerciseDir, "verify.qs"), 'utf8');
    const contentAsMarkdown = readFileSync(join(exerciseDir, "content.md"), 'utf8');
    const contentAsHtml = parse(contentAsMarkdown);
    return {
        id: exerciseMetadata.id,
        title: exerciseMetadata.title,
        placeholderImplementation: placeholderSource,
        referenceImplementation: referenceSource,
        verificationImplementation: verificationSource,
        contentAsMarkdown: contentAsMarkdown,
        contentAsHtml: contentAsHtml
    };
}

function buildKata(kataMetadata, katasDir) {
    const kataDir = join(katasDir, kataMetadata.directory)
    let exercises = [];
    for(const exerciseMetadata of kataMetadata.exercises) {
        const exercise = buildExercise(exerciseMetadata, kataDir);
        exercises.push(exercise);
    }

    const contentAsMarkdown = readFileSync(join(kataDir, "content.md"), 'utf8');
    const contentAsHtml = parse(contentAsMarkdown);
    return {
        id: kataMetadata.id,
        title: kataMetadata.title,
        contentAsMarkdown: contentAsMarkdown,
        contentAsHtml: contentAsHtml,
        exercises: exercises
    };
}

function buildKatasContentJs(katasDir, outJsPath, declarationTsPath) {
    console.log("Building katas content");
    var katas = [];
    for(const kataMetadata of katasMetadata.modules) {
        var kata = buildKata(kataMetadata, katasDir);
        katas.push(kata);
    }

    writeFileSync(outJsPath, 'export const katas = ' + inspect(katas, {depth: null }), 'utf-8');
    const tsDeclaration = `declare let katas: any; export {katas}`;

    writeFileSync(declarationTsPath, tsDeclaration, 'utf-8');
}

buildKatasContentJs(katasContentDir, katasContentJs, katasContentDeclarationTs);
