// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { getCompilerWorker, log, TargetProfile } from "qsharp-lang";
import * as vscode from "vscode";

import { invokeAndReportCommandDiagnostics } from "./diagnostics";
import {
  FullProgramConfig,
  getActiveProgram,
  getVisibleProgram,
  getVisibleQdkDocumentUri,
  getActiveQdkDocumentUri,
} from "./programConfig";
import {
  EventType,
  getActiveDocumentType,
  getVisibleDocumentType,
  QsharpDocumentType,
  sendTelemetryEvent,
} from "./telemetry";
import { getRandomGuid } from "./utils";
import { qsharpExtensionId } from "./common";
import { openManifestFile } from "./projectSystem";

const generateQirTimeoutMs = 120000;

let compilerWorkerScriptPath: string;

export class QirGenerationError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "QirGenerationError";
  }
}

export async function getQirForVisibleSource(
  targetSupportsAdaptive?: boolean, // should be true or false when submitting to Azure, undefined when generating QIR
): Promise<string> {
  const program = await getVisibleProgram();
  if (!program.success) {
    throw new QirGenerationError(program.errorMsg);
  }
  const docUri = getVisibleQdkDocumentUri();
  return getQirForProgram(
    program.programConfig,
    getVisibleDocumentType(),
    targetSupportsAdaptive,
    docUri,
  );
}

export async function getQirForActiveWindow(
  targetSupportsAdaptive?: boolean, // should be true or false when submitting to Azure, undefined when generating QIR
): Promise<string> {
  const program = await getActiveProgram({ showModalError: true });
  if (!program.success) {
    throw new QirGenerationError(program.errorMsg);
  }
  const docUri = getActiveQdkDocumentUri();
  return getQirForProgram(
    program.programConfig,
    getActiveDocumentType(),
    targetSupportsAdaptive,
    docUri,
  );
}

const TargetProfileValues: TargetProfile[] = [
  "base",
  "adaptive_ri",
  "adaptive_rif",
  "unrestricted",
];

function isValidProfile(value: string): value is TargetProfile {
  return TargetProfileValues.includes(value as TargetProfile);
}

async function getQirForProgram(
  config: FullProgramConfig,
  telemetryDocumentType: QsharpDocumentType,
  targetSupportsAdaptive?: boolean,
  documentUri?: vscode.Uri,
): Promise<string> {
  let result = "";
  const isLocalQirGeneration = targetSupportsAdaptive === undefined;
  const hasManifest = config.packageGraphSources.hasManifest;
  if (!hasManifest) {
    let profile = "";
    const worker = getCompilerWorker(compilerWorkerScriptPath);
    const compilerTimeout = setTimeout(() => {
      worker.terminate();
    }, generateQirTimeoutMs);
    try {
      profile = await invokeAndReportCommandDiagnostics(
        () => worker.getEntryPointProfile(config),
        { populateProblemsView: true, showModalError: true },
      );
      clearTimeout(compilerTimeout);
    } catch (e: any) {
      log.error("Codegen error. ", e.toString());
      if (e.toString() === "terminated") {
        throw new QirGenerationError(
          "QIR generation was cancelled or timed out.",
        );
      } else {
        throw new QirGenerationError(
          `QIR generation failed due to error: "${e.toString()}". Please ensure the code is compatible with a QIR profile ` +
            "by setting the target QIR profile to 'base' or 'adaptive_ri' and fixing any errors.",
        );
      }
    } finally {
      worker.terminate();
    }

    if (isValidProfile(profile)) {
      config.profile = profile as TargetProfile;
    }
  }

  if (config.profile === undefined) {
    config.profile = isLocalQirGeneration
      ? "adaptive_rif"
      : targetSupportsAdaptive
        ? "adaptive_ri"
        : "base";
  }

  const isUnrestricted = config.profile === "unrestricted";
  const isUnsupportedAdaptiveSubmissionProfile =
    config.profile === "adaptive_rif";
  const isTargetProfileBase = config.profile === "base";
  const isSubmittingAdaptiveToBaseAzureTarget =
    !isTargetProfileBase && targetSupportsAdaptive === false;
  const isSubmittingUnsupportedAdaptiveProfile =
    isUnsupportedAdaptiveSubmissionProfile && !isLocalQirGeneration;

  // We differentiate between submission to Azure and on-demand QIR codegen by checking
  // whether a boolean value was passed for `supports_adaptive`. On-demand codegen does not
  // have a target, so support for adaptive is unknown.
  let error_msg = isLocalQirGeneration
    ? "Generating QIR "
    : "Submitting to Azure ";
  if (isUnrestricted) {
    error_msg += "is not supported when using the unrestricted profile.";
  } else if (isSubmittingAdaptiveToBaseAzureTarget) {
    error_msg +=
      "using the Adaptive_RI or Adaptive_RIF profiles is not supported for targets that can only accept Base profile QIR.";
  } else if (isSubmittingUnsupportedAdaptiveProfile) {
    error_msg +=
      "using the Adaptive_RIF profile is not supported for targets that can only accept Adaptive_RI profile QIR.";
  }
  if (hasManifest) {
    error_msg += " Please update the QIR target via the qsharp.json.";
  }

  // Check that the current target is base or adaptive_ri profile, and current doc has no errors.
  if (
    isUnrestricted ||
    isSubmittingAdaptiveToBaseAzureTarget ||
    isSubmittingUnsupportedAdaptiveProfile
  ) {
    await vscode.window.showErrorMessage(
      // if supports_adaptive is undefined, use the generic codegen message
      error_msg,
      { modal: true },
      { title: "Okay", isCloseAffordance: true },
    );
    if (hasManifest) {
      // Open the manifest file to allow the user to update the profile.
      const docUri =
        documentUri ?? vscode.window.activeTextEditor?.document.uri;
      if (docUri != undefined) {
        try {
          await openManifestFile(docUri);
        } catch {
          // If the manifest file cannot be opened, just log the error.
          log.error(
            "Could not open qsharp.json manifest to update the QIR target profile.",
          );
        }
      }
    }
    throw new QirGenerationError(error_msg);
  }

  // Create a temporary worker just to get the QIR, as it may loop/panic during codegen.
  // Let it run for max 10 seconds, then terminate it if not complete.
  const worker = getCompilerWorker(compilerWorkerScriptPath);
  const compilerTimeout = setTimeout(() => {
    worker.terminate();
  }, generateQirTimeoutMs);
  try {
    const associationId = getRandomGuid();
    const start = performance.now();
    sendTelemetryEvent(
      EventType.GenerateQirStart,
      {
        associationId,
        targetProfile: config.profile,
        documentType: telemetryDocumentType,
      },
      {},
    );

    result = await vscode.window.withProgress(
      {
        location: vscode.ProgressLocation.Notification,
        cancellable: true,
        title: "Generating QIR",
      },
      async (progress, token) => {
        token.onCancellationRequested(() => {
          worker.terminate();
        });

        const qir = await invokeAndReportCommandDiagnostics(
          () => worker.getQir(config),
          { populateProblemsView: true, showModalError: true },
        );
        progress.report({ increment: 100 });
        return qir;
      },
    );

    sendTelemetryEvent(
      EventType.GenerateQirEnd,
      { associationId },
      { qirLength: result.length, timeToCompleteMs: performance.now() - start },
    );
    clearTimeout(compilerTimeout);
  } catch (e: any) {
    log.error("Codegen error. ", e.toString());
    if (e.toString() === "terminated") {
      throw new QirGenerationError(
        "QIR generation was cancelled or timed out.",
      );
    } else {
      throw new QirGenerationError(
        `QIR generation failed due to error: "${e.toString()}". Please ensure the code is compatible with a QIR profile ` +
          "by setting the target QIR profile to 'base' or 'adaptive_ri' and fixing any errors.",
      );
    }
  } finally {
    worker.terminate();
  }

  return result;
}

async function getQirForActiveWindowCommand() {
  try {
    const qir = await getQirForActiveWindow();
    const qirDoc = await vscode.workspace.openTextDocument({
      language: "llvm",
      content: qir,
    });
    await vscode.window.showTextDocument(qirDoc);
  } catch (e: any) {
    log.error("QIR generation failed. ", e);
    if (e.name === "QirGenerationError") {
      vscode.window.showErrorMessage(e.message);
    }
  }
}

export function initCodegen(context: vscode.ExtensionContext) {
  compilerWorkerScriptPath = vscode.Uri.joinPath(
    context.extensionUri,
    "./out/compilerWorker.js",
  ).toString();

  context.subscriptions.push(
    vscode.commands.registerCommand(
      `${qsharpExtensionId}.getQir`,
      getQirForActiveWindowCommand,
    ),
  );
}
