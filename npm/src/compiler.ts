// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import type {
  IDiagnostic,
  ICompletionList,
  IHover,
  IDefinition,
  QSharpLanguageService,
} from "../lib/node/qsc_wasm.cjs";
import { log } from "./log.js";
import {
  eventStringToMsg,
  mapDiagnostics,
  mapUtf16UnitsToUtf8Units,
  mapUtf8UnitsToUtf16Units,
  VSDiagnostic,
} from "./common.js";
import { IQscEventTarget, QscEvents, makeEvent } from "./events.js";

// The wasm types generated for the node.js bundle are just the exported APIs,
// so use those as the set used by the shared compiler
type Wasm = typeof import("../lib/node/qsc_wasm.cjs");

// These need to be async/promise results for when communicating across a WebWorker, however
// for running the compiler in the same thread the result will be synchronous (a resolved promise).
export type CompilerState = "idle" | "busy";
export interface ICompiler {
  updateCode(documentUri: string, code: string): Promise<void>;
  getCompletions(
    documentUri: string,
    code: string,
    offset: number
  ): Promise<ICompletionList>;
  getHover(documentUri: string, code: string, offset: number): Promise<IHover>;
  getDefinition(
    documentUri: string,
    code: string,
    offset: number
  ): Promise<IDefinition>;
  run(code: string, expr: string, shots: number): Promise<void>;
  runKata(user_code: string, verify_code: string): Promise<boolean>;
  onstatechange: ((state: CompilerState) => void) | null;
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
  private eventHandler: IQscEventTarget;
  private languageService: QSharpLanguageService;
  // We only need to keep a copy of the code for mapping diagnostics
  // It would be much better if the wasm layer could do the utf16 mapping
  // but here we are
  private code: { [uri: string]: string } = {};

  onstatechange: ((state: CompilerState) => void) | null = null;

  constructor(wasm: Wasm, eventHandler: IQscEventTarget) {
    log.info("Constructing a Compiler instance");
    this.wasm = wasm;
    this.eventHandler = eventHandler;
    globalThis.qscGitHash = this.wasm.git_hash();
    this.languageService = new this.wasm.QSharpLanguageService(
      this.onDiagnostics.bind(this),
      (msg: string) => {
        log.info(msg);
      }
    );
    // TODO: should call free() on this at some point?
  }

  async updateCode(documentUri: string, code: string): Promise<void> {
    this.code[documentUri] = code;
    this.languageService.update_code(documentUri, code);
  }

  onDiagnostics(diagnostics: IDiagnostic[]) {
    try {
      // TODO: use the uri of course
      const code = Object.values(this.code)[0];
      this.eventHandler.dispatchEvent(
        // Oh no, I don't have the source here to do the utf16 mapping 😱
        makeEvent("diagnostics", mapDiagnostics(diagnostics, code))
      );
    } catch (e) {
      log.error("Error in onCheck", e);
    }
  }

  async getCompletions(
    documentUri: string,
    code: string,
    offset: number
  ): Promise<ICompletionList> {
    const convertedOffset = mapUtf16UnitsToUtf8Units([offset], code)[offset];
    return this.languageService.get_completions(documentUri, convertedOffset);
  }

  async getHover(documentUri: string, code: string, offset: number) {
    const convertedOffset = mapUtf16UnitsToUtf8Units([offset], code)[offset];
    return this.languageService.get_hover(documentUri, convertedOffset);
  }

  async getDefinition(documentUri: string, code: string, offset: number) {
    const convertedOffset = mapUtf16UnitsToUtf8Units([offset], code)[offset];
    const result = this.languageService.get_definition(
      documentUri,
      convertedOffset
    );
    result.offset = mapUtf8UnitsToUtf16Units([result.offset], code)[
      result.offset
    ];
    return result;
  }

  async run(code: string, expr: string, shots: number): Promise<void> {
    // All results are communicated as events, but if there is a compiler error (e.g. an invalid
    // entry expression or similar), it may throw on run. The caller should expect this promise
    // may reject without all shots running or events firing.
    if (this.onstatechange) this.onstatechange("busy");

    this.wasm.run(
      code,
      expr,
      (msg: string) => onCompilerEvent(msg, this.eventHandler),
      shots
    );

    if (this.onstatechange) this.onstatechange("idle");
  }

  async runKata(user_code: string, verify_code: string): Promise<boolean> {
    let success = false;
    let err: any = null; // eslint-disable-line @typescript-eslint/no-explicit-any
    try {
      if (this.onstatechange) this.onstatechange("busy");
      success = this.wasm.run_kata_exercise(
        verify_code,
        user_code,
        (msg: string) => onCompilerEvent(msg, this.eventHandler)
      );
    } catch (e) {
      err = e;
    }
    if (this.onstatechange) this.onstatechange("idle");
    // Currently the kata wasm doesn't emit the success/failure events, so do those here.
    if (!err) {
      const evt = makeEvent("Result", {
        success: true,
        value: success.toString(),
      });
      this.eventHandler.dispatchEvent(evt);
    } else {
      const diag = errToDiagnostic(err);
      const evt = makeEvent("Result", { success: false, value: diag });
      this.eventHandler.dispatchEvent(evt);
    }
    return success;
  }
}

function onCompilerEvent(msg: string, eventTarget: IQscEventTarget) {
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
    case "diagnostics":
      qscEvent = makeEvent("diagnostics", qscMsg.diagnostics);
      break;
    default:
      log.never(msgType);
      throw "Unexpected message type";
  }
  eventTarget.dispatchEvent(qscEvent);
}
