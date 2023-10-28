// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { TelemetryEvent, log } from "./log.js";
import { CancellationToken } from "./cancellation.js";

/**
 * Used as a type constraint for a "service", i.e. an object
 * we can create proxy methods for. The type shouldn't define
 * any non-method properties.
 */
type ServiceMethods<T> = { [x in keyof T]: (...args: any[]) => any };

/**
 * Defines the service methods that the proxy will handle and their types.
 * "request" is a normal async method.
 * "requestWithProgress" methods take an @see IServiceEventTarget to
 *   communicate events back to the main thread as they run. They also set
 *   the service state to "busy" while they run.
 * "addEventListener" and "removeEventListener" methods are used to
 *   subscribe to events from the service.
 */
export type MethodMap<T> = {
  [M in keyof T]:
    | "request"
    | "requestWithProgress"
    | "addEventListener"
    | "removeEventListener";
};

/** Methods added to the service when wrapped in a proxy */
export type IServiceProxy = {
  onstatechange: ((state: ServiceState) => void) | null;
  terminate: () => void;
};

/** "requestWithProgress" type methods will set the service state to "busy" */
export type ServiceState = "idle" | "busy";

/** Request message from a main thread to the worker */
export type RequestMessage<T extends ServiceMethods<T>> = {
  [K in keyof T]: { type: K; args: Parameters<T[K]> };
}[keyof T];

/** Response message for a request from the worker to the main thread */
export type ResponseMessage<T extends ServiceMethods<T>> = {
  messageType: "response";
} & {
  [K in keyof T]: {
    type: K;
    result:
      | { success: true; result: Awaited<ReturnType<T[K]>> }
      | { success: false; error: unknown };
  };
}[keyof T];

/** Event message from the worker to the main thread */
export type EventMessage<TEventMsg extends IServiceEventMessage> = {
  messageType: "event";
} & TEventMsg;

/** Used as a constraint for events defined by the service */
interface IServiceEventMessage {
  type: string;
  detail: unknown;
}

/**
 * Strongly typed EventTarget interface. Used as a constraint for the
 * event target that "requestWithProgress" methods should take in the service.
 */
export interface IServiceEventTarget<TEvents extends IServiceEventMessage> {
  addEventListener<T extends TEvents["type"]>(
    type: T,
    listener: (event: Event & Extract<TEvents, { type: T }>) => void,
  ): void;

  removeEventListener<T extends TEvents["type"]>(
    type: T,
    listener: (event: Event & Extract<TEvents, { type: T }>) => void,
  ): void;

  dispatchEvent(event: Event & TEvents): boolean;
}

/** Holds state for a single request received by the proxy */
type RequestState<
  TService extends ServiceMethods<TService>,
  TServiceEventMsg extends IServiceEventMessage,
> = RequestMessage<TService> & {
  resolve: (val: any) => void;
  reject: (err: any) => void;
  requestEventTarget?: IServiceEventTarget<TServiceEventMsg>;
  cancellationToken?: CancellationToken;
};

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
 * Function to create the proxy for a type. To be used from the main thread.
 *
 * @param postMessage A function to post messages to the worker
 * @param terminator A function to call to tear down the worker thread
 * @param methods A map of method names to be proxied and some metadata @see MethodMap
 * @returns The proxy object. The caller should then set the onMsgFromWorker
 * property to a callback that will receive messages from the worker.
 */
export function createProxy<
  TService extends ServiceMethods<TService>,
  TServiceEventMsg extends IServiceEventMessage,
>(
  postMessage: (msg: RequestMessage<TService>) => void,
  terminator: () => void,
  methods: MethodMap<TService>,
): TService &
  IServiceProxy & {
    onMsgFromWorker: (
      msg: ResponseMessage<TService> | EventMessage<TServiceEventMsg>,
    ) => void;
  } {
  const queue: RequestState<TService, TServiceEventMsg>[] = [];
  const eventTarget = new EventTarget();
  let curr: RequestState<TService, TServiceEventMsg> | undefined;
  let state: ServiceState = "idle";

  function setState(newState: ServiceState) {
    if (state === newState) return;
    state = newState;
    if (proxy.onstatechange) proxy.onstatechange(state);
  }

  type ResultOf<TRespMsg> = TRespMsg extends { result: infer R } ? R : never;

  function queueRequest(
    msg: RequestMessage<TService>,
    requestEventTarget?: IServiceEventTarget<TServiceEventMsg>,
    cancellationToken?: CancellationToken,
  ): Promise<ResultOf<ResponseMessage<TService>>> {
    return new Promise((resolve, reject) => {
      queue.push({
        type: msg.type,
        args: msg.args,
        resolve,
        reject,
        requestEventTarget,
        cancellationToken,
      } as RequestState<TService, TServiceEventMsg>);

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
      log.debug("Proxy: Worker queue is empty");
      setState("idle");
      return;
    }

    const msg = { type: curr.type, args: curr.args };
    if (methods[curr.type] === "requestWithProgress") {
      setState("busy");
    }

    log.debug("Proxy: Posting message to worker: %o", msg);
    postMessage(msg);
  }

  function onMsgFromWorker(
    msg: ResponseMessage<TService> | EventMessage<TServiceEventMsg>,
  ) {
    if (log.getLogLevel() >= 4)
      log.debug("Proxy: Received message from worker: %s", JSON.stringify(msg));

    if (msg.messageType === "event") {
      // For telemetry events, just log and exit. There is nothing else waiting to consume them.
      if (msg.type === "telemetry-event") {
        const detail = msg.detail as TelemetryEvent;
        log.logTelemetry(detail);
        return;
      }
      const event = new Event(msg.type) as Event & TServiceEventMsg;
      event.detail = msg.detail;

      log.debug("Proxy: Posting event: %o", msg);
      // Post to a currently attached event target if there's a "requestWithProgress"
      // in progress
      curr?.requestEventTarget?.dispatchEvent(event);
      // Also post to the general event target
      eventTarget.dispatchEvent(event);
    } else if (msg.messageType === "response") {
      if (!curr) {
        log.error("Proxy: No active request when message received: %o", msg);
        return;
      }
      const result = {
        success: msg.result.success,
        data: msg.result.success ? msg.result.result : msg.result.error,
      };
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

  // Create the proxy object to be returned
  const proxy = {} as TService &
    IServiceProxy & { onMsgFromWorker: typeof onMsgFromWorker };

  // Assign each method with the desired proxying behavior
  for (const methodName of Object.keys(methods) as (keyof TService &
    string)[]) {
    // @ts-expect-error - tricky to derive the type of the actual method here
    proxy[methodName] = (...args: any[]) => {
      let requestEventTarget:
        | IServiceEventTarget<TServiceEventMsg>
        | undefined = undefined;

      switch (methods[methodName]) {
        case "addEventListener":
          {
            // @ts-expect-error - can't get the typing of the rest parameters quite right
            eventTarget.addEventListener(...args);
          }
          break;
        case "removeEventListener":
          {
            // @ts-expect-error - can't get the typing of the rest parameters quite right
            eventTarget.removeEventListener(...args);
          }
          break;
        case "requestWithProgress": {
          // For progress methods, the last argument is the event target
          requestEventTarget = args[args.length - 1];
          args = args.slice(0, args.length - 1);
        }
        // fallthrough
        case "request": {
          return queueRequest(
            { type: methodName, args } as RequestMessage<TService>,
            requestEventTarget,
          );
        }
      }
    };
  }

  proxy.onstatechange = null;
  proxy.terminate = () => {
    // Kill the worker without a chance to shutdown. May be needed if it is not responding.
    log.info("Proxy: Terminating the worker");
    if (curr) {
      log.debug(
        "Proxy: Terminating running worker item of type: %s",
        curr.type,
      );
      curr.reject("terminated");
    }
    // Reject any outstanding items
    while (queue.length) {
      const item = queue.shift();
      log.debug(
        "Proxy: Terminating outstanding work item of type: %s",
        item?.type,
      );
      item?.reject("terminated");
    }
    terminator();
  };
  proxy.onMsgFromWorker = onMsgFromWorker;

  return proxy;
}

/**
 * Function to wrap a service in a dispatcher. To be used in the worker thread.
 *
 * @param service The service to be wrapped
 * @param methods A map of method names. Should match the list passed into @see createProxy.
 * @param eventNames The list of event names that the service can emit
 * @param postMessage A function to post messages back to the main thread
 * @returns A function that takes a message and invokes the corresponding
 * method on the service. The caller should then set this method as a message handler.
 */
export function createDispatcher<
  TService extends ServiceMethods<TService>,
  TServiceEventMsg extends IServiceEventMessage,
>(
  postMessage: (
    msg: ResponseMessage<TService> | EventMessage<TServiceEventMsg>,
  ) => void,
  service: TService,
  methods: MethodMap<TService>,
  eventNames: TServiceEventMsg["type"][],
) {
  log.debug("Worker: Constructing WorkerEventHandler");

  function logAndPost(
    msg: ResponseMessage<TService> | EventMessage<TServiceEventMsg>,
  ) {
    log.debug(
      "Worker: Sending %s message from worker: %o",
      msg.messageType,
      msg,
    );
    postMessage(msg);
  }

  const eventTarget =
    new EventTarget() as IServiceEventTarget<TServiceEventMsg>;

  eventNames.forEach((eventName: TServiceEventMsg["type"]) => {
    // Subscribe to all known events and forward them as messages to the main thread.
    eventTarget.addEventListener(eventName, (ev) => {
      logAndPost({
        messageType: "event",
        type: ev.type,
        detail: ev.detail,
      });
    });

    // If there's an addEventListener on the object itself, forward those events as well.
    if ((service as any).addEventListener) {
      (service as any).addEventListener(eventName, (ev: any) => {
        logAndPost({
          messageType: "event",
          type: ev.type,
          detail: ev.detail,
        });
      });
    }
  });

  return function invokeMethod(req: RequestMessage<TService>) {
    // Pass the eventTarget to the methods marked as taking progress
    return service[req.type]
      .call(
        service,
        ...req.args,
        methods[req.type] === "requestWithProgress" ? eventTarget : undefined,
      )
      .then((result: any) =>
        logAndPost({
          messageType: "response",
          type: req.type,
          result: { success: true, result },
        }),
      )
      .catch((err: any) =>
        logAndPost({
          // If this happens then the wasm code likely threw an exception/panicked rather than
          // completing gracefully and fullfilling the promise. Communicate to the client
          // that there was an error and it should reject the current request
          messageType: "response",
          type: req.type,
          result: { success: false, error: err },
        }),
      );
  };
}
