// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { VSDiagnostic } from "../vsdiagnostic.js";
import { log } from "../log.js";

export type Diagnostics = {
  uri: string;
  version: number;
  diagnostics: VSDiagnostic[];
};

export type LanguageServiceEvents = Event & {
  type: "diagnostics";
  detail: Diagnostics;
};

export interface ILanguageServiceEventTarget {
  addEventListener<T extends LanguageServiceEvents["type"]>(
    type: T,
    listener: (event: Extract<LanguageServiceEvents, { type: T }>) => void
  ): void;

  removeEventListener<T extends LanguageServiceEvents["type"]>(
    type: T,
    listener: (event: Extract<LanguageServiceEvents, { type: T }>) => void
  ): void;

  dispatchEvent(event: LanguageServiceEvents): boolean;
}

export function makeEvent<E extends LanguageServiceEvents>(
  type: E["type"],
  detail: E["detail"]
): E {
  const event = new Event(type) as E;
  event.detail = detail;
  return event;
}

export class LanguageServiceEventTarget implements ILanguageServiceEventTarget {
  private eventTarget = new EventTarget();

  // Overrides for the base EventTarget methods to limit to expected event types
  addEventListener<T extends LanguageServiceEvents["type"]>(
    type: T,
    listener: (event: Extract<LanguageServiceEvents, { type: T }>) => void
  ): void {
    this.eventTarget.addEventListener(type, listener as EventListener);
  }

  removeEventListener<T extends LanguageServiceEvents["type"]>(
    type: T,
    listener: (event: Extract<LanguageServiceEvents, { type: T }>) => void
  ): void {
    this.eventTarget.removeEventListener(type, listener as EventListener);
  }

  dispatchEvent(event: LanguageServiceEvents): boolean {
    if (log.getLogLevel() >= 4) log.debug("Dispatching event: %o", event);
    return this.eventTarget.dispatchEvent(event);
  }
}
