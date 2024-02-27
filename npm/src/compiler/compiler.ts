// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { type VSDiagnostic } from "../../lib/web/qsc_wasm.js";
import { log } from "../log.js";
import {
  IServiceProxy,
  ServiceProtocol,
  ServiceState,
} from "../workers/common.js";
import { eventStringToMsg } from "./common.js";
import {
  IQscEventTarget,
  QscEventData,
  QscEvents,
  makeEvent,
} from "./events.js";

// The wasm types generated for the node.js bundle are just the exported APIs,
// so use those as the set used by the shared compiler
type Wasm = typeof import("../../lib/node/qsc_wasm.cjs");

// These need to be async/promise results for when communicating across a WebWorker, however
// for running the compiler in the same thread the result will be synchronous (a resolved promise).
export interface ICompiler {
  checkCode(code: string): Promise<VSDiagnostic[]>;
  getHir(code: string, languageFeatures?: string[]): Promise<string>;
  /** @deprecated -- switch to using `ProgramConfig`-based overload. Instead of passing
   * all arguments separately, pass an object with named properties. This change was made
   * for the sake of extensibility and future-compatibility. Note that only the new API
   * supports passing guage features. If you need to pass language features, you must use
   * the new API.
   **/
  run(
    sources: [string, string][],
    expr: string,
    shots: number,
    eventHandler: IQscEventTarget,
  ): Promise<void>;
  run(config: ProgramConfig, eventHandler: IQscEventTarget): Promise<void>;
  getQir(
    sources: [string, string][],
    languageFeatures?: string[],
  ): Promise<string>;
  getEstimates(
    sources: [string, string][],
    params: string,
    languageFeatures?: string[],
  ): Promise<string>;
  checkExerciseSolution(
    user_code: string,
    exercise_sources: string[],
    eventHandler: IQscEventTarget,
  ): Promise<boolean>;
}

/** Type definition for the configuration of a program. */
export type ProgramConfig = {
  /** An array of source objects, each containing a name and contents. */
  sources: [string, string][];
  /** The entry expression to be evaluated. */
  expr: string;
  /** The number of shots to be performed in the quantum simulation. */
  shots: number;
  /** An array of language features to be opted in to in this compilation. */
  languageFeatures?: string[];
};

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

  // Note: This function does not support project mode.
  // see https://github.com/microsoft/qsharp/pull/849#discussion_r1409821143
  async checkCode(code: string): Promise<VSDiagnostic[]> {
    let diags: VSDiagnostic[] = [];
    const languageService = new this.wasm.LanguageService();
    const work = languageService.start_background_work(
      (uri: string, version: number | undefined, errors: VSDiagnostic[]) => {
        diags = errors;
      },
      () => Promise.resolve(null),
      () => Promise.resolve([]),
      () => Promise.resolve(null),
    );
    languageService.update_document("code", 1, code);
    // Yield to let the language service background worker handle the update
    await Promise.resolve();
    languageService.stop_background_work();
    await work;
    languageService.free();
    return diags;
  }

  async getQir(
    sources: [string, string][],
    languageFeatures: string[],
  ): Promise<string> {
    return this.wasm.get_qir(sources, languageFeatures);
  }

  async getEstimates(
    sources: [string, string][],
    params: string,
    languageFeatures: string[],
  ): Promise<string> {
    return this.wasm.get_estimates(sources, params, languageFeatures);
  }

  async getHir(code: string, languageFeatures: string[]): Promise<string> {
    return this.wasm.get_hir(code, languageFeatures);
  }

  async run(
    sourcesOrConfig: [string, string][] | ProgramConfig,
    exprOrEventHandler: string | IQscEventTarget,
    maybeShots?: number,
    maybeEventHandler?: IQscEventTarget,
  ): Promise<void> {
    let sources;
    let expr;
    let shots;
    let eventHandler: IQscEventTarget | undefined;
    let languageFeatures: string[] = [];

    if (Array.isArray(sourcesOrConfig)) {
      // this is the deprecated API
      sources = sourcesOrConfig;
      expr = exprOrEventHandler as string;
      shots = maybeShots;
      eventHandler = maybeEventHandler;
    } else {
      // this is the new API
      sources = sourcesOrConfig.sources;
      expr = sourcesOrConfig.expr;
      shots = sourcesOrConfig.shots;
      eventHandler = exprOrEventHandler as IQscEventTarget;
      languageFeatures = sourcesOrConfig.languageFeatures || [];
    }
    // All results are communicated as events, but if there is a compiler error (e.g. an invalid
    // entry expression or similar), it may throw on run. The caller should expect this promise
    // may reject without all shots running or events firing.
    this.wasm.run(
      sources,
      expr,
      (msg: string) => onCompilerEvent(msg, eventHandler!),
      shots!,
      languageFeatures,
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

/** The protocol definition to allow running the compiler in a worker. */
export const compilerProtocol: ServiceProtocol<ICompiler, QscEventData> = {
  class: Compiler,
  methods: {
    checkCode: "request",
    getHir: "request",
    getQir: "request",
    getEstimates: "request",
    run: "requestWithProgress",
    checkExerciseSolution: "requestWithProgress",
  },
  eventNames: ["DumpMachine", "Message", "Result"],
};
