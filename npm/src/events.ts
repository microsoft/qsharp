// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { ShotResult, Dump, Result, VSDiagnostic } from "./common.js";
import { log } from "./log.js";

// Create strongly typed compiler events
type QscEvent<T> = Event & { detail: T };
interface QscEventMap {
  Message: QscEvent<string>;
  DumpMachine: QscEvent<Dump>;
  Result: QscEvent<Result>;
  diagnostics: QscEvent<VSDiagnostic[]>;
  uiResultsRefresh: QscEvent<void>;
}

// Union of valid message names
type QscMessageNames = keyof QscEventMap;

// Given the message name, return the type of the 'details' property
type QscEventDetail<K extends QscMessageNames> = QscEventMap[K]["detail"];

// Union of all possible events types that may be dispatched
export type QscEvents = QscEventMap[keyof QscEventMap];

type QscEventHandler<T extends Event> = (event: T) => void;

// Strongly typed event target for compiler operations.
export interface IQscEventTarget {
  addEventListener<K extends keyof QscEventMap>(
    type: K,
    listener: QscEventHandler<QscEventMap[K]>
  ): void;

  removeEventListener<K extends keyof QscEventMap>(
    type: K,
    listener: QscEventHandler<QscEventMap[K]>
  ): void;

  dispatchEvent(event: QscEvents): boolean;
}

// Convenience method that also provides type safety
export function makeEvent<K extends QscMessageNames>(
  type: K,
  detail: QscEventDetail<K>
): QscEventMap[K] {
  const event = new Event(type) as QscEventMap[K];
  event.detail = detail;
  return event;
}

function makeResultObj(): ShotResult {
  return { success: false, result: "", events: [] };
}

export class QscEventTarget extends EventTarget implements IQscEventTarget {
  private results: ShotResult[] = [];
  private shotActive = false;
  private animationFrameId = 0;
  private supportsUiRefresh = false;

  // Overrides for the base EventTarget methods to limit to expected event types
  addEventListener<K extends keyof QscEventMap>(
    type: K,
    listener: QscEventHandler<QscEventMap[K]>
  ): void {
    super.addEventListener(type, listener as EventListener);
  }

  removeEventListener<K extends keyof QscEventMap>(
    type: K,
    listener: QscEventHandler<QscEventMap[K]>
  ): void {
    super.removeEventListener(type, listener as EventListener);
  }

  dispatchEvent(event: QscEvents): boolean {
    if (log.getLogLevel() >= 4) log.debug("Dispatching event: %o", event);
    return super.dispatchEvent(event);
  }

  /**
   * @param captureEvents Set to true if this instance should record events internally
   */
  constructor(captureEvents: boolean) {
    super();
    this.supportsUiRefresh =
      typeof globalThis.requestAnimationFrame === "function";

    if (captureEvents) {
      this.addEventListener("Message", (ev) => this.onMessage(ev.detail));
      this.addEventListener("DumpMachine", (ev) =>
        this.onDumpMachine(ev.detail)
      );
      this.addEventListener("Result", (ev) => this.onResult(ev.detail));
    }
  }

  private onMessage(msg: string) {
    this.ensureActiveShot();

    const shotIdx = this.results.length - 1;
    this.results[shotIdx].events.push({ type: "Message", message: msg });

    this.queueUiRefresh();
  }

  private onDumpMachine(dump: Dump) {
    this.ensureActiveShot();

    const shotIdx = this.results.length - 1;
    this.results[shotIdx].events.push({ type: "DumpMachine", state: dump });

    this.queueUiRefresh();
  }

  private onResult(result: Result) {
    this.ensureActiveShot();

    const shotIdx = this.results.length - 1;

    this.results[shotIdx].success = result.success;
    this.results[shotIdx].result = result.value;
    this.shotActive = false;

    this.queueUiRefresh();
  }

  private ensureActiveShot() {
    if (!this.shotActive) {
      this.results.push(makeResultObj());
      this.shotActive = true;
    }
  }

  private queueUiRefresh() {
    if (this.supportsUiRefresh && !this.animationFrameId) {
      this.animationFrameId = requestAnimationFrame(() => {
        this.onUiRefresh();
      });
    }
  }

  private onUiRefresh() {
    this.animationFrameId = 0;
    const uiRefreshEvent = makeEvent("uiResultsRefresh", undefined);
    this.dispatchEvent(uiRefreshEvent);
  }

  getResults(): ShotResult[] {
    return this.results;
  }

  resultCount(): number {
    // May be one less than length if the last is still in flight
    return this.shotActive ? this.results.length - 1 : this.results.length;
  }

  clearResults(): void {
    this.results = [];
    this.shotActive = false;
  }
}
