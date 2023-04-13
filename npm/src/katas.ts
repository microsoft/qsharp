// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { run_kata_implementation } from "../lib/web/qsc_wasm.js";
import {katas} from "../dist/katas-content.js";

export type KataExercise = {
    id: string;
    title: string;
    description: string;
    verificationImplementation: string;
    referenceImplementatuon: string;
    placeholderImplementation: string;
}

export type KataModule = {
    id: string;
    title: string;
    description: string;
    exercises: KataExercise[]
}

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
    exercises: Exercise[]
}

let modules : KataModule[] = [
    {
        id: "single-qubit-gates",
        title: "Single-Qubit Gates",
        description: "Description for single qubit gates kata.",
        exercises: [
            {
                id: "single-qubit-gates_y-gate",
                title: "Pauli Y Gate",
                description: "Description for Pauli Y gate",
                verificationImplementation: `namespace Kata {
                    open Microsoft.Quantum.Diagnostics;
                    open Microsoft.Quantum.Intrinsic;
                
                    operation ApplyYReference(q : Qubit) : Unit is Adj + Ctl {
                        body ... {
                            Y(q);
                        }
                        adjoint self;
                    }
                
                    operation Verify() : Bool {
                        let task = ApplyY;
                        let taskRef = ApplyYReference;
                
                        use (aux, target) = (Qubit(), Qubit());
                        H(aux);
                        CNOT(aux, target);
                
                        task(target);
                        Adjoint taskRef(target);
                
                        CNOT(aux, target);
                        H(aux);
                
                        if CheckZero(target) {
                            if CheckZero(aux) {
                                Message("CORRECT");
                                task(target);
                                Message("Expected State:");
                                DumpMachine();
                                return true;
                            }
                        }
                
                        Reset(aux);
                        Reset(target);
                
                        // Use DumpMachine to display actual vs desired state.
                        Message("INCORRECT");
                        task(target);
                        Message("Actual State:");
                        DumpMachine();
                        Reset(target);
                        taskRef(target);
                        Message("Expected State:");
                        DumpMachine();
                
                        return false;
                    }
                }`,
                referenceImplementatuon: "Reference",
                placeholderImplementation: `
namespace Kata {
    operation ApplyY(q : Qubit) : Unit is Adj + Ctl {
        // ...

    }
}`
            },
            {
                id: "single-qubit-gates_global-phase-i",
                title: "Global Phase i",
                description: "Description for global phase i",
                verificationImplementation: "Verification",
                referenceImplementatuon: "Reference",
                placeholderImplementation: "Placeholder"
            }
        ]
    },
    {
        id: "multi-qubit-gates",
        title: "Multi-Qubit Gates",
        description: "Description for multi qubit gates kata.",
        exercises: [
            {
                id: "multi-qubit-gates_cnot-gate",
                title: "CNOT Gate",
                description: "Description for CNOT gate",
                verificationImplementation: "Verification",
                referenceImplementatuon: "Reference",
                placeholderImplementation: "Placeholder"
            }
        ]
    }
];

export function getAllKatas() : Kata[] {
    return katas as Kata[];
}

export function getKataModule(id: string) : KataModule {
    let filteredModules = modules.filter(m => m.id == id);
    if (filteredModules.length != 1)
    {
        throw new Error(`Failed to get module with id: ${id}`);
    }

    return filteredModules.at(0)!;
}

export function getKataExercise(id: string) : KataExercise {
    for (let module of modules)
    {
        let filteredExercises = module.exercises.filter(e => e.id == id);
        if (filteredExercises.length == 1)
        {
            return filteredExercises.at(0)!;
        }
    }

    throw new Error(`Failed to get exercise with id: ${id}`);
}

export function queryKataModules() : KataModule[] {
    return modules;
}

export function verifyKata(id: string, kataImplementation: string, eventCb: (msg: string) => void) : boolean
{
    let exercise = getKataExercise(id);
    return run_kata_implementation(
        exercise.verificationImplementation,
        kataImplementation,
        eventCb);
}