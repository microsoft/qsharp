// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { log } from "./log.js";
import { ICompletionList } from "../lib/web/qsc_wasm.js";
import { DumpMsg, MessageMsg, VSDiagnostic } from "./common.js";
import { ICompiler, ICompilerWorker } from "./compiler.js";
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
  terminator: () => void
): ICompilerWorker {
  const queue: RequestState[] = [];
  let curr: RequestState | undefined;

  function queueRequest(
    type: string,
    args: any[], // eslint-disable-line @typescript-eslint/no-explicit-any
    evtTarget?: IQscEventTarget,
    cancellationToken?: CancellationToken
  ): Promise<RespResultTypes> {
    return new Promise((resolve, reject) => {
      queue.push({ type, args, resolve, reject, evtTarget, cancellationToken });

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
    if (!curr) return;

    let msg: CompilerReqMsg | null = null;
    switch (curr.type) {
      case "checkCode":
        msg = { type: "checkCode", code: curr.args[0] };
        break;
      case "getCompletions":
        msg = { type: "getCompletions" };
        break;
      case "run":
        msg = {
          type: "run",
          code: curr.args[0],
          expr: curr.args[1],
          shots: curr.args[2],
        };
        break;
      case "runKata":
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
        curr.evtTarget?.dispatchEvent(msgEvent);
        return;
      }
      case "dumpMachine-event": {
        const dmpEvent = makeEvent("DumpMachine", msg.event.state);
        curr.evtTarget?.dispatchEvent(dmpEvent);
        return;
      }
      case "failure-event": {
        const failEvent = makeEvent("Result", {
          success: false,
          value: msg.event,
        });
        curr.evtTarget?.dispatchEvent(failEvent);
        return;
      }
      case "success-event": {
        const successEvent = makeEvent("Result", {
          success: true,
          value: msg.event,
        });
        curr.evtTarget?.dispatchEvent(successEvent);
        return;
      }

      // Response type messages. Resolve and complete this request.
      case "checkCode-result":
      case "getCompletions-result":
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
    checkCode(code) {
      return queueRequest("checkCode", [code]);
    },
    getCompletions() {
      return queueRequest("getCompletions", []);
    },
    run(code, expr, shots, evtHandler) {
      return queueRequest("run", [code, expr, shots], evtHandler);
    },
    runKata(user_code, verify_code, evtHandler) {
      return queueRequest("runKata", [user_code, verify_code], evtHandler);
    },
    // Kill the worker without a chance to shutdown. May be needed if it is not responding.
    terminate: () => {
      log.info("Terminating the worker");
      terminator();
    },
  };
  return proxy;
}

// Used by the worker to handle compiler events by posting a message back to the client
export function getWorkerEventHandlers(
  postMessage: (msg: CompilerEventMsg) => void
): IQscEventTarget {
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
  postMessage: (msg: CompilerRespMsg) => void,
  evtTarget: IQscEventTarget
) {
  log.debug("Handling message in worker: %o", data);
  const logIntercepter = (msg: CompilerRespMsg) => {
    log.debug("Sending response message from worker: %o", msg);
    postMessage(msg);
  };

  try {
    const msgType = data.type;
    switch (msgType) {
      case "checkCode":
        compiler
          .checkCode(data.code)
          .then((result) =>
            logIntercepter({ type: "checkCode-result", result })
          );
        break;
      case "getCompletions":
        compiler
          .getCompletions()
          .then((result) =>
            logIntercepter({ type: "getCompletions-result", result })
          );
        break;
      case "run":
        compiler
          .run(data.code, data.expr, data.shots, evtTarget)
          // 'run' can throw on compiler errors, which should be reported as events for
          // each 'shot', so just resolve as run 'complete' regardless.
          .finally(() =>
            logIntercepter({ type: "run-result", result: undefined })
          );
        break;
      case "runKata":
        compiler
          .runKata(data.user_code, data.verify_code, evtTarget)
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
  | { type: "checkCode"; code: string }
  | { type: "getCompletions" }
  | { type: "run"; code: string; expr: string; shots: number }
  | { type: "runKata"; user_code: string; verify_code: string };

type CompilerRespMsg =
  | { type: "checkCode-result"; result: VSDiagnostic[] }
  | { type: "getCompletions-result"; result: ICompletionList }
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
  | { type: "failure-event"; event: any }; // eslint-disable-line @typescript-eslint/no-explicit-any

export type ResponseMsgType = CompilerRespMsg | CompilerEventMsg;
