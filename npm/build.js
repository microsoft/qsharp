// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import {readFileSync, writeFileSync}  from "node:fs";
import {dirname, join} from "node:path";
import {fileURLToPath} from "node:url";
import {inspect} from "node:util";

import {katasMetadata} from "../katas/content/dist/metadata.js"

const thisDir = dirname(fileURLToPath(import.meta.url));
const katasContentDir = join(thisDir, "..", "katas/content")
const katasContentJs = join(thisDir, "dist", "katas-content.js");

function buildExercise(exerciseMetadata, moduleDir) {
    const exerciseDir = join(moduleDir, exerciseMetadata.directory);
    const placeholderSource = readFileSync(join(exerciseDir, "placeholder.qs"), 'utf8');
    const referenceSource = readFileSync(join(exerciseDir, "reference.qs"), 'utf8');
    const verificationSource = readFileSync(join(exerciseDir, "verify.qs"), 'utf8');
    return {
        id: exerciseMetadata.id,
        title: exerciseMetadata.title,
        placeholderImplementation: placeholderSource,
        referenceImplementation: referenceSource,
        verificationImplementation: verificationSource
    };
}

function buildKata(kataMetadata, katasDir) {
    const kataDir = join(katasDir, kataMetadata.directory)
    let exercises = [];
    for(const exerciseMetadata of kataMetadata.exercises) {
        const exercise = buildExercise(exerciseMetadata, kataDir);
        exercises.push(exercise);
    }

    return {
        id: kataMetadata.id,
        title: kataMetadata.title,
        exercises: exercises
    };
}

function buildKatasContentJs(katasDir, outJsPath)
{
    console.log("Building katas content");
    var katas = [];
    for(const kataMetadata of katasMetadata.modules) {
        var kata = buildKata(kataMetadata, katasDir);
        katas.push(kata);
    }

    writeFileSync(outJsPath, 'const katas = ' + inspect(katas, {depth: null }) , 'utf-8');
}

buildKatasContentJs(katasContentDir, katasContentJs);
