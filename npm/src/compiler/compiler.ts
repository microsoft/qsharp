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
  /**
   * @deprecated use the language service for errors and other editor features.
   */
  checkCode(code: string): Promise<VSDiagnostic[]>;
  getHir(code: string): Promise<string>;
  run(
    code: string,
    expr: string,
    shots: number,
    eventHandler: IQscEventTarget
  ): Promise<void>;
  getQir(code: string): Promise<string>;
  checkExerciseSolution(
    user_code: string,
    exercise_sources: string[],
    eventHandler: IQscEventTarget
  ): Promise<boolean>;
}

// WebWorker also support being explicitly terminated to tear down the worker thread
export type ICompilerWorker = ICompiler & IServiceProxy;
export type CompilerState = ServiceState;

type Entry = File | Dir 

interface File {
  t: 'file',
  name: string,
  contents: string
}

interface Dir {
  t: 'dir'
  name: string,
  entries: Entry[]
}

function name(entry: Entry) : string {
  return entry.name
}


function lookupFn (root: Dir, path: string) : File | null{
  for (const entry of root.entries) {
    if (entry.t === 'file' && name(entry) === path) {
      return entry;
    }
  }

  for (const entry of root.entries) {
    if (entry.t === 'dir') {
      let result= lookupFn(entry, path);
      if (result !== null) { return result; }
    }
  }

  return null
}
function listDir (root: Dir, path: string) : Entry[] {
  for (const entry of root.entries) {
    if (entry.t === 'dir' && name(entry) === path) {
      return entry.entries;
    }
  }

  for (const entry of root.entries) {
    if (entry.t === 'dir') {
      let result= listDir(entry, path);
      if (result.length !== 0) { return result; }
    }
  }

  return [];
}

export class Compiler implements ICompiler {
  private wasm: Wasm;

  constructor(wasm: Wasm) {
    log.info("Constructing a Compiler instance");
    this.wasm = wasm;
    globalThis.qscGitHash = this.wasm.git_hash();
  }

  /**
   * @deprecated use the language service for errors and other editor features.
   */
  async checkCode(code: string): Promise<VSDiagnostic[]> {
    let diags: VSDiagnostic[] = [];
    const languageService = new this.wasm.LanguageService(
      (uri: string, version: number | undefined, errors: VSDiagnostic[]) => {
        diags = errors;
      }
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


  // TODO return type -- not sure if this is correct
  async loadProject(files: Dir): Promise<string[]> {
    const lookup_fn = (path: string): string | undefined =>  files[path];
    // TODO below fn
    const listDirFn= (path: string) => listDir(files, path);

    const projectLoader = new this.wasm.ProjectLoader(lookup_fn, listDirFn);

    const manifestDescriptor = new this.wasm.ManifestDescriptor(["TODO exclude files"], ["TODO exclude regexes"], "TODO root dir");

    const project = projectLoader.load( manifestDescriptor);
    
    log.info(JSON.stringify(project, null, 2));

    return project
;  }

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

  async checkExerciseSolution(
    user_code: string,
    exercise_sources: string[],
    eventHandler: IQscEventTarget
  ): Promise<boolean> {
    const success = this.wasm.check_exercise_solution(
      user_code,
      exercise_sources,
      (msg: string) => onCompilerEvent(msg, eventHandler)
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
