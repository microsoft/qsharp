// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as wasm from "../../lib/web/qsc_wasm.js";
import { log } from "../log.js";
import {
  IServiceEventMessage,
  IServiceProxy,
  RequestMessage,
  ServiceMethods,
  ServiceProtocol,
  createProxyInternal,
  initService,
} from "./common.js";

/**
 * Creates an initializes a service, setting it up to receive requests.
 * This function to be is used in the worker.
 *
 * @param serviceProtocol An object that describes the service: its constructor, methods and events
 * @returns A message handler to be assigned to the `self.onmessage` handler in a web worker
 */
export function createWorker<
  TService extends ServiceMethods<TService>,
  TServiceEventMsg extends IServiceEventMessage,
>(
  serviceProtocol: ServiceProtocol<TService, TServiceEventMsg>,
): (e: MessageEvent) => void {
  let invokeService: ((req: RequestMessage<TService>) => Promise<void>) | null =
    null;

  // This export should be assigned to 'self.onmessage' in a WebWorker
  return function messageHandler(e: MessageEvent) {
    const data = e.data;

    if (!data.type || typeof data.type !== "string") {
      log.error(`Unrecognized msg: ${data}`);
      return;
    }

    switch (data.type) {
      case "init":
        {
          wasm.initSync({ module: data.wasmModule });

          invokeService = initService<TService, TServiceEventMsg>(
            self.postMessage.bind(self),
            serviceProtocol,
            wasm,
            data.qscLogLevel,
          );
        }
        break;
      default:
        if (!invokeService) {
          log.error(
            `Received message before the service was initialized: %o`,
            data,
          );
        } else {
          invokeService(data);
        }
    }
  };
}

/**
 * Creates and initializes a service in a web worker, and returns a proxy for the service
 * to be used from the main thread.
 *
 * @param workerArg The service web worker or the URL of the web worker script.
 * @param wasmModule The wasm module to initialize the service with
 * @param serviceProtocol An object that describes the service: its constructor, methods and events
 * @returns A proxy object that implements the service interface.
 *   This interface can now be used as if calling into the real service,
 *   and the calls will be proxied to the web worker.
 */
export function createProxy<
  TService extends ServiceMethods<TService>,
  TServiceEventMsg extends IServiceEventMessage,
>(
  workerArg: string | Worker,
  wasmModule: WebAssembly.Module,
  serviceProtocol: ServiceProtocol<TService, TServiceEventMsg>,
): TService & IServiceProxy {
  // Create or use the WebWorker
  const worker =
    typeof workerArg === "string" ? new Worker(workerArg) : workerArg;

  // Send it the Wasm module to instantiate
  worker.postMessage({
    type: "init",
    wasmModule,
    qscLogLevel: log.getLogLevel(),
  });

  // If you lose the 'this' binding, some environments have issues
  const postMessage = worker.postMessage.bind(worker);
  const onTerminate = () => worker.terminate();

  // Create the proxy which will forward method calls to the worker
  const proxy = createProxyInternal<TService, TServiceEventMsg>(
    postMessage,
    onTerminate,
    serviceProtocol.methods,
  );

  // Let proxy handle response and event messages from the worker
  worker.onmessage = (ev) => proxy.onMsgFromWorker(ev.data);
  return proxy;
}
