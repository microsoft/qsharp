// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import type { IDiagnostic, ICompletionList } from "../lib/node/qsc_wasm.cjs";
import { log } from "./log.js";
import { eventStringToMsg, mapDiagnostics, VSDiagnostic } from "./common.js";
import { IQscEventTarget, QscEvents, makeEvent } from "./events.js";

// The wasm types generated for the node.js bundle are just the exported APIs,
// so use those as the set used by the shared compiler
type Wasm = typeof import("../lib/node/qsc_wasm.cjs");

// These need to be async/promise results for when communicating across a WebWorker, however
// for running the compiler in the same thread the result will be synchronous (a resolved promise).
export interface ICompiler {
  checkCode(code: string): Promise<VSDiagnostic[]>;
  getCompletions(): Promise<ICompletionList>;
  run(
    code: string,
    expr: string,
    shots: number,
    eventHandler: IQscEventTarget
  ): Promise<void>;
  runKata(
    user_code: string,
    verify_code: string,
    eventHandler: IQscEventTarget
  ): Promise<boolean>;
}

// WebWorker also support being explicitly terminated to tear down the worker thread
export type ICompilerWorker = ICompiler & { terminate: () => void };

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function errToDiagnostic(err: any): VSDiagnostic {
  if (
    err &&
    typeof err.severity === "number" &&
    typeof err.message === "string"
  ) {
    err.start_pos = err.start_pos || 0;
    err.end_pos = err.end_pos || 0;
    return err;
  } else {
    return {
      severity: 0,
      message: err.toString(),
      start_pos: 0,
      end_pos: 0,
    };
  }
}

export class Compiler implements ICompiler {
  private wasm: Wasm;

  constructor(wasm: Wasm) {
    log.info("Constructing a Compiler instance");
    this.wasm = wasm;
  }

  async checkCode(code: string): Promise<VSDiagnostic[]> {
    const raw_result = this.wasm.check_code(code) as IDiagnostic[];
    return mapDiagnostics(raw_result, code);
  }

  async getCompletions(): Promise<ICompletionList> {
    return this.wasm.get_completions();
  }

  async run(
    code: string,
    expr: string,
    shots: number,
    eventHandler: IQscEventTarget
  ): Promise<void> {
    // All results are communicated as events, but if there is a compiler error (e.g. an invalid
    // entry expression or similar), it may throw on run. The caller should expect this promise
    // may reject without all shots running or events firing.
    this.wasm.run(
      code,
      expr,
      (msg: string) => onCompilerEvent(msg, eventHandler),
      shots
    );
  }

  async runKata(
    user_code: string,
    verify_code: string,
    eventHandler: IQscEventTarget
  ): Promise<boolean> {
    let success = false;
    let err: any = null; // eslint-disable-line @typescript-eslint/no-explicit-any
    try {
      success = this.wasm.run_kata_exercise(
        verify_code,
        user_code,
        (msg: string) => onCompilerEvent(msg, eventHandler)
      );
    } catch (e) {
      err = e;
    }
    // Currently the kata wasm doesn't emit the success/failure events, so do those here.
    if (!err) {
      const evt = makeEvent("Result", {
        success: true,
        value: success.toString(),
      });
      eventHandler.dispatchEvent(evt);
    } else {
      const diag = errToDiagnostic(err);
      const evt = makeEvent("Result", { success: false, value: diag });
      eventHandler.dispatchEvent(evt);
    }
    return success;
  }
}

export function onCompilerEvent(msg: string, eventTarget: IQscEventTarget) {
  const qscMsg = eventStringToMsg(msg);
  if (!qscMsg) {
    log.error("Unknown event message: %s", msg);
    return;
  }

  let qscEvent: QscEvents;

  const msgType = qscMsg.type;
  switch (msgType) {
    case "Message":
      qscEvent = makeEvent("Message", qscMsg.message);
      break;
    case "DumpMachine":
      qscEvent = makeEvent("DumpMachine", qscMsg.state);
      break;
    case "Result":
      qscEvent = makeEvent("Result", qscMsg.result);
      break;
    default:
      log.never(msgType);
      throw "Unexpected message type";
  }
  eventTarget.dispatchEvent(qscEvent);
}
