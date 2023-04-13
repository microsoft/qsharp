// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { run_kata_exercise } from "../lib/web/qsc_wasm.js";
import {katas} from "../dist/katas-content.js";

export type Exercise = {
    id: string;
    title: string;
    verificationImplementation: string;
    referenceImplementation: string;
    placeholderImplementation: string;
}

export type Kata = {
    id: string;
    title: string;
    contentAsHtml: string;
    contentAsMarkdown: string;
    exercises: Exercise[]
}

export function getAllKatas() : Kata[] {
    return katas as Kata[];
}

export function getKata(id: string) : Kata {
    let filteredKatas = getAllKatas().filter(k => k.id == id);
    if (filteredKatas.length != 1) {
        throw new Error(`Failed to get kata with id: ${id}`);
    }

    return filteredKatas.at(0)!;
}

export function getExercise(id: string) : Exercise {
    for (let kata of getAllKatas()) {
        let filteredExercises = kata.exercises.filter(e => e.id == id);
        if (filteredExercises.length == 1) {
            return filteredExercises.at(0)!;
        }
    }

    throw new Error(`Failed to get exercise with id: ${id}`);
}

export function runExercise(id: string, implementation: string, eventCb: (msg: string) => void) : boolean
{
    let exercise = getExercise(id);
    return run_kata_exercise(
        exercise.verificationImplementation,
        implementation,
        eventCb);
}