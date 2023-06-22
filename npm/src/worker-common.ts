// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { log } from "./log.js";
import { CancellationToken } from "./cancellation.js";

// These will be parameters
type IServiceWorker = {
  onstatechange: ((state: ServiceState) => void) | null;
  terminate: () => void;
};

type CoreType<TServiceWorker> = Exclude<
  TServiceWorker,
  {
    onstatechange: ((state: ServiceState) => void) | null;
    terminate: () => void;
  }
>;

export interface IServiceEventTarget<TEvents extends { type: string }> {
  addEventListener<T extends TEvents["type"]>(
    type: T,
    listener: (event: Extract<TEvents, { type: T }>) => void
  ): void;

  removeEventListener<T extends TEvents["type"]>(
    type: T,
    listener: (event: Extract<TEvents, { type: T }>) => void
  ): void;

  dispatchEvent(event: TEvents): boolean;
}

// function makeEvent<E extends { type: string, detail: unknown }>(
//   type: E["type"],
//   detail: E["detail"]
// ): E {
//   return makeCompilerEvent<QscEvents>(type, detail);
// }

/* eslint-disable @typescript-eslint/no-explicit-any */
type RequestState<TEvents extends { type: string }> = {
  type: string;
  args: any[];
  resolve: (val: any) => void;
  reject: (err: any) => void;
  uiEventTarget?: IServiceEventTarget<TEvents>;
  cancellationToken?: CancellationToken;
};
/* eslint-enable @typescript-eslint/no-explicit-any */

// Get the possible 'result' types from a compiler response
type ExtractResult<TRespMsg> = TRespMsg extends { result: infer R } ? R : never;
export type ResponseMessageWithType<TRespMsg> = {
  messageType: "response";
} & TRespMsg;
export type EventMessageWithType<TEventMsg> = {
  messageType: "event";
} & TEventMsg;
// end parameters

// Real types (not parameters)
export type ServiceState = "idle" | "busy";
// end types

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

/**
 * @param postMessage A function to post messages to the worker
 * @param setMsgHandler A function to call to set the callback for messages received from the worker
 * @param terminator A function to call to tear down the worker thread
 * @returns
 */
export function createWorkerProxy<
  TServiceReqMsg extends { type: string; args: unknown[] },
  TServiceRespMsg extends { type: string; result: unknown },
  TServiceEvents extends Event & { type: string },
  TServiceEventMsg extends { type: string; event: unknown },
  TEventTarget extends IServiceEventTarget<TServiceEvents>,
  TServiceWorker extends IServiceWorker
>(
  postMessage: (msg: TServiceReqMsg) => void,
  setMsgHandler: (
    handler: (
      e:
        | ResponseMessageWithType<TServiceRespMsg>
        | EventMessageWithType<TServiceEventMsg>
    ) => void
  ) => void,
  terminator: () => void,
  makeRequestMessage: (
    type: string,
    // TODO: I should be able to make this strongly typed if it's not an array but an object
    args: any[] // eslint-disable-line @typescript-eslint/no-explicit-any
  ) => { msg: TServiceReqMsg; longRunning: boolean } | null,
  makePassThroughEvent: (msg: TServiceEventMsg) => TServiceEvents | null,
  makeResult: (
    msg: TServiceRespMsg
  ) => { success: boolean; data: TServiceRespMsg["result"] } | null,
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  methodToRequestMessage: {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    [method in keyof CoreType<TServiceWorker>]: (...args: any[]) => {
      request: string;
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      args: any[];
      uiEventTarget?: TEventTarget;
    };
  }
): TServiceWorker {
  const queue: RequestState<TServiceEvents>[] = [];
  let curr: RequestState<TServiceEvents> | undefined;
  let state: ServiceState = "idle";

  function setState(newState: ServiceState) {
    if (state === newState) return;
    state = newState;
    if (proxy.onstatechange) proxy.onstatechange(state);
  }

  function queueRequest(
    type: string,
    args: any[], // eslint-disable-line @typescript-eslint/no-explicit-any
    uiEventTarget?: TEventTarget,
    cancellationToken?: CancellationToken
  ): Promise<ExtractResult<TServiceRespMsg>> {
    return new Promise((resolve, reject) => {
      queue.push({
        type,
        args,
        resolve,
        reject,
        uiEventTarget,
        cancellationToken,
      });

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
    console.log(`handling request ${curr.type}`);

    const msg = makeRequestMessage(curr.type, curr.args);
    if (!msg) {
      return;
    }
    if (msg.longRunning) {
      setState("busy");
    }

    if (log.getLogLevel() >= 4) log.debug("Posting message to worker: %o", msg);
    postMessage(msg.msg);
  }

  function onMsgFromWorker(
    msg:
      | ResponseMessageWithType<TServiceRespMsg>
      | EventMessageWithType<TServiceEventMsg>
  ) {
    if (!curr) {
      log.error("No active request when message received: %o", msg);
      return;
    }
    log.debug("Received message from worker: %o", msg);

    if (msg.messageType === "event") {
      const event = makePassThroughEvent(msg);
      if (!event) return;

      log.debug("Posting event: %o", msg);
      curr.uiEventTarget?.dispatchEvent(event);
    } else if (msg.messageType === "response") {
      const result = makeResult(msg);
      if (!result) return;
      if (result.success) {
        curr.resolve(result.data);
        curr = undefined;
        doNextRequest();
      } else {
        curr.reject(result.data);
        curr = undefined;
        doNextRequest();
      }
      return;
    }
  }

  setMsgHandler(onMsgFromWorker);

  // @ts-expect-error let's just power through this TypeScript
  const proxy: TServiceWorker = {};

  for (const methodName of Object.keys(methodToRequestMessage)) {
    // @ts-expect-error let's just power through this TypeScript
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    proxy[methodName] = (...args: any[]) => {
      console.log(`method ${methodName} called`);
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const data = methodToRequestMessage[
        methodName as keyof CoreType<TServiceWorker>
      ](...args);
      log.debug(
        `about to queue a request with args ${JSON.stringify(data.args)}`
      );
      return queueRequest(data.request, data.args, data.uiEventTarget);
    };
  }
  proxy.onstatechange = null;
  proxy.terminate = () => {
    // Kill the worker without a chance to shutdown. May be needed if it is not responding.
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
  };

  console.log(`returning proxy`);
  return proxy;
}

// Used by the worker to handle service events by posting a message back to the client
export function getWorkerEventHandlersGeneric<
  TServiceEvents extends Event & { type: string },
  TEventTarget extends IServiceEventTarget<TServiceEvents>,
  TServiceEventMsg extends { type: string; event: unknown }
>(
  postMessage: (msg: TServiceEventMsg) => void,
  eventMap: {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    [eventName in TServiceEvents["type"]]: (ev: any) => TServiceEventMsg;
  }
): TEventTarget {
  log.debug("Constructing WorkerEventHandler");

  const logAndPost = (msg: TServiceEventMsg) => {
    log.debug("Sending event message from worker: %o", msg);
    postMessage({ messageType: "event", ...msg });
  };
  const serviceEventTarget = new EventTarget() as TEventTarget;

  Object.keys(eventMap).forEach((eventName: TServiceEvents["type"]) => {
    log.debug("subscribing to event %s", eventName);

    serviceEventTarget.addEventListener(eventName, (ev) => {
      logAndPost(eventMap[eventName](ev));
    });
  });

  return serviceEventTarget;
}

type Method<C, T extends keyof C> = C[T] extends (...args: infer A) => infer R
  ? { argsTuple: A; returnType: R }
  : never;

type MethodOf<T, M extends keyof T> = T[M] extends (...args: any[]) => any
  ? M
  : never;

// This is the main function that the worker thread should delegate incoming messages to
export function invokeWorkerMethod<C, T extends keyof C>(
  data: {
    type: MethodOf<C, T>;
    args: Method<C, T>["argsTuple"];
  },
  service: C,
  postMessage: (msg: {
    messageType: "response";
    type: string;
    result: unknown;
  }) => void,
  serviceEventTarget: unknown // IServiceEventTarget<TServiceEvents>
) {
  log.debug(`Handling message in worker: ${data.type.toString()}`);
  const logAndPost = (msg: {
    messageType: "response";
    type: string;
    result: unknown;
  }) => {
    log.debug("Sending response message from worker: %o", msg);
    postMessage(msg);
  };

  try {
    log.debug(
      `Calling service method: ${data.type.toString()} with args ${JSON.stringify(
        data.args
      )}`
    );
    service[data.type]
      // @ts-expect-error Just... ok
      .call(service, ...data.args, serviceEventTarget)
      // @ts-expect-error yepyep
      .then((result) =>
        logAndPost({
          messageType: "response",
          // @ts-expect-error yepyepyep
          type: data.type + "-result",
          result,
        })
      );
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
  } catch (err: any) {
    // THIS DOESN'T WORK TODAY
    // If this happens then the wasm code likely threw an exception/paniced rather than
    // completing gracefully and fullfilling the promise. Communicate to the client
    // that there was an error and it should reject the current request

    logAndPost({ messageType: "response", type: "error-result", result: err });
  }
}
