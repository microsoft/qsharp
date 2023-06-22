// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { log } from "./log.js";
import { CancellationToken } from "./cancellation.js";

// These will be parameters
export type RequestMessage<
  T extends { [x in keyof T]: (...args: any[]) => any }
> = {
  [K in keyof T]: { type: K; args: Parameters<T[K]> };
}[keyof T];

export type ResponseMessage<
  T extends { [x in keyof T]: (...args: any[]) => Promise<any> }
> = {
  [K in keyof T]: {
    type: K;
    result:
      | { success: true; result: Awaited<ReturnType<T[K]>> }
      | { success: false; error: unknown };
  };
}[keyof T];

type IServiceWorker = {
  onstatechange: ((state: ServiceState) => void) | null;
  terminate: () => void;
};

export interface IServiceEventTarget<
  TEvents extends { type: string; detail: unknown }
> {
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

type RequestState<
  TServiceReqMsg extends IServiceRequestMessage,
  TEvents extends IServiceEventMessage
> = TServiceReqMsg & {
  resolve: (val: any) => void;
  reject: (err: any) => void;
  uiEventTarget?: IServiceEventTarget<TEvents>;
  cancellationToken?: CancellationToken;
};

interface IServiceRequestMessage {
  type: string;
  args: unknown[];
}

interface IServiceResponseMessage {
  type: string;
  result:
    | { success: true; result: unknown }
    | { success: false; error: unknown };
}

interface IServiceEventMessage {
  type: string;
  detail: unknown;
}

type ExtractResult<TRespMsg> = TRespMsg extends { result: infer R } ? R : never;
export type ResponseMessageWithType<TRespMsg> = {
  messageType: "response";
} & TRespMsg;
export type EventMessageWithType<TEventMsg> = {
  messageType: "event";
} & TEventMsg;

export type ServiceResponseMessageWithType =
  | ResponseMessageWithType<IServiceResponseMessage>
  | EventMessageWithType<IServiceEventMessage>;

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
  TServiceReqMsg extends IServiceRequestMessage,
  TServiceRespMsg extends IServiceResponseMessage,
  TServiceEvents extends Event & IServiceEventMessage,
  TServiceEventMsg extends IServiceEventMessage,
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
  methods: { [M in TServiceReqMsg["type"]]: { longRunning: boolean } }
): TServiceWorker {
  const queue: RequestState<TServiceReqMsg, TServiceEvents>[] = [];
  let curr: RequestState<TServiceReqMsg, TServiceEvents> | undefined;
  let state: ServiceState = "idle";

  function setState(newState: ServiceState) {
    if (state === newState) return;
    state = newState;
    if (proxy.onstatechange) proxy.onstatechange(state);
  }

  function queueRequest(
    msg: TServiceReqMsg,
    uiEventTarget?: IServiceEventTarget<TServiceEvents>,
    cancellationToken?: CancellationToken
  ): Promise<ExtractResult<TServiceRespMsg>> {
    return new Promise((resolve, reject) => {
      queue.push({
        type: msg.type,
        args: msg.args,
        resolve,
        reject,
        uiEventTarget,
        cancellationToken,
      } as RequestState<TServiceReqMsg, TServiceEvents>);

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
    log.debug(`handling request ${curr.type}`);

    const msg = { type: curr.type, args: curr.args };
    log.debug("request message: " + JSON.stringify(msg));
    if (methods[curr.type as TServiceReqMsg["type"]].longRunning) {
      setState("busy");
    }

    if (log.getLogLevel() >= 4) log.debug("Posting message to worker: %o", msg);
    postMessage(msg as TServiceReqMsg);
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
      const event = new Event(msg.type) as TServiceEvents;
      event.detail = msg.detail;

      log.debug("Posting event: %o", msg);
      curr.uiEventTarget?.dispatchEvent(event);
    } else if (msg.messageType === "response") {
      const result = {
        success: msg.result.success,
        data: msg.result.success ? msg.result.result : msg.result.error,
      };
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

  for (const methodName of Object.keys(methods)) {
    // @ts-expect-error let's just power through this TypeScript
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    proxy[methodName] = (...args: any[]) => {
      log.debug(`method ${methodName} called`);
      const longRunning =
        methods[methodName as TServiceReqMsg["type"]].longRunning;
      // TODO: make the event target the first argument and then this won't be so painful
      let uiEventTarget: IServiceEventTarget<TServiceEvents> | undefined =
        undefined;
      if (longRunning) {
        uiEventTarget = args[args.length - 1];
        args = args.slice(0, args.length - 1);
      }
      log.debug(`about to queue a request with args ${JSON.stringify(args)}`);
      return queueRequest(
        { type: methodName, args } as TServiceReqMsg,
        uiEventTarget
      );
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
  TEvent extends { type: string; detail: unknown } // e.g. QscEventData
>(
  eventNames: TEvent["type"][],
  postMessage: (msg: EventMessageWithType<TEvent>) => void
): IServiceEventTarget<TEvent> {
  log.debug("Constructing WorkerEventHandler");

  const logAndPost = (msg: EventMessageWithType<TEvent>) => {
    log.debug("Sending event message from worker: %o", msg);
    postMessage(msg);
  };
  const serviceEventTarget =
    new EventTarget() as unknown as IServiceEventTarget<TEvent>;

  eventNames.forEach((eventName: TEvent["type"]) => {
    log.debug("subscribing to event %s", eventName);

    serviceEventTarget.addEventListener(eventName, (ev) => {
      logAndPost({
        messageType: "event",
        type: ev.type,
        detail: ev.detail,
      } as EventMessageWithType<TEvent>);
    });
  });

  return serviceEventTarget;
}

// This is the main function that the worker thread should delegate incoming messages to
export function invokeWorkerMethod<
  TService extends { [x in keyof TService]: (...args: any[]) => any }
>(
  data: RequestMessage<TService>,
  service: TService,
  postMessage: (
    msg: ResponseMessageWithType<ResponseMessage<TService>>
  ) => void,
  serviceEventTarget: unknown // IServiceEventTarget<TServiceEvents>
) {
  log.debug(`Handling message in worker: ${data.type.toString()}`);
  const logAndPost = (
    msg: ResponseMessageWithType<ResponseMessage<TService>>
  ) => {
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
      .call(service, ...data.args, serviceEventTarget)
      // @ts-expect-error yepyep
      .then((result) =>
        logAndPost({
          messageType: "response",
          type: data.type,
          result: { success: true, result },
        })
      );
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
  } catch (err: any) {
    // THIS DOESN'T WORK TODAY
    // If this happens then the wasm code likely threw an exception/paniced rather than
    // completing gracefully and fullfilling the promise. Communicate to the client
    // that there was an error and it should reject the current request
    logAndPost({
      messageType: "response",
      type: data.type,
      result: { success: false, error: err },
    });
  }
}
