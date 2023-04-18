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
    const katas = await getAllKatas();
    const filteredKatas = katas.filter(k => k.id == id);
    if (filteredKatas.length != 1) {
        throw new Error(`Failed to get kata with id: ${id}`);
    }

    return filteredKatas.at(0)!;
}

export async function runExercise(exercise: Exercise, implementation: string, eventCb: (msg: string) => void): Promise<boolean> {
    return run_kata_exercise(
        exercise.verificationImplementation,
        implementation,
        eventCb);
}