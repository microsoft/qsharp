// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { log } from "./log.js";
import { ICompletionList, IHover, IDefinition } from "../lib/web/qsc_wasm.js";
import { DiagnosticsMsg, DumpMsg, MessageMsg } from "./common.js";
import { CompilerState, ICompiler, ICompilerWorker } from "./compiler.js";
import { CancellationToken } from "./cancellation.js";
import { IQscEventTarget, QscEventTarget, makeEvent } from "./events.js";

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
  evtTarget?: IQscEventTarget;
  cancellationToken?: CancellationToken;
};
/* eslint-enable @typescript-eslint/no-explicit-any */

/**
 * @param postMessage A function to post messages to the worker
 * @param setMsgHandler A function to call to set the callback for messages received from the worker
 * @param terminator A function to call to tear down the worker thread
 * @returns
 */
export function createWorkerProxy(
  postMessage: (msg: CompilerReqMsg) => void,
  setMsgHandler: (handler: (e: ResponseMsgType) => void) => void,
  terminator: () => void,
  evtTarget: IQscEventTarget
): ICompilerWorker {
  const queue: RequestState[] = [];
  let curr: RequestState | undefined;
  let state: CompilerState = "idle";

  function setState(newState: CompilerState) {
    if (state === newState) return;
    state = newState;
    if (proxy.onstatechange) proxy.onstatechange(state);
  }

  function queueRequest(
    type: string,
    args: any[], // eslint-disable-line @typescript-eslint/no-explicit-any
    cancellationToken?: CancellationToken
  ): Promise<RespResultTypes> {
    return new Promise((resolve, reject) => {
      queue.push({ type, args, resolve, reject, cancellationToken });

      // If nothing was running when this got added, kick off processing
      if (queue.length === 1) doNextRequest();
    });
  }

  function doNextRequest() {
    if (curr) return;

    while ((curr = queue.shift())) {
      // eslint-disable-line no-cond-assign
      if (curr.cancellationToken?.isCancellationRequested) {
        curr.reject("cancelled");
        continue;
      } else {
        break;
      }
    }
    if (!curr) {
      // Nothing else queued, signal that we're now idle and exit.
      log.debug("Worker queue is empty");
      setState("idle");
      return;
    }

    let msg: CompilerReqMsg | null = null;
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
      case "run":
        // run and runKata can take a long time, so set state to busy
        setState("busy");
        msg = {
          type: "run",
          code: curr.args[0],
          expr: curr.args[1],
          shots: curr.args[2],
        };
        break;
      case "runKata":
        setState("busy");
        msg = {
          type: "runKata",
          user_code: curr.args[0],
          verify_code: curr.args[1],
        };
        break;
      default:
        log.error("message type is invalid");
        return;
    }
    if (log.getLogLevel() >= 4) log.debug("Posting message to worker: %o", msg);
    postMessage(msg);
  }

  function onMsgFromWorker(msg: CompilerRespMsg | CompilerEventMsg) {
    if (!curr) {
      log.error("No active request when message received: %o", msg);
      return;
    }
    if (log.getLogLevel() >= 4)
      log.debug("Received message from worker: %o", msg);

    const msgType = msg.type;
    switch (msgType) {
      // Event type messages don't complete the request
      case "message-event": {
        const msgEvent = makeEvent("Message", msg.event.message);
        evtTarget.dispatchEvent(msgEvent);
        return;
      }
      case "dumpMachine-event": {
        const dmpEvent = makeEvent("DumpMachine", msg.event.state);
        evtTarget.dispatchEvent(dmpEvent);
        return;
      }
      case "failure-event": {
        const failEvent = makeEvent("Result", {
          success: false,
          value: msg.event,
        });
        evtTarget.dispatchEvent(failEvent);
        return;
      }
      case "diagnostics-event": {
        const diagEvent = makeEvent("diagnostics", msg.event.diagnostics);
        evtTarget.dispatchEvent(diagEvent);
        return;
      }
      case "success-event": {
        const successEvent = makeEvent("Result", {
          success: true,
          value: msg.event,
        });
        evtTarget.dispatchEvent(successEvent);
        return;
      }

      // Response type messages. Resolve and complete this request.
      case "updateDocument-result":
      case "closeDocument-result":
      case "getCompletions-result":
      case "getHover-result":
      case "getDefinition-result":
      case "run-result":
      case "runKata-result":
        curr.resolve(msg.result);
        curr = undefined;
        doNextRequest();
        return;

      case "error-result":
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

  const proxy: ICompilerWorker = {
    updateDocument(uri, version, code) {
      return queueRequest("updateDocument", [uri, version, code]);
    },
    closeDocument(uri) {
      return queueRequest("closeDocument", [uri]);
    },
    getCompletions(documentUri, code, offset) {
      return queueRequest("getCompletions", [documentUri, code, offset]);
    },
    getHover(documentUri, code, offset) {
      return queueRequest("getHover", [documentUri, code, offset]);
    },
    getDefinition(documentUri, code, offset) {
      return queueRequest("getDefinition", [documentUri, code, offset]);
    },
    run(code, expr, shots) {
      return queueRequest("run", [code, expr, shots]);
    },
    runKata(user_code, verify_code) {
      return queueRequest("runKata", [user_code, verify_code]);
    },
    onstatechange: null,
    // Kill the worker without a chance to shutdown. May be needed if it is not responding.
    terminate: () => {
      log.info("Terminating the worker");
      if (curr) {
        log.debug("Terminating running worker item of type: %s", curr.type);
        curr.reject("terminated");
      }
      // Reject any outstanding items
      while (queue.length) {
        const item = queue.shift();
        log.debug("Terminating outstanding work item of type: %s", item?.type);
        item?.reject("terminated");
      }
      terminator();
    },
  };
  return proxy;
}

// Used by the worker to handle compiler events by posting a message back to the client
export function getWorkerEventHandlers(
  postMessage: (msg: CompilerEventMsg) => void
): QscEventTarget {
  log.debug("Constructing WorkerEventHandler");

  const logAndPost = (msg: CompilerEventMsg) => {
    log.debug("Sending event message from worker: %o", msg);
    postMessage(msg);
  };
  const evtTarget = new QscEventTarget(false);

  evtTarget.addEventListener("Message", (ev) => {
    logAndPost({
      type: "message-event",
      event: { type: "Message", message: ev.detail },
    });
  });

  evtTarget.addEventListener("DumpMachine", (ev) => {
    logAndPost({
      type: "dumpMachine-event",
      event: { type: "DumpMachine", state: ev.detail },
    });
  });

  evtTarget.addEventListener("diagnostics", (ev) => {
    logAndPost({
      type: "diagnostics-event",
      event: { type: "diagnostics", diagnostics: ev.detail },
    });
  });

  evtTarget.addEventListener("Result", (ev) => {
    if (ev.detail.success) {
      logAndPost({ type: "success-event", event: ev.detail.value });
    } else {
      logAndPost({ type: "failure-event", event: ev.detail.value });
    }
  });

  return evtTarget;
}

// This is the main function that the worker thread should delegate incoming messages to
export function handleMessageInWorker(
  data: CompilerReqMsg,
  compiler: ICompiler,
  postMessage: (msg: CompilerRespMsg) => void
) {
  log.debug("Handling message in worker: %o", data);
  const logIntercepter = (msg: CompilerRespMsg) => {
    log.debug("Sending response message from worker: %o", msg);
    postMessage(msg);
  };

  try {
    const msgType = data.type;
    switch (msgType) {
      case "updateDocument":
        compiler
          .updateDocument(data.uri, data.version, data.code)
          .then(() =>
            logIntercepter({ type: "updateDocument-result", result: undefined })
          );
        break;
      case "closeDocument":
        compiler
          .closeDocument(data.uri)
          .then(() =>
            logIntercepter({ type: "closeDocument-result", result: undefined })
          );
        break;
      case "getCompletions":
        compiler
          .getCompletions(data.documentUri, data.code, data.offset)
          .then((result) =>
            logIntercepter({ type: "getCompletions-result", result })
          );
        break;
      case "getHover":
        compiler
          .getHover(data.documentUri, data.code, data.offset)
          .then((result) =>
            logIntercepter({ type: "getHover-result", result })
          );
        break;
      case "getDefinition":
        compiler
          .getDefinition(data.documentUri, data.code, data.offset)
          .then((result) =>
            logIntercepter({ type: "getDefinition-result", result })
          );
        break;
      case "run":
        compiler
          .run(data.code, data.expr, data.shots)
          // 'run' can throw on compiler errors, which should be reported as events for
          // each 'shot', so just resolve as run 'complete' regardless.
          .finally(() =>
            logIntercepter({ type: "run-result", result: undefined })
          );
        break;
      case "runKata":
        compiler
          .runKata(data.user_code, data.verify_code)
          .then((result) => logIntercepter({ type: "runKata-result", result }))
          // It shouldn't throw, but just in case there's a runtime or compiler failure
          .catch(() =>
            logIntercepter({ type: "runKata-result", result: false })
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

export type CompilerReqMsg =
  | { type: "updateDocument"; uri: string; version: number; code: string }
  | { type: "closeDocument"; uri: string }
  | {
      type: "getCompletions";
      documentUri: string;
      code: string;
      offset: number;
    }
  | { type: "getHover"; documentUri: string; code: string; offset: number }
  | { type: "getDefinition"; documentUri: string; code: string; offset: number }
  | { type: "run"; code: string; expr: string; shots: number }
  | { type: "runKata"; user_code: string; verify_code: string };

type CompilerRespMsg =
  | { type: "updateDocument-result"; result: void }
  | { type: "closeDocument-result"; result: void }
  | { type: "getCompletions-result"; result: ICompletionList }
  | {
      type: "getHover-result";
      result: IHover | null;
    }
  | {
      type: "getDefinition-result";
      result: IDefinition;
    }
  | { type: "run-result"; result: void }
  | { type: "runKata-result"; result: boolean }
  | { type: "error-result"; result: any }; // eslint-disable-line @typescript-eslint/no-explicit-any

// Get the possible 'result' types from a compiler response
type ExtractResult<T> = T extends { result: infer R } ? R : never;
type RespResultTypes = ExtractResult<CompilerRespMsg>;

type CompilerEventMsg =
  | { type: "message-event"; event: MessageMsg }
  | { type: "dumpMachine-event"; event: DumpMsg }
  | { type: "success-event"; event: string }
  | { type: "failure-event"; event: any } // eslint-disable-line @typescript-eslint/no-explicit-any
  | { type: "diagnostics-event"; event: DiagnosticsMsg };

export type ResponseMsgType = CompilerRespMsg | CompilerEventMsg;
