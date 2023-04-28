// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { ShotResult, Dump, Result } from "./common.js";
import { log } from "./log.js";

// Create strongly typed compiler events
type QscEvent<T> = Event & { detail: T };
interface QscEventMap {
    "Message": QscEvent<string>;
    "DumpMachine": QscEvent<Dump>;
    "Result": QscEvent<Result>;
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
        type: K, listener: QscEventHandler<QscEventMap[K]>): void;

    removeEventListener<K extends keyof QscEventMap>(
        type: K, listener: QscEventHandler<QscEventMap[K]>): void;

    dispatchEvent(event: QscEvents): boolean;
}

// Convenience method that also provides type safety
export function makeEvent<K extends QscMessageNames>(type: K, detail: QscEventDetail<K>): QscEventMap[K] {
    let event = new Event(type) as QscEventMap[K];
    event.detail = detail;
    return event;
}

function makeResultObj(): ShotResult {
    return { success: false, result: "", events: [], };
}

export class QscEventTarget extends EventTarget implements IQscEventTarget {
    private results: ShotResult[] = [];
    private shotActive = false;

    // Overrides for the base EventTarget methods to limit to expected event types
    addEventListener<K extends keyof QscEventMap>(
        type: K, listener: QscEventHandler<QscEventMap[K]>): void {
        super.addEventListener(type, listener as EventListener);
    }

    removeEventListener<K extends keyof QscEventMap>(
        type: K, listener: QscEventHandler<QscEventMap[K]>): void {
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
        if (captureEvents) {
            this.addEventListener('Message', (ev) => this.onMessage(ev.detail));
            this.addEventListener('DumpMachine', (ev) => this.onDumpMachine(ev.detail));
            this.addEventListener('Result', (ev) => this.onResult(ev.detail));
        }
    }

    private onMessage(msg: string) {
        if (!this.shotActive) {
            this.results.push(makeResultObj());
            this.shotActive = true;
        }
        const shotIdx = this.results.length - 1;

        this.results[shotIdx].events.push({ "type": "Message", "message": msg });
    }

    private onDumpMachine(dump: Dump) {
        if (!this.shotActive) {
            this.results.push(makeResultObj());
            this.shotActive = true;
        }
        const shotIdx = this.results.length - 1;
        this.results[shotIdx].events.push({ "type": "DumpMachine", "state": dump });
    }

    private onResult(result: Result) {
        if (!this.shotActive) {
            this.results.push(makeResultObj());
            this.shotActive = true;
        }
        const shotIdx = this.results.length - 1;

        this.results[shotIdx].success = result.success;
        this.results[shotIdx].result = result.value;
        this.shotActive = false;
    }

    getResults(): ShotResult[] { return this.results; }
    clearResults(): void { this.results.length = 0; }
}
