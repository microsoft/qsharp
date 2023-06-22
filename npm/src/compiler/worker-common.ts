// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { log } from "../log.js";
import { ICompiler, ICompilerWorker } from "./compiler.js";
import {
  makeEvent as makeCompilerEvent,
  type QscEvents,
  QscEvent,
} from "./events.js";

import { ICompletionList } from "../../lib/web/qsc_wasm.js";
import { DumpMsg, MessageMsg } from "./common.js";
import { VSDiagnostic } from "../vsdiagnostic.js";
import { EventMessageWithType, IServiceEventTarget, ResponseMessageWithType, ServiceState, createWorkerProxy, getWorkerEventHandlersGeneric, invokeWorkerMethod } from "../worker-common.js";

type IQscEventTarget = IServiceEventTarget<QscEvents>;

export type CompilerReqMsg =
  | { type: "checkCode"; args: [string] }
  | { type: "getHir"; args: [string] }
  | { type: "getCompletions"; args: [] }
  | { type: "run"; args: [string, string, number] }
  | { type: "runKata"; args: [string, string] };

type CompilerRespMsg =
  | { type: "checkCode-result"; result: VSDiagnostic[] }
  | { type: "getHir-result"; result: string }
  | { type: "getCompletions-result"; result: ICompletionList }
  | { type: "run-result"; result: void }
  | { type: "runKata-result"; result: boolean }
  | { type: "error-result"; result: any }; // eslint-disable-line @typescript-eslint/no-explicit-any


type CompilerEventMsg =
  | { type: "message-event"; event: MessageMsg }
  | { type: "dumpMachine-event"; event: DumpMsg }
  | { type: "success-event"; event: string }
  | { type: "failure-event"; event: any }; // eslint-disable-line @typescript-eslint/no-explicit-any

export type ResponseMsgType =
  | ResponseMessageWithType<CompilerRespMsg>
  | EventMessageWithType<CompilerEventMsg>;

function makeRequestMessage(
  type: string,
  /* eslint-disable @typescript-eslint/no-explicit-any */
  args: any[]
): { msg: CompilerReqMsg; longRunning: boolean } | null {
  let msg: CompilerReqMsg;
  let longRunning = false;
  switch (type) {
    case "checkCode":
      msg = { type: "checkCode", args: args as [string] };
      break;
    case "getHir":
      msg = { type: "getHir", args: args as [string] };
      break;
    case "getCompletions":
      msg = { type: "getCompletions", args: args as [] };
      break;
    case "run":
      // run and runKata can take a long time, so set state to busy
      longRunning = true;
      msg = {
        type: "run",
        args: args as [string, string, number],
      };
      break;
    case "runKata":
      longRunning = true;
      msg = {
        type: "runKata",
        args: args as [string, string],
      };
      break;
    default:
      log.error("message type is invalid");
      return null;
  }

  log.debug("request message: " + JSON.stringify(msg));
  return { msg, longRunning };
}

function makePassThroughEvent(msg: CompilerEventMsg): QscEvents | null {
  const msgType = msg.type;
  switch (msgType) {
    // Event type messages don't complete the request
    case "message-event": {
      return makeCompilerEvent("Message", msg.event.message) as QscEvents;
    }
    case "dumpMachine-event": {
      return makeCompilerEvent("DumpMachine", msg.event.state) as QscEvents;
    }
    case "failure-event": {
      const failEvent = makeCompilerEvent("Result", {
        success: false,
        value: msg.event,
      });
      return failEvent as QscEvents;
    }
    case "success-event": {
      const successEvent = makeCompilerEvent("Result", {
        success: true,
        value: msg.event,
      });
      return successEvent as QscEvents;
    }

    default:
      log.never(msg);
      return null;
  }
}

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

// this should be combined with "makeRequestMsg" pretty sure
const methodToRequestMessage = {
  checkCode(code: string) {
    return { request: "checkCode", args: [code] };
  },
  getHir(code: string) {
    return { request: "getHir", args: [code] };
  },
  getCompletions() {
    return { request: "getCompletions", args: [] };
  },
  run(
    code: string,
    expr: string,
    shots: number,
    uiEventTarget: IQscEventTarget
  ) {
    return { request: "run", args: [code, expr, shots], uiEventTarget };
  },
  runKata(
    user_code: string,
    verify_code: string,
    uiEventTarget: IQscEventTarget
  ) {
    return {
      request: "runKata",
      args: [user_code, verify_code],
      uiEventTarget,
    };
  },
} as {
  [method in keyof Exclude<
    ICompiler,
    { onstatechange: ((state: ServiceState) => void) | null }
  >]: (...args: any[]) => {
    request: string;
    args: any[];
    uiEventTarget?: IQscEventTarget;
  };
};

export function handleMessageInWorker(
  data: CompilerReqMsg,
  compiler: ICompiler,
  postMessage: (msg: CompilerRespMsg) => void,
  serviceEventTarget: IQscEventTarget
) {
  return invokeWorkerMethod(
    data as any,
    compiler,
    postMessage as any,
    serviceEventTarget
  );
}

const eventMap = {
  Message: (ev: QscEvent<"Message">) => ({
    type: "message-event",
    event: { type: "Message", message: ev.detail },
  }),
  DumpMachine: (ev: QscEvent<"DumpMachine">) => ({
    type: "dumpMachine-event",
    event: { type: "DumpMachine", state: ev.detail },
  }),
  Result: (ev: QscEvent<"Result">) =>
    ev.detail.success
      ? {
          type: "success-event",
          event: ev.detail.value,
        }
      : {
          type: "failure-event",
          event: ev.detail.value,
        },
} as {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  [eventName in QscEvents["type"]]: (ev: any) => CompilerEventMsg;
};

export function getWorkerEventHandlers(
  postMessage: (msg: CompilerEventMsg) => void
): IQscEventTarget {
  return getWorkerEventHandlersGeneric<
    QscEvents,
    IQscEventTarget,
    CompilerEventMsg
  >(postMessage, eventMap);
}

export function createCompilerProxy(
  postMessage: (msg: CompilerReqMsg) => void,
  setMsgHandler: (handler: (e: ResponseMsgType) => void) => void,
  terminator: () => void
) {
  return createWorkerProxy<
    CompilerReqMsg,
    CompilerRespMsg,
    QscEvents,
    CompilerEventMsg,
    IQscEventTarget,
    ICompilerWorker
  >(
    postMessage,
    setMsgHandler,
    terminator,
    makeRequestMessage,
    makePassThroughEvent,
    makeResult,
    methodToRequestMessage
  );
}
// end
