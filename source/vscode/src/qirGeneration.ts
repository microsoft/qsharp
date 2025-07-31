// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { getCompilerWorker, log, TargetProfile } from "qsharp-lang";
import * as vscode from "vscode";
import { invokeAndReportCommandDiagnostics } from "./diagnostics";
import {
  FullProgramConfig,
  getActiveProgram,
  getActiveQdkDocumentUri,
  getVisibleProgram,
  getVisibleQdkDocumentUri,
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
  preferredTargetProfile: TargetProfile,
): Promise<string> {
  const program = await getVisibleProgram({
    targetProfileFallback: preferredTargetProfile,
  });

  if (!program.success) {
    throw new QirGenerationError(program.errorMsg);
  }

  const docUri = getVisibleQdkDocumentUri();
  return getQirForProgram(
    program.programConfig,
    preferredTargetProfile,
    getVisibleDocumentType(),
    docUri,
  );
}

export async function getQirForActiveWindow(
  preferredTargetProfile: TargetProfile,
  isLocalQirGeneration: boolean = false,
): Promise<string> {
  const program = await getActiveProgram({
    showModalError: true,
    targetProfileFallback: preferredTargetProfile,
  });

  if (!program.success) {
    throw new QirGenerationError(program.errorMsg);
  }

  const docUri = getActiveQdkDocumentUri();
  return getQirForProgram(
    program.programConfig,
    preferredTargetProfile,
    getActiveDocumentType(),
    docUri,
    isLocalQirGeneration,
  );
}

function checkCompatibility(
  configuredTargetProfile: TargetProfile,
  preferredTargetProfile: TargetProfile,
) {
  // Trick: since each profile is a superset of
  // the previous one, we can turn this into a check
  // using an array
  const profiles: TargetProfile[] = [
    "base",
    "adaptive_ri",
    "adaptive_rif",
    "unrestricted",
  ];

  return (
    profiles.indexOf(preferredTargetProfile) >=
    profiles.indexOf(configuredTargetProfile)
  );
}

async function getQirForProgram(
  config: FullProgramConfig,
  preferredTargetProfile: TargetProfile,
  telemetryDocumentType: QsharpDocumentType,
  documentUri?: vscode.Uri,
  isLocalQirGeneration = false,
): Promise<string> {
  let result = "";

  const compatible = checkCompatibility(config.profile, preferredTargetProfile);
  if (!compatible) {
    let errorMsg =
      "The current program is configured to use the target profile " +
      config.profile +
      ", which is not compatible with the QIR target profile " +
      preferredTargetProfile +
      " required by " +
      isLocalQirGeneration
        ? "local QIR generation."
        : "the selected target.";

    if (config.packageGraphSources.hasManifest) {
      // Open the manifest file to allow the user to update the profile.
      const docUri =
        documentUri ?? vscode.window.activeTextEditor?.document.uri;
      if (docUri != undefined) {
        try {
          await openManifestFile(docUri);
          errorMsg +=
            " Please update the target profile in the manifest file to " +
            preferredTargetProfile;
        } catch {
          // If the manifest file cannot be opened, just log the error.
          log.error(
            "Could not open qsharp.json manifest to update the QIR target profile.",
          );
        }
      }
    }
    throw new QirGenerationError(errorMsg);
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
          { populateProblemsView: true },
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
    if (e.toString() === "terminated") {
      throw new QirGenerationError(
        "QIR generation was cancelled or timed out.",
      );
    } else {
      throw new QirGenerationError(
        `QIR generation failed due to error: "${e.toString()}". Please ensure the code is compatible with the selected QIR profile.`,
      );
    }
  } finally {
    worker.terminate();
  }

  return result;
}

async function getQirForActiveWindowCommand() {
  try {
    const qir = await getQirForActiveWindow("adaptive_rif", true);
    const qirDoc = await vscode.workspace.openTextDocument({
      language: "llvm",
      content: qir,
    });
    await vscode.window.showTextDocument(qirDoc);
  } catch (e: any) {
    log.error("QIR generation failed. ", e);
    if (e.name === "QirGenerationError") {
      vscode.window.showErrorMessage(e.message, { modal: true });
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
