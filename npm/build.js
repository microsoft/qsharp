// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import {readFileSync, writeFileSync}  from "node:fs";
import {dirname, join} from "node:path";
import {fileURLToPath} from "node:url";
import {inspect} from "node:util";

import {katasMetadata} from "../katas/content/dist/metadata.js"

console.log("CESARZC: BUILD");

const thisDir = dirname(fileURLToPath(import.meta.url));
const katasContentDir = join(thisDir, "..", "katas/content")
const katasMetadataJs = join(thisDir, "dist", "katas-metadata.js");

//
var modules = [];
for(const moduleMetadata of katasMetadata.modules)
{
    console.log(`KATAS MODULE: ${moduleMetadata.title}`);
    const moduleDir = join(katasContentDir, moduleMetadata.directory)
    var exercises = [];
    for(const exerciseMetadata of moduleMetadata.exercises)
    {
        var exerciseDir = join(moduleDir, exerciseMetadata.directory);
        var placeholderPath = join(exerciseDir, "placeholder.qs");
        var placeholderSource = readFileSync(placeholderPath, 'utf8');
        var referencePath = join(exerciseDir, "reference.qs");
        var referenceSource = readFileSync(referencePath, 'utf8');
        var verificationPath = join(exerciseDir, "verify.qs");
        var verificationSource = readFileSync(verificationPath, 'utf8');
        var exercise = {
            id: exerciseMetadata.id,
            title: exerciseMetadata.title,
            placeholderImplementation: placeholderSource,
            referenceImplementation: referenceSource,
            verificationImplementation: verificationSource
        };
        exercises.push(exercise);
    }

    var module = {
        id: moduleMetadata.id,
        title: moduleMetadata.title,
        exercises: exercises
    };
    modules.push(module);
}

console.log(katasMetadataJs);
writeFileSync(katasMetadataJs, 'var modules = ' + inspect(modules, {depth: null }) , 'utf-8');