import * as vscode from "vscode";
import TelemetryReporter, {
  TelemetryEventMeasurements,
  TelemetryEventProperties,
} from "@vscode/extension-telemetry";
import { log } from "qsharp";
// the application insights key (also known as instrumentation key)
const key = "AIF-d9b70cd4-b9f9-4d70-929b-a071c400b217";

// telemetry reporter
let reporter: TelemetryReporter | undefined;

export enum EventType {
  DebugSessionStart = "DebugSessionStart",
  Initialize = "Initialize",
  LoadLanguageService = "LoadLanguageService",
  QSharpJupyterCellInitialized = "QSharpJupyterCellInitialized",
}

type Empty = { [K in any]: never };

type EventTypes = {
  [EventType.Initialize]: {
    properties: Empty;
    measurements: Empty;
  };
  [EventType.DebugSessionStart]: {
    properties: Empty;
    measurements: {
      timeToStartMs: number;
    };
  };
  [EventType.LoadLanguageService]: {
    properties: Empty;
    measurements: {
      timeToStartMs: number;
    };
  };
  [EventType.QSharpJupyterCellInitialized] :{
    properties: Empty,
    measurements: Empty
  }
};

type WrappedTelemetryEvent = {
  id: string;
  data?: {
    measurements: TelemetryEventMeasurements;
    properties: TelemetryEventProperties;
  };
};

export function initTelemetry() {
  // see issue here: https://github.com/microsoft/vscode-extension-telemetry/issues/183
  // we cannot use the latest version of extension-telemetry until this is fixed
  const reporter = new TelemetryReporter("qsharp-vscode", "0.0.0", key);
  log.setTelemetryCollector(
    ({
      id,
      data: { properties, measurements } = { properties: {}, measurements: {} },
    }: WrappedTelemetryEvent) =>
      reporter.sendTelemetryEvent(id, properties, measurements)
  );
  sendTelemetryEvent(EventType.Initialize, {}, {});
}

export function sendTelemetryEvent<E extends keyof EventTypes>(
  event: E,
  properties: EventTypes[E]["properties"] = {},
  measurements: EventTypes[E]["measurements"] = {}
) {
  log.logTelemetry({ id: event, data: { properties, measurements } });
  if (reporter !== undefined) {
    reporter.sendTelemetryEvent(event, properties, measurements);
    log.info(`Sent telemetry event ${event}`);
  } else {
    log.info("Telemetry reporter undefined.");
  }
}
