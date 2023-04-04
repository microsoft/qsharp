// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

export type KataExercise = {
    id: string;
    title: string;
    description: string;
    verificationImplementation: string;
    referenceImplementatuon: string;
}

export type KataModule = {
    id: string;
    title: string;
    description: string;
    exercises: KataExercise[]
}

let modules : KataModule[] = [
    {
        id: "single-qubit-gates",
        title: "Single-Qubit Gates",
        description: "Description for single qubit gates kata.",
        exercises: []
    },
    {
        id: "multi-qubit-gates",
        title: "Multi-Qubit Gates",
        description: "Description for multi qubit gates kata.",
        exercises: []
    }
];

export function queryKataModules() : KataModule[] {
    return modules;
}

export function getKataModule(id: string) : KataModule {
    let filteredModules = modules.filter(m => m.id == id);
    if (filteredModules.length != 1)
    {
        throw new Error(`Failed to get module with id: ${id}`);
    }

    return filteredModules.at(0)!;
}