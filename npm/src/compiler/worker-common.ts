// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { EventMessageWithType, RequestMessage, ResponseMessage, ResponseMessageWithType, createWorkerProxy, getWorkerEventHandlersGeneric } from "../worker-common.js";
import { ICompiler, ICompilerWorker } from "./compiler.js";
import {
  QscEventData,
  type QscEvents,
} from "./events.js";

export type ICompilerMethodsOnly = Omit<ICompiler, "onstatechange">;
const requests: { [M in keyof ICompilerMethodsOnly] : { longRunning: boolean }  }= {
  checkCode: { longRunning: false },
  getHir: { longRunning: false },
  getCompletions: { longRunning: false },
  run: { longRunning: true },
  runKata: { longRunning: true }
}

export function getWorkerEventHandlers(
  postMessage: (msg: QscEventData) => void
) {
  return getWorkerEventHandlersGeneric<
  QscEventData
  >(["DumpMachine", "Message", "Result"], postMessage);
}

export type WorkerToMainMessage =
  | ResponseMessageWithType<ResponseMessage<ICompilerMethodsOnly>>
  | EventMessageWithType<QscEventData>;

export function createCompilerProxy(
  postMessage: (msg: RequestMessage<ICompilerMethodsOnly>) => void,
  setMsgHandler: (handler: (e: WorkerToMainMessage) => void) => void,
  terminator: () => void
) {
  return createWorkerProxy<
  RequestMessage<ICompilerMethodsOnly>,
  ResponseMessage<ICompilerMethodsOnly>,
    QscEvents,
    QscEventData,
    ICompilerWorker
  >(
    postMessage,
    setMsgHandler,
    terminator,
    requests
  );
}
