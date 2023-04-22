// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import type {IDiagnostic, ICompletionList} from "../lib/node/qsc_wasm.cjs";
import { DumpMsg, MessageMsg, eventStringToMsg, mapDiagnostics, VSDiagnostic } from "./common.js";

// The wasm types generated for the node.js bundle are just the exported APIs,
// so use those as the set used by the shared compiler
type Wasm = typeof import("../lib/node/qsc_wasm.cjs")

// These need to be async/promise results for when communicating across a WebWorker, however
// for running the compiler in the same thread the result will be synchronous (a resolved promise).
export interface ICompiler {
    checkCode(code: string): Promise<VSDiagnostic[]>;
    getCompletions(): Promise<ICompletionList>;
    run(code: string, expr: string, shots: number): Promise<void>;
    runKata(user_code: string, verify_code: string): Promise<void>;
}

// WebWorker also support being explicitly terminated to tear down the worker thread
export type ICompilerWorker = ICompiler & { terminate: () => void };

export interface CompilerEvents {
    onMessage: (msg: MessageMsg) => void;
    onDumpMachine: (dump: DumpMsg) => void;
    onSuccess: (result: string) => void;
    onFailure: (err: any) => void;
}

export class Compiler implements ICompiler {
    private wasm: Wasm;
    private callbacks: CompilerEvents;
    private isRunning: boolean = false; // To avoid reentrancy when processing a callback
    private currentSource: string = ""; // Store to map any source positions in events

    constructor(wasm: Wasm, callbacks: CompilerEvents) {
        this.wasm = wasm;
        this.callbacks = callbacks;
    }

    checkCode(code: string): Promise<VSDiagnostic[]> {
        if (this.isRunning) throw "Compiler invoked while already running";
        let result = this.wasm.check_code(code) as IDiagnostic[];
        return Promise.resolve(mapDiagnostics(result, code));
    }

    getCompletions(): Promise<ICompletionList> {
        if (this.isRunning) throw "Compiler invoked while already running";
        return Promise.resolve(this.wasm.get_completions());
    }

    run(code: string, expr: string, shots: number): Promise<void> {
        if (this.isRunning) throw "Compiler invoked while already running";
        this.isRunning = true;
        this.currentSource = code;
        try {
            this.wasm.run(code, expr, (ev: string) => this.mapEvent(ev), shots);
        } finally {
            this.isRunning = false;
            this.currentSource = "";
        }
        return Promise.resolve();
    }

    runKata(user_code: string, verify_code: string) {
        if (this.isRunning) throw "Compiler invoked while running other code";
        this.isRunning = true;
        this.currentSource = user_code;
        try {
            this.wasm.run_kata_exercise(verify_code, user_code, (ev: string) => this.mapEvent(ev));
        } finally {
            this.isRunning = false;
            this.currentSource = "";
        }
        return Promise.resolve();
    }

    private mapEvent(eventMsg: string): void {
        // Take a generic event string and call the appropriate callback
        const msg = eventStringToMsg(eventMsg);
        if (!msg) throw `Unknown message: ${eventMsg}`;

        switch (msg.type) {
            case "Message":
                this.callbacks.onMessage(msg);
                break;
            case "DumpMachine":
                this.callbacks.onDumpMachine(msg);
                break;
            case "Result":
                if (msg.success) {
                    this.callbacks.onSuccess(msg.result as string);
                } else {
                    // If it's a diagnostic, positions need to be mapped.
                    let result = msg.result;
                    if (typeof result === "object" && typeof result.start_pos === "number") {
                        result = mapDiagnostics([result], this.currentSource)[0];
                    }
                    this.callbacks.onFailure(result);
                }
                break;
            default:
                console.error(`Unrecognized event: ${msg}`);
                break;
        }
    }
}
