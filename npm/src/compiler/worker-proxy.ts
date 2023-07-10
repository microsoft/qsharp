// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import {
  EventMessage,
  MethodMap,
  RequestMessage,
  ResponseMessage,
  createDispatcher,
  createProxy,
} from "../worker-proxy.js";
import { ICompiler } from "./compiler.js";
import { QscEventData } from "./events.js";

const requests: MethodMap<ICompiler> = {
  checkCode: "request",
  getHir: "request",
  run: "requestWithProgress",
  checkExerciseSolution: "requestWithProgress",
};

const events: QscEventData["type"][] = ["DumpMachine", "Message", "Result"];

export function createCompilerDispatcher(
  postMessage: (
    msg: ResponseMessage<ICompiler> | EventMessage<QscEventData>
  ) => void,
  service: ICompiler
) {
  return createDispatcher<ICompiler, QscEventData>(
    postMessage,
    service,
    requests,
    events
  );
}

export function createCompilerProxy(
  postMessage: (msg: RequestMessage<ICompiler>) => void,
  terminator: () => void
) {
  return createProxy<ICompiler, QscEventData>(
    postMessage,
    terminator,
    requests
  );
}
