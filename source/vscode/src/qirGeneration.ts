// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { getCompilerWorker, log } from "qsharp-lang";
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
import { updateManifestProfile, findManifestDocument } from "./projectSystem";

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
  // Use the visible document's URI
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
  // Use the active document's URI
  const docUri = getActiveQdkDocumentUri();
  return getQirForProgram(
    program.programConfig,
    getActiveDocumentType(),
    targetSupportsAdaptive,
    docUri,
  );
}

async function getQirForProgram(
  config: FullProgramConfig,
  telemetryDocumentType: QsharpDocumentType,
  targetSupportsAdaptive?: boolean,
  documentUri?: vscode.Uri,
): Promise<string> {
  let result = "";
  const isLocalQirGeneration = targetSupportsAdaptive === undefined;
  if (config.isSingleFile) {
    // Single-file profile is inferred from the target.
    config.profile =
      isLocalQirGeneration || !targetSupportsAdaptive ? "base" : "adaptive_ri";
  }
  let targetProfile = config.profile;
  const isUnrestricted = targetProfile === "unrestricted";
  const isUnsupportedAdaptiveSubmissionProfile =
    targetProfile === "adaptive_rif";
  const isTargetProfileBase = targetProfile === "base";
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
      "using the Adaptive_RI profile is not supported for targets that can only accept Base profile QIR.";
  } else if (isSubmittingUnsupportedAdaptiveProfile) {
    error_msg +=
      "using the Adaptive_RIF profile is not supported for targets that can only accept Adaptive_RI profile QIR.";
  }

  // Check that the current target is base or adaptive_ri profile, and current doc has no errors.
  if (
    isUnrestricted ||
    isSubmittingAdaptiveToBaseAzureTarget ||
    isSubmittingUnsupportedAdaptiveProfile
  ) {
    // Check if this is a single-file (no manifest) scenario
    let isSingleFile = false;
    if (documentUri) {
      const manifestInfo = await findManifestDocument(documentUri.toString());
      isSingleFile = !manifestInfo;
    }
    if (isSingleFile) {
      throw new QirGenerationError(
        "QIR generation is not supported for single-file (non-project) code. Please open or create a Q# project with a qsharp.json manifest.",
      );
    }
    const title =
      "Set the QIR target profile to " +
      (targetSupportsAdaptive ? "Adaptive_RI" : "Base") +
      " to continue";
    const result = await vscode.window.showWarningMessage(
      // if supports_adaptive is undefined, use the generic codegen message
      error_msg,
      { modal: true },
      {
        title: title,
        action: "set",
      },
      { title: "Cancel", action: "cancel", isCloseAffordance: true },
    );
    if (result?.action !== "set") {
      throw new QirGenerationError(
        error_msg +
          " Please update the QIR target via the qsharp.json to continue.",
      );
    } else {
      // Try to update the qsharp.json manifest automatically.
      try {
        // Use the provided documentUri, fallback to activeTextEditor if not provided
        const docUri =
          documentUri ?? vscode.window.activeTextEditor?.document.uri;
        if (!docUri) {
          vscode.window.showErrorMessage(
            "Could not determine the document URI to update qsharp.json.",
          );
        } else {
          try {
            await updateManifestProfile(
              docUri,
              targetSupportsAdaptive ? "adaptive_ri" : "base",
            );
            vscode.window.showInformationMessage(
              `Updated 'profile' in qsharp.json to '${targetSupportsAdaptive ? "adaptive_ri" : "base"}'.`,
            );
            targetProfile = targetSupportsAdaptive ? "adaptive_ri" : "base";
            config.profile = targetProfile; // Update the config profile to match the new target
          } catch (e: any) {
            vscode.window.showErrorMessage(
              "Failed to update qsharp.json: " + (e?.message ?? e),
            );
            return "";
          }
        }
      } catch (e: any) {
        vscode.window.showErrorMessage(
          "Failed to update qsharp.json: " + e.message,
        );
      }
    }
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
      { associationId, targetProfile, documentType: telemetryDocumentType },
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
