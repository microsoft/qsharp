import * as vscode from "vscode";
import TelemetryReporter from "@vscode/extension-telemetry";
import { log } from "qsharp-lang";
import { v4 } from "uuid";

export enum EventType {
  DebugSessionStart = "Qsharp.DebugSessionStart",
  InitializePlugin = "Qsharp.InitializePlugin",
  LoadLanguageService = "Qsharp.LoadLanguageService",
  QSharpJupyterCellInitialized = "Qsharp.JupyterCellInitialized",
  ReturnCompletionList = "Qsharp.ReturnCompletionList",
  GenerateQirStart = "Qsharp.GenerateQirStart",
  GenerateQirEnd = "Qsharp.GenerateQirEnd",
}

export const generateAssociationId: () => string  = v4;

type Empty = { [K in any]: never };

type EventTypes = {
  [EventType.InitializePlugin]: {
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
  [EventType.QSharpJupyterCellInitialized]: {
    properties: Empty;
    measurements: Empty;
  };
  [EventType.ReturnCompletionList]: {
    properties: Empty;
    measurements: {timeToCompletionMs: number; completionListLength: number; };
  };
  [EventType.GenerateQirStart]: {
    properties: {associationId: string};
    measurements: Empty;
  };
  [EventType.GenerateQirEnd]: {
    properties: {associationId: string};
    measurements: Empty;
  };
};

let reporter: TelemetryReporter | undefined;

export function initTelemetry(context: vscode.ExtensionContext) {
  const packageJson = context.extension?.packageJSON;
  if (!packageJson) {
    return;
  }
  reporter = new TelemetryReporter(packageJson.aiKey);

  sendTelemetryEvent(EventType.InitializePlugin, {}, {});
}

export function sendTelemetryEvent<E extends keyof EventTypes>(
  event: E,
  properties: EventTypes[E]["properties"] = {},
  measurements: EventTypes[E]["measurements"] = {}
) {
  if (reporter === undefined) {
    log.trace(`No telemetry reporter. Omitting telemetry event ${event}`);
    return;
  }
  reporter.sendTelemetryEvent(event, properties, measurements);
  log.debug(
    `Sent telemetry: ${event} ${JSON.stringify(properties)} ${JSON.stringify(
      measurements
    )}`
  );
}
