// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import {
  BreakpointSpan,
  DebugService,
  StackFrame,
} from "../../lib/node/qsc_wasm.cjs";
import { eventStringToMsg } from "../compiler/common.js";
import { IQscEventTarget, QscEvents, makeEvent } from "../compiler/events.js";
import { log } from "../log.js";
import { IServiceProxy } from "../worker-proxy.js";
type QscWasm = typeof import("../../lib/node/qsc_wasm.cjs");

// These need to be async/promise results for when communicating across a WebWorker, however
// for running the debugger in the same thread the result will be synchronous (a resolved promise).
export interface IDebugService {
  loadSource(path: string, source: string): Promise<boolean>;
  getBreakpoints(path: string): Promise<BreakpointSpan[]>;
  getStackFrames(): Promise<StackFrame[]>;
  evalContinue(
    bps: number[],
    eventHandler: IQscEventTarget
  ): Promise<number | undefined>;
  dispose(): Promise<void>;
}

export type IDebugServiceWorker = IDebugService & IServiceProxy;

export class QSharpDebugService implements IDebugService {
  private debugService: DebugService;

  constructor(wasm: QscWasm) {
    log.info("Constructing a QSharpDebugService instance");
    this.debugService = new wasm.DebugService();
  }

  async loadSource(path: string, source: string): Promise<boolean> {
    return this.debugService.load_source(path, source);
  }

  async getStackFrames(): Promise<StackFrame[]> {
    const stack_frame_list = this.debugService.get_stack_frames();
    const stack_frames: StackFrame[] = stack_frame_list.frames;
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

  async getBreakpoints(path: string): Promise<BreakpointSpan[]> {
    const breakpoint_list = this.debugService.get_breakpoints(path);
    const breakpoint_spans: BreakpointSpan[] = breakpoint_list.spans;
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
