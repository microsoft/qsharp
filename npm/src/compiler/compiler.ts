// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { log } from "../log.js";
import { VSDiagnostic, mapDiagnostics } from "../vsdiagnostic.js";
import { IServiceProxy, ServiceState } from "../worker-proxy.js";
import { eventStringToMsg } from "./common.js";
import { IQscEventTarget, QscEvents, makeEvent } from "./events.js";

// The wasm types generated for the node.js bundle are just the exported APIs,
// so use those as the set used by the shared compiler
type Wasm = typeof import("../../lib/node/qsc_wasm.cjs");

// These need to be async/promise results for when communicating across a WebWorker, however
// for running the compiler in the same thread the result will be synchronous (a resolved promise).
export interface ICompiler {
  checkCode(code: string): Promise<VSDiagnostic[]>;
  getHir(code: string): Promise<string>;
  run(
    code: string,
    expr: string,
    shots: number,
    eventHandler: IQscEventTarget,
  ): Promise<void>;
  getQir(code: string): Promise<string>;
  checkExerciseSolution(
    user_code: string,
    exercise_sources: string[],
    eventHandler: IQscEventTarget,
  ): Promise<boolean>;
}

// WebWorker also support being explicitly terminated to tear down the worker thread
export type ICompilerWorker = ICompiler & IServiceProxy;
export type CompilerState = ServiceState;

export class Compiler implements ICompiler {
  private wasm: Wasm;

  constructor(wasm: Wasm) {
    log.info("Constructing a Compiler instance");
    this.wasm = wasm;
    globalThis.qscGitHash = this.wasm.git_hash();
  }

  async checkCode(code: string): Promise<VSDiagnostic[]> {
    let diags: VSDiagnostic[] = [];
    const languageService = new this.wasm.LanguageService(
      (uri: string, version: number | undefined, errors: VSDiagnostic[]) => {
        diags = errors;
      },
    );
    languageService.update_document("code", 1, code);
    return mapDiagnostics(diags, code);
  }

  async getQir(code: string): Promise<string> {
    return this.wasm.get_qir(code);
  }

  async getHir(code: string): Promise<string> {
    return this.wasm.get_hir(code);
  }

  async run(
    code: string,
    expr: string,
    shots: number,
    eventHandler: IQscEventTarget,
  ): Promise<void> {
    // All results are communicated as events, but if there is a compiler error (e.g. an invalid
    // entry expression or similar), it may throw on run. The caller should expect this promise
    // may reject without all shots running or events firing.
    this.wasm.run(
      code,
      expr,
      (msg: string) => onCompilerEvent(msg, eventHandler),
      shots,
    );
  }

  async checkExerciseSolution(
    user_code: string,
    exercise_sources: string[],
    eventHandler: IQscEventTarget,
  ): Promise<boolean> {
    const success = this.wasm.check_exercise_solution(
      user_code,
      exercise_sources,
      (msg: string) => onCompilerEvent(msg, eventHandler),
    );

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
  log.debug("worker dispatching event " + JSON.stringify(qscEvent));
  eventTarget.dispatchEvent(qscEvent);
}
