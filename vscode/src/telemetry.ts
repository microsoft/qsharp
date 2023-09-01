import * as vscode from "vscode";
import TelemetryReporter, {
  TelemetryEventMeasurements,
  TelemetryEventProperties,
} from "@vscode/extension-telemetry";
import { log } from "qsharp";
// the application insights key (also known as instrumentation key)
const key = "175861b7-3a41-4015-9571-1d930b8b0722";

// telemetry reporter
let reporter: TelemetryReporter | undefined;

export enum EventType {
  Install = "Install",
  DebugSessionStart = "DebugSessionStart",
  Compile = "Compile",
  Initialize = "Initialize",
}
type Empty = { [K in any]: never };

type EventTypes = {
  [EventType.Install]: {
    properties: Empty;
    measurements: {
      timeToInstall: number;
    };
  };
  [EventType.Initialize]: {
    properties: Empty;
    measurements: {};
  };
  [EventType.Compile]: {
    properties: Empty;
    measurements: {};
  };
  [EventType.DebugSessionStart]: {
    properties: Empty;
    measurements: {};
  };
};

export function initTelemetry(context: vscode.ExtensionContext) {
  // create telemetry reporter on extension activation
  reporter = new TelemetryReporter(key);
  // ensure it gets properly disposed. Upon disposal the events will be flushed
  context.subscriptions.push(reporter);
  sendTelemetryEvent(EventType.Initialize, {}, {});
}

export function sendTelemetryEvent<E extends keyof EventTypes>(
  event: E,
  properties: EventTypes[E]["properties"] = {},
  measurements: EventTypes[E]["measurements"] = {},
) {
  if (reporter !== undefined) {
    reporter.sendTelemetryEvent(event, properties, measurements);
    log.info(`Sent telemetry event ${event}`);
  } else {
    log.info(`Did not send telemetry event ${event}.`);
  }
}
