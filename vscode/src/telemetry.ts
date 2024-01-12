import * as vscode from "vscode";
import TelemetryReporter from "@vscode/extension-telemetry";
import { log } from "qsharp-lang";

export enum EventType {
  InitializePlugin = "Qsharp.InitializePlugin",
  LoadLanguageService = "Qsharp.LoadLanguageService",
  ReturnCompletionList = "Qsharp.ReturnCompletionList",
  GenerateQirStart = "Qsharp.GenerateQirStart",
  GenerateQirEnd = "Qsharp.GenerateQirEnd",
  RenderQuantumStateStart = "Qsharp.RenderQuantumStateStart",
  RenderQuantumStateEnd = "Qsharp.RenderQuantumStateEnd",
  SubmitToAzureStart = "Qsharp.SubmitToAzureStart",
  SubmitToAzureEnd = "Qsharp.SubmitToAzureEnd",
  AuthSessionStart = "Qsharp.AuthSessionStart",
  AuthSessionEnd = "Qsharp.AuthSessionEnd",
  QueryWorkspacesStart = "Qsharp.QueryWorkspacesStart",
  QueryWorkspacesEnd = "Qsharp.QueryWorkspacesEnd",
  AzureRequestFailed = "Qsharp.AzureRequestFailed",
  StorageRequestFailed = "Qsharp.StorageRequestFailed",
  GetJobFilesStart = "Qsharp.GetJobFilesStart",
  GetJobFilesEnd = "Qsharp.GetJobFilesEnd",
  QueryWorkspaceStart = "Qsharp.QueryWorkspaceStart",
  QueryWorkspaceEnd = "Qsharp.QueryWorkspaceEnd",
  CheckCorsStart = "Qsharp.CheckCorsStart",
  CheckCorsEnd = "Qsharp.CheckCorsEnd",
  InitializeRuntimeStart = "Qsharp.InitializeRuntimeStart",
  InitializeRuntimeEnd = "Qsharp.InitializeRuntimeEnd",
  DebugSessionEvent = "Qsharp.DebugSessionEvent",
  Launch = "Qsharp.Launch",
  OpenedDocument = "Qsharp.OpenedDocument",
  TriggerResourceEstimation = "Qsharp.TriggerResourceEstimation",
  ResourceEstimationStart = "Qsharp.ResourceEstimationStart",
  ResourceEstimationEnd = "Qsharp.ResourceEstimationEnd",
  TriggerHistogram = "Qsharp.TriggerHistogram",
  HistogramStart = "Qsharp.HistogramStart",
  HistogramEnd = "Qsharp.HistogramEnd",
}

type Empty = { [K in any]: never };

type EventTypes = {
  [EventType.InitializePlugin]: {
    properties: Empty;
    measurements: Empty;
  };
  [EventType.LoadLanguageService]: {
    properties: Empty;
    measurements: {
      timeToStartMs: number;
    };
  };
  [EventType.ReturnCompletionList]: {
    properties: Empty;
    measurements: { timeToCompletionMs: number; completionListLength: number };
  };
  [EventType.GenerateQirStart]: {
    properties: { associationId: string };
    measurements: Empty;
  };
  [EventType.GenerateQirEnd]: {
    properties: { associationId: string };
    measurements: { qirLength: number; timeToCompleteMs: number };
  };
  [EventType.RenderQuantumStateStart]: {
    properties: { associationId: string };
    measurements: Empty;
  };
  [EventType.RenderQuantumStateEnd]: {
    properties: { associationId: string };
    measurements: { timeToCompleteMs: number };
  };
  [EventType.SubmitToAzureStart]: {
    properties: { associationId: string };
    measurements: Empty;
  };
  [EventType.SubmitToAzureEnd]: {
    properties: {
      associationId: string;
      reason?: string;
      flowStatus: UserFlowStatus;
    };
    measurements: { timeToCompleteMs: number };
  };
  [EventType.AuthSessionStart]: {
    properties: { associationId: string };
    measurements: Empty;
  };
  [EventType.AuthSessionEnd]: {
    properties: {
      associationId: string;
      reason?: string;
      flowStatus: UserFlowStatus;
    };
    measurements: { timeToCompleteMs: number };
  };
  [EventType.QueryWorkspacesStart]: {
    properties: { associationId: string };
    measurements: Empty;
  };
  [EventType.QueryWorkspacesEnd]: {
    properties: {
      associationId: string;
      reason?: string;
      flowStatus: UserFlowStatus;
    };
    measurements: { timeToCompleteMs: number };
  };
  [EventType.AzureRequestFailed]: {
    properties: { associationId: string; reason?: string };
    measurements: Empty;
  };
  [EventType.StorageRequestFailed]: {
    properties: { associationId: string; reason?: string };
    measurements: Empty;
  };
  [EventType.GetJobFilesStart]: {
    properties: { associationId: string };
    measurements: Empty;
  };
  [EventType.GetJobFilesEnd]: {
    properties: {
      associationId: string;
      reason?: string;
      flowStatus: UserFlowStatus;
    };
    measurements: { timeToCompleteMs: number };
  };
  [EventType.QueryWorkspaceStart]: {
    properties: { associationId: string };
    measurements: Empty;
  };
  [EventType.QueryWorkspaceEnd]: {
    properties: {
      associationId: string;
      reason?: string;
      flowStatus: UserFlowStatus;
    };
    measurements: { timeToCompleteMs: number };
  };
  [EventType.CheckCorsStart]: {
    properties: { associationId: string };
    measurements: Empty;
  };
  [EventType.CheckCorsEnd]: {
    properties: {
      associationId: string;
      reason?: string;
      flowStatus: UserFlowStatus;
    };
    measurements: { timeToCompleteMs: number };
  };
  [EventType.InitializeRuntimeStart]: {
    properties: { associationId: string };
    measurements: Empty;
  };
  [EventType.InitializeRuntimeEnd]: {
    properties: {
      associationId: string;
      reason?: string;
      flowStatus: UserFlowStatus;
    };
    measurements: { timeToCompleteMs: number };
  };
  [EventType.DebugSessionEvent]: {
    properties: {
      associationId: string;
      event: DebugEvent;
    };
    measurements: Empty;
  };
  [EventType.Launch]: {
    properties: { associationId: string };
    measurements: Empty;
  };
  [EventType.OpenedDocument]: {
    properties: { documentType: QsharpDocumentType };
    measurements: { linesOfCode: number };
  };
  [EventType.TriggerResourceEstimation]: {
    properties: { associationId: string };
    measurements: Empty;
  };
  [EventType.ResourceEstimationStart]: {
    properties: { associationId: string };
    measurements: Empty;
  };
  [EventType.ResourceEstimationEnd]: {
    properties: { associationId: string };
    measurements: { timeToCompleteMs: number };
  };
  [EventType.TriggerHistogram]: {
    properties: { associationId: string };
    measurements: Empty;
  };
  [EventType.HistogramStart]: {
    properties: { associationId: string };
    measurements: Empty;
  };
  [EventType.HistogramEnd]: {
    properties: { associationId: string };
    measurements: { timeToCompleteMs: number };
  };
};

export enum QsharpDocumentType {
  JupyterCell = "JupyterCell",
  Qsharp = "Qsharp",
  Other = "Other",
}

export enum UserFlowStatus {
  // "Aborted" means the flow was intentionally canceled or left, either by us or the user
  Aborted = "Aborted",
  Succeeded = "Succeeded",
  // "CompletedWithFailure" means something that we can action -- service request failure, exceptions, etc.
  Failed = "Failed",
}

export enum DebugEvent {
  StepIn = "StepIn",
  Continue = "Continue",
}

let reporter: TelemetryReporter | undefined;

export function initTelemetry(context: vscode.ExtensionContext) {
  const packageJson = context.extension?.packageJSON;
  if (!packageJson) {
    return;
  }
  log.error("here", process.platform);
  log.error( JSON.stringify(process));
  reporter = new TelemetryReporter(packageJson.aiKey);

  sendTelemetryEvent(EventType.InitializePlugin, {}, {});
}

export function sendTelemetryEvent<E extends keyof EventTypes>(
  event: E,
  properties: EventTypes[E]["properties"] = {},
  measurements: EventTypes[E]["measurements"] = {},
) {
  if (reporter === undefined) {
    log.trace(`No telemetry reporter. Omitting telemetry event ${event}`);
    return;
  }
  reporter.sendTelemetryEvent(event, properties, measurements);
  log.debug(
    `Sent telemetry: ${event} ${JSON.stringify(properties)} ${JSON.stringify(
      measurements,
    )}`,
  );
}

interface NavigatorUA {
    readonly userAgentData?: NavigatorUAData;
}

// https://wicg.github.io/ua-client-hints/#dictdef-navigatoruabrandversion
interface NavigatorUABrandVersion {
    readonly brand: string;
    readonly version: string;
}

// https://wicg.github.io/ua-client-hints/#dictdef-uadatavalues
interface UADataValues {
    readonly brands?: NavigatorUABrandVersion[];
    readonly mobile?: boolean;
    readonly platform?: string;
    readonly architecture?: string;
    readonly bitness?: string;
    readonly formFactor?: string[];
    readonly model?: string;
    readonly platformVersion?: string;
    /** @deprecated in favour of fullVersionList */
    readonly uaFullVersion?: string;
    readonly fullVersionList?: NavigatorUABrandVersion[];
    readonly wow64?: boolean;
}

// https://wicg.github.io/ua-client-hints/#dictdef-ualowentropyjson
interface UALowEntropyJSON {
    readonly brands: NavigatorUABrandVersion[];
    readonly mobile: boolean;
    readonly platform: string;
}

// https://wicg.github.io/ua-client-hints/#navigatoruadata
interface NavigatorUAData extends UALowEntropyJSON {
    getHighEntropyValues(hints: string[]): Promise<UADataValues>;
    toJSON(): UALowEntropyJSON;
}

function getBrowserRelease(): string {
  const navigatorUa: NavigatorUA = navigator as NavigatorUA;
	if (navigatorUa.userAgentData) {
		const browser = navigatorUa.userAgentData.brands[navigatorUa.userAgentData.brands.length - 1];
    return `${browser.brand}/${browser.version}`
	} else {
    return ""
  }
}

export function getUserAgent(context: vscode.ExtensionContext): string {
  let version = context.extension?.packageJSON?.version;
  let browserAndRelease = getBrowserRelease();
  return `VSCode/${version} ${browserAndRelease}`;
}

