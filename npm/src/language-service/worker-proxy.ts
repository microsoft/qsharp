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
import { ILanguageService, LanguageServiceEvent } from "./language-service.js";

const requests: MethodMap<ILanguageService> = {
  updateConfiguration: "request",
  updateDocument: "request",
  closeDocument: "request",
  getCompletions: "request",
  getHover: "request",
  getDefinition: "request",
  getSignatureHelp: "request",
  getRename: "request",
  prepareRename: "request",
  dispose: "request",
  addEventListener: "addEventListener",
  removeEventListener: "removeEventListener",
};

const events: LanguageServiceEvent["type"][] = ["diagnostics"];

export function createLanguageServiceDispatcher(
  postMessage: (
    msg: ResponseMessage<ILanguageService> | EventMessage<LanguageServiceEvent>,
  ) => void,
  service: ILanguageService,
) {
  return createDispatcher<ILanguageService, LanguageServiceEvent>(
    postMessage,
    service,
    requests,
    events,
  );
}

export function createLanguageServiceProxy(
  postMessage: (msg: RequestMessage<ILanguageService>) => void,
  terminator: () => void,
) {
  return createProxy<ILanguageService, LanguageServiceEvent>(
    postMessage,
    terminator,
    requests,
  );
}
