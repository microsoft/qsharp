// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { log } from "../log.js";
import {
  ICompletionList,
  IDefinition,
  IHover,
} from "../../lib/web/qsc_wasm.js";
import { ILanguageService } from "./language-service.js";
import {
  Diagnostics,
  ILanguageServiceEventTarget,
  LanguageServiceEventTarget,
  makeEvent,
} from "./events.js";

/*
The WorkerProxy works by queuing up requests to send over to the Worker, only
ever having one in flight at a time. By queuing on the caller side, this allows
for cancellation (it checks if a request is cancelled before sending to the worker).

The queue contains an entry for each request with the data to send, the promise
to resolve, the event handler, and the cancellation token. When a request completes
the next one (if present) is fetched from the queue. If it is marked as cancelled,
it is resolved immediately, else it is marked as the current request and the command
sent to the worker. As events occurs on the current request the event handler is
invoked. When the response is received this is used to resolve the promise and
complete the request.
*/

/* eslint-disable @typescript-eslint/no-explicit-any */
type RequestState = {
  type: string;
  args: any[];
  resolve: (val: any) => void;
  reject: (err: any) => void;
  evtTarget?: ILanguageServiceEventTarget;
};
/* eslint-enable @typescript-eslint/no-explicit-any */

/**
 * @param postMessage A function to post messages to the worker
 * @param setMsgHandler A function to call to set the callback for messages received from the worker
 * @returns
 */
export function createWorkerProxy(
  postMessage: (msg: LanguageServiceReqMsg) => void,
  setMsgHandler: (handler: (e: ResponseMsgType) => void) => void,
  evtTarget: ILanguageServiceEventTarget
): ILanguageService {
  const queue: RequestState[] = [];
  let curr: RequestState | undefined;

  function queueRequest(
    type: string,
    args: any[] // eslint-disable-line @typescript-eslint/no-explicit-any
  ): Promise<RespResultTypes> {
    return new Promise((resolve, reject) => {
      queue.push({ type, args, resolve, reject });

      // If nothing was running when this got added, kick off processing
      if (queue.length === 1) doNextRequest();
    });
  }

  function doNextRequest() {
    if (curr) return;

    curr = queue.shift();

    if (!curr) {
      // Nothing else queued, signal that we're now idle and exit.
      log.debug("Worker queue is empty");
      return;
    }

    let msg: LanguageServiceReqMsg | null = null;
    switch (curr.type) {
      case "updateDocument":
        msg = {
          type: "updateDocument",
          uri: curr.args[0],
          version: curr.args[1],
          code: curr.args[2],
        };
        break;
      case "closeDocument":
        msg = {
          type: "closeDocument",
          uri: curr.args[0],
        };
        break;
      case "getCompletions":
        msg = {
          type: "getCompletions",
          documentUri: curr.args[0],
          code: curr.args[1],
          offset: curr.args[2],
        };
        break;
      case "getHover":
        msg = {
          type: "getHover",
          documentUri: curr.args[0],
          code: curr.args[1],
          offset: curr.args[2],
        };
        break;
      case "getDefinition":
        msg = {
          type: "getDefinition",
          documentUri: curr.args[0],
          code: curr.args[1],
          offset: curr.args[2],
        };
        break;
      default:
        log.error("message type is invalid");
        return;
    }
    if (log.getLogLevel() >= 4) log.debug("Posting message to worker: %o", msg);
    postMessage(msg);
  }

  function onMsgFromWorker(
    msg: LanguageServiceRespMsg | LanguageServiceEventMsg
  ) {
    if (log.getLogLevel() >= 4)
      log.debug("Received message from worker: %o", msg);

    const msgType = msg.type;
    switch (msgType) {
      // Event type messages don't complete the request
      case "diagnostics-event": {
        const diagEvent = makeEvent("diagnostics", msg.event.diagnostics);
        evtTarget.dispatchEvent(diagEvent);
        return;
      }
      // Response type messages. Resolve and complete this request.
      case "updateDocument-result":
      case "closeDocument-result":
      case "getCompletions-result":
      case "getHover-result":
      case "getDefinition-result":
        if (!curr) {
          log.error("No active request when message received: %o", msg);
          return;
        }
        curr.resolve(msg.result);
        curr = undefined;
        doNextRequest();
        return;

      case "error-result":
        if (!curr) {
          log.error("No active request when message received: %o", msg);
          return;
        }
        // Something unexpected failed the request. Reject and move on.
        curr.reject(msg.result);
        curr = undefined;
        doNextRequest();
        return;

      default:
        log.never(msg);
        return;
    }
  }

  setMsgHandler(onMsgFromWorker);

  const proxy: ILanguageService = {
    updateDocument(uri, version, code) {
      return queueRequest("updateDocument", [uri, version, code]);
    },
    closeDocument(uri) {
      return queueRequest("closeDocument", [uri]);
    },
    getCompletions(documentUri, offset) {
      return queueRequest("getCompletions", [documentUri, offset]);
    },
    getHover(documentUri, offset) {
      return queueRequest("getHover", [documentUri, offset]);
    },
    getDefinition(documentUri, offset) {
      return queueRequest("getDefinition", [documentUri, offset]);
    },
  };
  return proxy;
}

// Used by the worker to handle language service events by posting a message back to the client
export function getWorkerEventHandlers(
  postMessage: (msg: LanguageServiceEventMsg) => void
): ILanguageServiceEventTarget {
  log.debug("Constructing WorkerEventHandler");

  const logAndPost = (msg: LanguageServiceEventMsg) => {
    log.debug("Sending event message from worker: %o", msg);
    postMessage(msg);
  };
  const evtTarget = new LanguageServiceEventTarget();

  evtTarget.addEventListener("diagnostics", (ev) => {
    logAndPost({
      type: "diagnostics-event",
      event: { type: "diagnostics", diagnostics: ev.detail },
    });
  });

  return evtTarget;
}

// This is the main function that the worker thread should delegate incoming messages to
export function handleMessageInWorker(
  data: LanguageServiceReqMsg,
  languageService: ILanguageService,
  postMessage: (msg: LanguageServiceRespMsg) => void
) {
  log.debug("Handling message in worker: %o", data);
  const logIntercepter = (msg: LanguageServiceRespMsg) => {
    log.debug("Sending response message from worker: %o", msg);
    postMessage(msg);
  };

  try {
    const msgType = data.type;
    switch (msgType) {
      case "updateDocument":
        languageService
          .updateDocument(data.uri, data.version, data.code)
          .then(() =>
            logIntercepter({ type: "updateDocument-result", result: undefined })
          );
        break;
      case "closeDocument":
        languageService
          .closeDocument(data.uri)
          .then(() =>
            logIntercepter({ type: "closeDocument-result", result: undefined })
          );
        break;
      case "getCompletions":
        languageService
          .getCompletions(data.documentUri, data.offset)
          .then((result) =>
            logIntercepter({ type: "getCompletions-result", result })
          );
        break;
      case "getHover":
        languageService
          .getHover(data.documentUri, data.offset)
          .then((result) =>
            logIntercepter({ type: "getHover-result", result })
          );
        break;
      case "getDefinition":
        languageService
          .getDefinition(data.documentUri, data.offset)
          .then((result) =>
            logIntercepter({ type: "getDefinition-result", result })
          );
        break;
      default:
        log.never(msgType);
    }
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
  } catch (err: any) {
    // If this happens then the wasm code likely threw an exception/paniced rather than
    // completing gracefully and fullfilling the promise. Communicate to the client
    // that there was an error and it should reject the current request

    logIntercepter({ type: "error-result", result: err });
  }
}

export type LanguageServiceReqMsg =
  | { type: "updateDocument"; uri: string; version: number; code: string }
  | { type: "closeDocument"; uri: string }
  | {
      type: "getCompletions";
      documentUri: string;
      code: string;
      offset: number;
    }
  | { type: "getHover"; documentUri: string; code: string; offset: number }
  | {
      type: "getDefinition";
      documentUri: string;
      code: string;
      offset: number;
    };

type LanguageServiceRespMsg =
  | { type: "updateDocument-result"; result: void }
  | { type: "closeDocument-result"; result: void }
  | { type: "getCompletions-result"; result: ICompletionList }
  | {
      type: "getHover-result";
      result: IHover | null;
    }
  | {
      type: "getDefinition-result";
      result: IDefinition | null;
    }
  | { type: "error-result"; result: any }; // eslint-disable-line @typescript-eslint/no-explicit-any

// Get the possible 'result' types from a language service response
type ExtractResult<T> = T extends { result: infer R } ? R : never;
type RespResultTypes = ExtractResult<LanguageServiceRespMsg>;

type LanguageServiceEventMsg = {
  type: "diagnostics-event";
  event: {
    type: "diagnostics";
    diagnostics: Diagnostics;
  };
};

export type ResponseMsgType = LanguageServiceRespMsg | LanguageServiceEventMsg;
