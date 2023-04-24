// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import type {IDiagnostic, ICompletionList} from "../lib/node/qsc_wasm.cjs";
import { log } from "./log.js";
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
    runKata(user_code: string, verify_code: string): Promise<boolean>;
    isRunning(): boolean;
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
    private _isRunning: boolean = false; // To avoid reentrancy when processing a callback
    private currentSource: string = ""; // Store when running to map any source positions in events

    constructor(wasm: Wasm, callbacks: CompilerEvents) {
        log.info("Constructing a Compiler instance");
        this.wasm = wasm;
        this.callbacks = callbacks;
    }

    async checkCode(code: string): Promise<VSDiagnostic[]> {
        if (this._isRunning) throw "Compiler invoked while already running";
        let raw_result = this.wasm.check_code(code) as IDiagnostic[];
        return mapDiagnostics(raw_result, code);
    }

    async getCompletions(): Promise<ICompletionList> {
        if (this._isRunning) throw "Compiler invoked while already running";
        return this.wasm.get_completions();
    }

    async run(code: string, expr: string, shots: number): Promise<void> {
        if (this._isRunning) throw "Compiler invoked while already running";
        this._isRunning = true;
        this.currentSource = code;
        try {
            this.wasm.run(code, expr, this.mapEvent.bind(this), shots);
        } catch(e) {
            // Likely something failed before success/failure got reported.
            this.callbacks.onFailure(e);
            throw e;
        } finally {
            this._isRunning = false;
            this.currentSource = "";
        }
    }

    async runKata(user_code: string, verify_code: string): Promise<boolean> {
        if (this._isRunning) throw "Compiler invoked while running other code";
        this._isRunning = true;
        this.currentSource = user_code;
        let success = false;
        let err: any = null;
        try {
            success = this.wasm.run_kata_exercise(verify_code, user_code, this.mapEvent.bind(this));
        } catch(e) {
            err = e;
        } finally {
            this._isRunning = false;
            this.currentSource = "";
        }
        // Currently the kata wasm doesn't emit the success/failure events, so do those here.
        if (success) {
            this.callbacks.onSuccess("true");
        } else {
            this.callbacks.onFailure(JSON.stringify(err));
        }
        return success;
    }

    isRunning(): boolean {
        return this._isRunning;
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
                log.error(`Unrecognized event: ${msg}`);
                break;
        }
    }
}
