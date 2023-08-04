// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import type {
  IBreakpointSpan,
  DebugService,
  IStackFrame,
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
  loadSource(path: string, source: string): Promise<boolean>;
  getBreakpoints(path: string): Promise<IBreakpointSpan[]>;
  getStackFrames(): Promise<IStackFrame[]>;
  evalContinue(
    bps: number[],
    eventHandler: IQscEventTarget
  ): Promise<number | undefined>;
  dispose(): Promise<void>;
}

export type IDebugServiceWorker = IDebugService & IServiceProxy;

export class QSharpDebugService implements IDebugService {
  private debugService: DebugService;

  // We need to keep a copy of the code for mapping diagnostics to utf16 offsets
  private code: { [uri: string]: string } = {};

  constructor(wasm: QscWasm) {
    log.info("Constructing a QSharpDebugService instance");
    this.debugService = new wasm.DebugService();
  }

  async loadSource(path: string, source: string): Promise<boolean> {
    this.code[path] = source;
    return this.debugService.load_source(path, source);
  }

  async getStackFrames(): Promise<IStackFrame[]> {
    const stack_frame_list = this.debugService.get_stack_frames();

    const stack_frames: IStackFrame[] = stack_frame_list.frames.map(
      (frame: IStackFrame) => {
        if (frame.path in this.code) {
          const mappedSpan = mapUtf8UnitsToUtf16Units(
            [frame.lo, frame.hi],
            this.code[frame.path]
          );
          const result = {} as IStackFrame;
          result.name = frame.name;
          result.path = frame.path;
          result.lo = mappedSpan[frame.lo];
          result.hi = mappedSpan[frame.hi];
          return result;
        } else {
          return frame;
        }
      }
    );
    return stack_frames;
  }

  async evalContinue(
    bps: number[],
    eventHandler: IQscEventTarget
  ): Promise<number | undefined> {
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
      (span: IBreakpointSpan) => {
        const result = {} as IBreakpointSpan;
        result.id = span.id;
        result.lo = positionMap[span.lo];
        result.hi = positionMap[span.hi];
        return result;
      }
    );
    return breakpoint_spans;
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
