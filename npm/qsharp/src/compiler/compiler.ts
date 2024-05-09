// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { type Circuit as CircuitData } from "@microsoft/quantum-viz.js/lib/circuit.js";
import {
  IDocFile,
  IOperationInfo,
  TargetProfile,
  type VSDiagnostic,
} from "../../lib/web/qsc_wasm.js";
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
type Wasm = typeof import("../../lib/web/qsc_wasm.js");

// These need to be async/promise results for when communicating across a WebWorker, however
// for running the compiler in the same thread the result will be synchronous (a resolved promise).
export interface ICompiler {
  checkCode(code: string): Promise<VSDiagnostic[]>;

  getAst(
    code: string,
    languageFeatures?: string[],
    profile?: TargetProfile,
  ): Promise<string>;

  getHir(
    code: string,
    languageFeatures?: string[],
    profile?: TargetProfile,
  ): Promise<string>;

  /** @deprecated -- switch to using `ProgramConfig`-based overload. Instead of passing
   * sources and language features separately, pass an object with named properties. This change was made
   * for the sake of extensibility and future-compatibility. Note that only the new API
   * supports passing language features. If you need to pass language features, you must use
   * the new API.
   **/
  run(
    sources: [string, string][],
    expr: string,
    shots: number,
    eventHandler: IQscEventTarget,
  ): Promise<void>;
  run(
    config: ProgramConfig,
    expr: string,
    shots: number,
    eventHandler: IQscEventTarget,
  ): Promise<void>;

  /** @deprecated -- switch to using `ProgramConfig`-based overload. Instead of passing
   * sources and language features separately, pass an object with named properties. This change was made
   * for the sake of extensibility and future-compatibility. Note that only the new API
   * supports passing language features. If you need to pass language features, you must use
   * the new API.
   **/
  getQir(
    sources: [string, string][],
    languageFeatures?: string[],
  ): Promise<string>;
  getQir(config: ProgramConfig): Promise<string>;

  /** @deprecated -- switch to using `ProgramConfig`-based overload. Instead of passing
   * sources and language features separately, pass an object with named properties. This change was made
   * for the sake of extensibility and future-compatibility. Note that only the new API
   * supports passing language features. If you need to pass language features, you must use
   * the new API.
   **/
  getEstimates(
    sources: [string, string][],
    params: string,
    languageFeatures?: string[],
  ): Promise<string>;
  getEstimates(config: ProgramConfig, params: string): Promise<string>;
  getCircuit(
    config: ProgramConfig,
    target: TargetProfile,
    simulate: boolean,
    operation?: IOperationInfo,
  ): Promise<CircuitData>;

  getDocumentation(
    additionalSources?: [string, string][],
    targetProfile?: string,
    languageFeatures?: string[],
  ): Promise<IDocFile[]>;

  checkExerciseSolution(
    userCode: string,
    exerciseSources: string[],
    eventHandler: IQscEventTarget,
  ): Promise<boolean>;
}

/** Type definition for the configuration of a program. */
export type ProgramConfig = {
  /** An array of source objects, each containing a name and contents. */
  sources: [string, string][];
  /** An array of language features to be opted in to in this compilation. */
  languageFeatures?: string[];
  /** Target compilation profile. */
  profile?: TargetProfile;
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
    sourcesOrConfig: [string, string][] | ProgramConfig,
    languageFeatures?: string[],
  ): Promise<string> {
    if (Array.isArray(sourcesOrConfig)) {
      return this.deprecatedGetQir(sourcesOrConfig, languageFeatures || []);
    } else {
      const config = sourcesOrConfig as ProgramConfig;
      return this.newGetQir(config);
    }
  }

  async newGetQir({
    sources,
    languageFeatures = [],
    profile = "base",
  }: ProgramConfig): Promise<string> {
    return this.wasm.get_qir(sources, languageFeatures, profile);
  }

  async deprecatedGetQir(
    sources: [string, string][],
    languageFeatures: string[],
  ): Promise<string> {
    return this.wasm.get_qir(sources, languageFeatures, "base");
  }

  async getEstimates(
    sourcesOrConfig: [string, string][] | ProgramConfig,
    params: string,
    languageFeatures: string[] = [],
  ): Promise<string> {
    if (Array.isArray(sourcesOrConfig)) {
      return this.deprecatedGetEstimates(
        sourcesOrConfig,
        params,
        languageFeatures,
      );
    } else {
      return this.newGetEstimates(sourcesOrConfig, params);
    }
  }

  async newGetEstimates(
    { sources, languageFeatures }: ProgramConfig,
    params: string,
  ): Promise<string> {
    return this.wasm.get_estimates(sources, params, languageFeatures || []);
  }

  async deprecatedGetEstimates(
    sources: [string, string][],
    params: string,
    languageFeatures: string[],
  ): Promise<string> {
    return this.wasm.get_estimates(sources, params, languageFeatures);
  }

  async getAst(
    code: string,
    languageFeatures?: string[],
    profile?: TargetProfile,
  ): Promise<string> {
    return this.wasm.get_ast(
      code,
      languageFeatures ?? [],
      profile ?? "adaptive_ri",
    );
  }

  async getHir(
    code: string,
    languageFeatures: string[],
    profile: TargetProfile,
  ): Promise<string> {
    return this.wasm.get_hir(
      code,
      languageFeatures ?? [],
      profile ?? "adaptive_ri",
    );
  }

  async run(
    sourcesOrConfig: [string, string][] | ProgramConfig,
    expr: string,
    shots: number,
    eventHandler: IQscEventTarget,
  ): Promise<void> {
    let sources;
    let languageFeatures: string[] = [];

    if (Array.isArray(sourcesOrConfig)) {
      // this is the deprecated API
      sources = sourcesOrConfig;
    } else {
      // this is the new API
      sources = sourcesOrConfig.sources;
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

  async getCircuit(
    config: ProgramConfig,
    target: TargetProfile,
    simulate: boolean,
    operation?: IOperationInfo,
  ): Promise<CircuitData> {
    return this.wasm.get_circuit(
      config.sources,
      target,
      config.languageFeatures || [],
      simulate,
      operation,
    );
  }

  // Returns all autogenerated documentation files for the standard library
  // and loaded project (if requested). This include file names and metadata,
  // including specially formatted table of content file.
  async getDocumentation(
    additionalSources?: [string, string][],
    targetProfile?: string,
    languageFeatures?: string[],
  ): Promise<IDocFile[]> {
    // Get documentation from wasm layer
    return this.wasm.generate_docs(
      additionalSources,
      targetProfile,
      languageFeatures,
    );
  }

  async checkExerciseSolution(
    userCode: string,
    exerciseSources: string[],
    eventHandler: IQscEventTarget,
  ): Promise<boolean> {
    const success = this.wasm.check_exercise_solution(
      userCode,
      exerciseSources,
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
      qscEvent = makeEvent("DumpMachine", {
        state: qscMsg.state,
        stateLatex: qscMsg.stateLatex,
      });
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
    getAst: "request",
    getHir: "request",
    getQir: "request",
    getEstimates: "request",
    getCircuit: "request",
    getDocumentation: "request",
    run: "requestWithProgress",
    checkExerciseSolution: "requestWithProgress",
  },
  eventNames: ["DumpMachine", "Message", "Result"],
};
