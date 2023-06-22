// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { ICompletionList } from "../../lib/web/qsc_wasm.js";
import { log } from "../log.js";
import { VSDiagnostic } from "../vsdiagnostic.js";
import { EventMessageWithType, IServiceEventTarget, RequestMessage, ResponseMessageWithType, createWorkerProxy, getWorkerEventHandlersGeneric } from "../worker-common.js";
import { ICompiler, ICompilerWorker } from "./compiler.js";
import {
  QscEventData,
  type QscEvents,
} from "./events.js";

//////////////////////////////////////
/// EVENTS
type IQscEventTarget = IServiceEventTarget<QscEvents>;
type CompilerEventMsg = QscEventData;
export function getWorkerEventHandlers(
  postMessage: (msg: CompilerEventMsg) => void
): IQscEventTarget {
  return getWorkerEventHandlersGeneric<
    QscEvents
  >(["DumpMachine", "Message", "Result"], postMessage);
}
//////////////////////////////////////

//////////////////////////////////////
/// REQUESTS
type CompilerMethods = Omit<ICompiler, "onstatechange">;
const requests: { [M in keyof CompilerMethods] : { longRunning: boolean }  }= {
  checkCode: { longRunning: false },
  getHir: { longRunning: false },
  getCompletions: { longRunning: false },
  run: { longRunning: true },
  runKata: { longRunning: true }
}
//////////////////////////////////////


type CompilerRespMsg =
  | { type: "checkCode-result"; result: VSDiagnostic[] }
  | { type: "getHir-result"; result: string }
  | { type: "getCompletions-result"; result: ICompletionList }
  | { type: "run-result"; result: void }
  | { type: "runKata-result"; result: boolean }
  | { type: "error-result"; result: any }; // eslint-disable-line @typescript-eslint/no-explicit-any


export type ResponseMsgType =
  | ResponseMessageWithType<CompilerRespMsg>
  | EventMessageWithType<CompilerEventMsg>;

function makeResult(msg: CompilerRespMsg) {
  const msgType = msg.type;
  switch (msgType) {
    // Response type messages. Resolve and complete this request.
    case "checkCode-result":
    case "getHir-result":
    case "getCompletions-result":
    case "run-result":
    case "runKata-result":
      return { success: true, data: msg.result };

    case "error-result":
      // Something unexpected failed the request. Reject and move on.
      return { success: false, data: msg.result };

    default:
      log.never(msg);
      return null;
  }
}

export function createCompilerProxy(
  postMessage: (msg: RequestMessage<CompilerMethods>) => void,
  setMsgHandler: (handler: (e: ResponseMsgType) => void) => void,
  terminator: () => void
) {
  return createWorkerProxy<
  RequestMessage<CompilerMethods>,
    CompilerRespMsg,
    QscEvents,
    CompilerEventMsg,
    IQscEventTarget,
    ICompilerWorker
  >(
    postMessage,
    setMsgHandler,
    terminator,
    requests,
    makeResult
  );
}
