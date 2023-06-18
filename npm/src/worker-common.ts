// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as Comlink from "comlink";
import { ICompletionList } from "../lib/web/qsc_wasm.js";
import { VSDiagnostic } from "./common.js";
import { CompilerState, ICompiler, ICompilerWorker } from "./compiler.js";
import { IQscEventTarget } from "./events.js";

/**
 * This is a serializer for QscEvent objects which will allow comlink
 * to transfer them between the main thread and the worker.
 */
export const eventTransferHandler = {
  canHandle: ((obj: unknown) => obj instanceof Event) as (
    obj: unknown
  ) => obj is Event,
  serialize: (ev: Event) => {
    return [
      {
        type: ev.type,
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        detail: (ev as any).detail,
      },
      [],
    ];
  },
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  deserialize: (obj: any) => {
    const ev = new Event(obj.type);
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    (ev as any).detail = obj.detail;
    return ev;
  },
};

/**
 * This is a proxy to be used on the main thread side.
 * It exists to
 *  provide a terminate() method which will cancel outstanding operations
 *  wrap callback parameters in Comlink.proxy()
 */
export class CompilerProxy<TTerminatable extends { terminate(): void }>
  implements ICompilerWorker
{
  private signalTermination: (() => void) | undefined;
  // never is appropriate for the result type since this promise will never
  // resolve, only be rejected.
  private terminatePromise: Promise<never> = new Promise((resolve, reject) => {
    // This should run from the constructor, so signalTermination
    // will in fact always be defined.
    this.signalTermination = reject.bind(this, "terminated");
  });

  constructor(
    private compiler: Comlink.Remote<ICompiler>,
    private worker: TTerminatable
  ) {}
  checkCode(code: string): Promise<VSDiagnostic[]> {
    return Promise.race([this.terminatePromise, this.compiler.checkCode(code)]);
  }
  getHir(code: string): Promise<string> {
    return Promise.race([this.terminatePromise, this.compiler.getHir(code)]);
  }
  getCompletions(): Promise<ICompletionList> {
    return Promise.race([
      this.terminatePromise,
      this.compiler.getCompletions(),
    ]);
  }
  run(
    code: string,
    expr: string,
    shots: number,
    eventHandler: IQscEventTarget
  ): Promise<void> {
    return Promise.race([
      this.terminatePromise,
      this.compiler.run(code, expr, shots, Comlink.proxy(eventHandler)),
    ]);
  }
  runKata(
    user_code: string,
    verify_code: string,
    eventHandler: IQscEventTarget
  ): Promise<boolean> {
    return Promise.race([
      this.terminatePromise,
      this.compiler.runKata(
        user_code,
        verify_code,
        Comlink.proxy(eventHandler)
      ),
    ]);
  }
  setStateHandler(
    onstatechange: (state: CompilerState) => void
  ): Promise<void> {
    return Promise.race([
      this.terminatePromise,
      this.compiler.setStateHandler(Comlink.proxy(onstatechange)),
    ]);
  }
  terminate() {
    this.compiler[Comlink.releaseProxy]();
    this.worker.terminate();
    // signalTermination will in fact always be defined (see terminatePromise
    // initialization) but TypeScript doesn't know that.
    this.signalTermination?.();
  }
}
