// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import type {
  IBreakpointSpan,
  DebugService,
  IStackFrame,
  IStructStepResult,
  IVariable,
  IQuantumState,
} from "../../lib/node/qsc_wasm.cjs";
import { eventStringToMsg } from "../compiler/common.js";
import { IQscEventTarget, QscEvents, makeEvent } from "../compiler/events.js";
import { log } from "../log.js";
import { mapUtf8UnitsToUtf16Units } from "../vsdiagnostic.js";
import { IServiceProxy } from "../worker-proxy.js";
type QscWasm = typeof import("../../lib/node/qsc_wasm.cjs");

// These need to be async/promise results for when communicating across a WebWorker, however
// for running the debugger in the same thread the result will be synchronous (a resolved promise).
export interface IDebugService {
  loadSource(
    path: string,
    source: string,
    entry: string | undefined,
  ): Promise<string>;
  getBreakpoints(path: string): Promise<IBreakpointSpan[]>;
  getLocalVariables(): Promise<Array<IVariable>>;
  captureQuantumState(): Promise<Array<IQuantumState>>;
  getStackFrames(): Promise<IStackFrame[]>;
  evalContinue(
    bps: number[],
    eventHandler: IQscEventTarget,
  ): Promise<IStructStepResult>;
  evalNext(
    bps: number[],
    eventHandler: IQscEventTarget,
  ): Promise<IStructStepResult>;
  evalStepIn(
    bps: number[],
    eventHandler: IQscEventTarget,
  ): Promise<IStructStepResult>;
  evalStepOut(
    bps: number[],
    eventHandler: IQscEventTarget,
  ): Promise<IStructStepResult>;
  dispose(): Promise<void>;
}

export type IDebugServiceWorker = IDebugService & IServiceProxy;

export class QSharpDebugService implements IDebugService {
  private wasm: QscWasm;
  private debugService: DebugService;

  // We need to keep a copy of the code for mapping diagnostics to utf16 offsets
  private code: { [uri: string]: string } = {};

  constructor(wasm: QscWasm) {
    log.info("Constructing a QSharpDebugService instance");
    this.wasm = wasm;
    this.debugService = new wasm.DebugService();
  }

  async loadSource(
    path: string,
    source: string,
    entry: string | undefined,
  ): Promise<string> {
    this.code[path] = source;
    return this.debugService.load_source(path, source, entry);
  }

  async getStackFrames(): Promise<IStackFrame[]> {
    const stack_frame_list = this.debugService.get_stack_frames();

    const stack_frames: IStackFrame[] = await Promise.all(
      stack_frame_list.frames.map(async (frame) => {
        // get any missing sources if possible
        if (!(frame.path in this.code)) {
          const content = await this.wasm.get_library_source_content(
            frame.path,
          );
          if (content) {
            this.code[frame.path] = content;
          }
        }
        if (frame.path in this.code) {
          const mappedSpan = mapUtf8UnitsToUtf16Units(
            [frame.lo, frame.hi],
            this.code[frame.path],
          );
          return {
            ...frame,
            lo: mappedSpan[frame.lo],
            hi: mappedSpan[frame.hi],
          };
        } else {
          // We don't have a source file for this frame,
          // and we couldn't load it, so just return it as-is
          return frame;
        }
      }),
    );
    return stack_frames;
  }

  async evalNext(
    bps: number[],
    eventHandler: IQscEventTarget,
  ): Promise<IStructStepResult> {
    const event_cb = (msg: string) => onCompilerEvent(msg, eventHandler);
    const ids = new Uint32Array(bps);
    return this.debugService.eval_next(event_cb, ids);
  }

  async evalStepIn(
    bps: number[],
    eventHandler: IQscEventTarget,
  ): Promise<IStructStepResult> {
    const event_cb = (msg: string) => onCompilerEvent(msg, eventHandler);
    const ids = new Uint32Array(bps);
    return this.debugService.eval_step_in(event_cb, ids);
  }

  async evalStepOut(
    bps: number[],
    eventHandler: IQscEventTarget,
  ): Promise<IStructStepResult> {
    const event_cb = (msg: string) => onCompilerEvent(msg, eventHandler);
    const ids = new Uint32Array(bps);
    return this.debugService.eval_step_out(event_cb, ids);
  }

  async evalContinue(
    bps: number[],
    eventHandler: IQscEventTarget,
  ): Promise<IStructStepResult> {
    const event_cb = (msg: string) => onCompilerEvent(msg, eventHandler);
    const ids = new Uint32Array(bps);
    return this.debugService.eval_continue(event_cb, ids);
  }

  async getBreakpoints(path: string): Promise<IBreakpointSpan[]> {
    const breakpoint_list = this.debugService.get_breakpoints(path);
    // Get a map of the Rust source positions to the JavaScript source positions
    const positions: number[] = [];
    breakpoint_list.spans.forEach((span: IBreakpointSpan) => {
      positions.push(span.lo);
      positions.push(span.hi);
    });
    const positionMap = mapUtf8UnitsToUtf16Units(positions, this.code[path]);
    const breakpoint_spans: IBreakpointSpan[] = breakpoint_list.spans.map(
      (span) => ({
        id: span.id,
        lo: positionMap[span.lo],
        hi: positionMap[span.hi],
      }),
    );
    return breakpoint_spans;
  }

  async captureQuantumState(): Promise<Array<IQuantumState>> {
    const state = this.debugService.capture_quantum_state();
    return state.entries;
  }

  async getLocalVariables(): Promise<Array<IVariable>> {
    const variable_list = this.debugService.get_locals();
    return variable_list.variables;
  }

  async dispose() {
    this.debugService.free();
  }
}

export function onCompilerEvent(msg: string, eventTarget: IQscEventTarget) {
  const qscMsg = eventStringToMsg(msg);
  if (!qscMsg) {
    log.error("Unknown event message: %s", msg);
    return;
  }

  let qscEvent: QscEvents;

  const msgType = qscMsg.type;
  switch (msgType) {
    case "Message":
      qscEvent = makeEvent("Message", qscMsg.message);
      break;
    case "DumpMachine":
      qscEvent = makeEvent("DumpMachine", qscMsg.state);
      break;
    case "Result":
      qscEvent = makeEvent("Result", qscMsg.result);
      break;
    default:
      log.never(msgType);
      throw "Unexpected message type";
  }
  log.debug("worker dispatching event " + JSON.stringify(qscEvent));
  eventTarget.dispatchEvent(qscEvent);
}
