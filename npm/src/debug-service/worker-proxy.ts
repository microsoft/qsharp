// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { QscEventData } from "../compiler/events.js";
import {
  EventMessage,
  MethodMap,
  RequestMessage,
  ResponseMessage,
  createDispatcher,
  createProxy,
} from "../worker-proxy.js";
import { IDebugService } from "./debug-service.js";

const requests: MethodMap<IDebugService> = {
  loadSource: "request",
  getBreakpoints: "request",
  getLocalVariables: "request",
  captureQuantumState: "request",
  getStackFrames: "request",
  evalContinue: "requestWithProgress",
  evalNext: "requestWithProgress",
  evalStepIn: "requestWithProgress",
  evalStepOut: "requestWithProgress",
  dispose: "request",
};

const events: QscEventData["type"][] = ["DumpMachine", "Message", "Result"];

export function createDebugServiceDispatcher(
  postMessage: (
    msg: ResponseMessage<IDebugService> | EventMessage<QscEventData>,
  ) => void,
  service: IDebugService,
) {
  return createDispatcher<IDebugService, QscEventData>(
    postMessage,
    service,
    requests,
    events,
  );
}

export function createDebugServiceProxy(
  postMessage: (msg: RequestMessage<IDebugService>) => void,
  terminator: () => void,
) {
  return createProxy<IDebugService, QscEventData>(
    postMessage,
    terminator,
    requests,
  );
}
