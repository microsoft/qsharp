// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { run_kata_exercise } from "../lib/web/qsc_wasm.js";
import { katas } from "./katas-content.generated.js";

export type Exercise = {
    type: "exercise";
    id: string;
    title: string;
    contentAsHtml: string;
    contentAsMarkdown: string;
    verificationImplementation: string;
    referenceImplementation: string;
    placeholderImplementation: string;
}

export type KataModule = Exercise;

export type Kata = {
    id: string;
    title: string;
    contentAsHtml: string;
    contentAsMarkdown: string;
    modules: KataModule[]
}

export async function getAllKatas(): Promise<Kata[]> {
    return katas as Kata[];
}

export async function getKata(id: string): Promise<Kata> {
    let katas = await getAllKatas();
    let filteredKatas = katas.filter(k => k.id == id);
    if (filteredKatas.length != 1) {
        throw new Error(`Failed to get kata with id: ${id}`);
    }

    return filteredKatas.at(0)!;
}

export async function getExercise(id: string): Promise<Exercise> {
    let katas = await getAllKatas();
    for (let kata of katas) {
        let filteredExercises = kata.modules.filter(m => m.type === "exercise" && m.id === id);
        if (filteredExercises.length == 1) {
            return filteredExercises.at(0)!;
        }
    }

    throw new Error(`Failed to get exercise with id: ${id}`);
}

export async function runExercise(id: string, implementation: string, eventCb: (msg: string) => void): Promise<boolean> {
    let exercise = await getExercise(id);
    return run_kata_exercise(
        exercise.verificationImplementation,
        implementation,
        eventCb);
}