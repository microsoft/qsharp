// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { ShotResult, Dump, Result } from "./common.js";
import { TelemetryEvent, log } from "../log.js";
import { IServiceEventTarget } from "../worker-proxy.js";

// Create strongly typed compiler events
export type QscEventData =
  | { type: "Message"; detail: string }
  | { type: "DumpMachine"; detail: Dump }
  | { type: "Result"; detail: Result }
  | { type: "telemetry-event"; detail: TelemetryEvent };

export type QscEvents = Event & QscEventData;

export type QscEvent<T extends QscEvents["type"]> = Extract<
  QscEvents,
  { type: T }
>;

// Strongly typed event target for compiler operations.
export type IQscEventTarget = IServiceEventTarget<QscEventData>;

// Convenience method that also provides type safety
export function makeEvent<E extends QscEvents>(
  type: E["type"],
  detail: E["detail"]
): E {
  const event = new Event(type) as E;
  event.detail = detail;
  return event;
}

function makeResultObj(): ShotResult {
  return { success: false, result: "", events: [] };
}

// The actual event target implementation adds one more event type
// to notify the UI that the results should be refreshed.
// This event does not come from the compiler service itself
// so it's not exposed as part of QscEvents or IQscEventTarget.
// Direct consumers of QscEventTarget can attach a listener for
// this event.
type QscUiEvents =
  | QscEvents
  | (Event & { type: "uiResultsRefresh"; detail: undefined });

export class QscEventTarget implements IQscEventTarget {
  private eventTarget = new EventTarget();
  private results: ShotResult[] = [];
  private shotActive = false;
  private animationFrameId = 0;
  private supportsUiRefresh = false;

  // Overrides for the base EventTarget methods to limit to expected event types
  addEventListener<T extends QscUiEvents["type"]>(
    type: T,
    listener: (event: Extract<QscEvents, { type: T }>) => void
  ): void {
    this.eventTarget.addEventListener(type, listener as EventListener);
  }

  removeEventListener<T extends QscUiEvents["type"]>(
    type: T,
    listener: (event: Extract<QscEvents, { type: T }>) => void
  ): void {
    this.eventTarget.removeEventListener(type, listener as EventListener);
  }

  dispatchEvent(event: QscUiEvents): boolean {
    if (log.getLogLevel() >= 4) log.debug("Dispatching event: %o", event);
    return this.eventTarget.dispatchEvent(event);
  }

  /**
   * @param captureEvents Set to true if this instance should record events internally
   */
  constructor(captureEvents: boolean) {
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
    const uiRefreshEvent = new Event("uiResultsRefresh") as QscUiEvents;
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
