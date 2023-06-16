/* eslint-disable @typescript-eslint/no-non-null-assertion */
import * as Comlink from "comlink";
import { Compiler, CompilerState, ICompiler } from "./compiler.js";
import * as wasm from "../lib/web/qsc_wasm.js";
import { log } from "./log.js";
import { ICompletionList } from "../lib/node/qsc_wasm.cjs";
import { VSDiagnostic } from "./common.js";
import { IQscEventTarget } from "./events.js";

type InitableCompiler = {
  init(w: WebAssembly.Module, qscLogLevel: number): void;
} & ICompiler;

Comlink.transferHandlers.set("EVENT", {
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
});

class CompilerWorker implements InitableCompiler {
  private compiler: ICompiler | undefined;

  init(wasmModule: WebAssembly.Module, qscLogLevel: number) {
    log.setLogLevel(qscLogLevel);
    wasm.initSync(wasmModule);
    this.compiler = new Compiler(wasm);
  }

  checkCode(code: string): Promise<VSDiagnostic[]> {
    return this.compiler!.checkCode(code);
  }
  getHir(code: string): Promise<string> {
    return this.compiler!.getHir(code);
  }
  getCompletions(): Promise<ICompletionList> {
    return this.compiler!.getCompletions();
  }
  run(
    code: string,
    expr: string,
    shots: number,
    eventHandler: IQscEventTarget
  ): Promise<void> {
    return this.compiler!.run(code, expr, shots, eventHandler);
  }
  runKata(
    user_code: string,
    verify_code: string,
    eventHandler: IQscEventTarget
  ): Promise<boolean> {
    return this.compiler!.runKata(user_code, verify_code, eventHandler);
  }
  setStateHandler(
    onstatechange: (state: CompilerState) => void
  ): Promise<void> {
    return this.compiler!.setStateHandler(onstatechange);
  }
}

const compiler: InitableCompiler = new CompilerWorker();

Comlink.expose(compiler);
