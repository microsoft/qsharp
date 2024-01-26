// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { type VSDiagnostic } from "../../lib/web/qsc_wasm.js";

// Each DumpMachine output is represented as an object where each key is a basis
// state, e.g., "|3>" and the value is the [real, imag] parts of the complex amplitude.
export type Dump = {
  [index: string]: [number, number];
};

export type Result =
  | { success: true; value: string }
  | { success: false; value: VSDiagnostic };

interface DumpMsg {
  type: "DumpMachine";
  state: Dump;
}

interface MessageMsg {
  type: "Message";
  message: string;
}

interface ResultMsg {
  type: "Result";
  result: Result;
}

type EventMsg = ResultMsg | DumpMsg | MessageMsg;

function outputAsResult(msg: string): ResultMsg | null {
  try {
    const obj = JSON.parse(msg);
    if (obj?.type == "Result" && typeof obj.success == "boolean") {
      return {
        type: "Result",
        result: {
          success: obj.success,
          value: obj.result,
        },
      };
    }
  } catch {
    return null;
  }
  return null;
}

function outputAsMessage(msg: string): MessageMsg | null {
  try {
    const obj = JSON.parse(msg);
    if (obj?.type == "Message" && typeof obj.message == "string") {
      return obj as MessageMsg;
    }
  } catch {
    return null;
  }
  return null;
}

function outputAsDump(msg: string): DumpMsg | null {
  try {
    const obj = JSON.parse(msg);
    if (obj?.type == "DumpMachine" && typeof obj.state == "object") {
      return obj as DumpMsg;
    }
  } catch {
    return null;
  }
  return null;
}

export function eventStringToMsg(msg: string): EventMsg | null {
  return outputAsResult(msg) || outputAsMessage(msg) || outputAsDump(msg);
}

export type ShotResult = {
  success: boolean;
  result: string | VSDiagnostic;
  events: Array<MessageMsg | DumpMsg>;
};
