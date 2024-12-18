// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { getCompilerWorker, log } from "qsharp-lang";
import * as vscode from "vscode";
import { getTarget, setTarget } from "./config";
import { invokeAndReportCommandDiagnostics } from "./diagnostics";
import { getActiveProgram } from "./programConfig";
import { EventType, sendTelemetryEvent } from "./telemetry";
import { getRandomGuid } from "./utils";
import { qsharpExtensionId } from "./common";

const generateQirTimeoutMs = 120000;

let compilerWorkerScriptPath: string;

export class QirGenerationError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "QirGenerationError";
  }
}

export async function getQirForActiveWindow(
  supports_adaptive?: boolean, // should be true or false when submitting to Azure, undefined when generating QIR
): Promise<string> {
  let result = "";
  const program = await getActiveProgram();
  if (!program.success) {
    throw new QirGenerationError(program.errorMsg);
  }

  const config = program.programConfig;
  const targetProfile = config.profile;
  const is_unrestricted = targetProfile === "unrestricted";
  const is_base = targetProfile === "base";

  // We differentiate between submission to Azure and on-demand QIR codegen by checking
  // whether a boolean value was passed for `supports_adaptive`. On-demand codegen does not
  // have a target, so support for adaptive is unknown.
  let error_msg =
    supports_adaptive === undefined
      ? "Generating QIR "
      : "Submitting to Azure ";
  if (is_unrestricted) {
    error_msg += "is not supported when using the unrestricted profile.";
  } else if (!is_base && supports_adaptive === false) {
    error_msg +=
      "using the Adaptive_RI profile is not supported for targets that can only accept Base profile QIR.";
  }

  // Check that the current target is base or adaptive_ri profile, and current doc has no errors.
  if (is_unrestricted || (!is_base && supports_adaptive === false)) {
    const result = await vscode.window.showWarningMessage(
      // if supports_adaptive is undefined, use the generic codegen message
      error_msg,
      { modal: true },
      {
        title:
          "Set the QIR target profile to " +
          (supports_adaptive ? "Adaptive_RI" : "Base") +
          " to continue",
        action: "set",
      },
      { title: "Cancel", action: "cancel", isCloseAffordance: true },
    );
    if (result?.action !== "set") {
      throw new QirGenerationError(
        error_msg +
          " Please update the QIR target via the status bar selector or extension settings.",
      );
    } else {
      await setTarget(supports_adaptive ? "adaptive_ri" : "base");
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
      { associationId, targetProfile },
      {},
    );

    // Override the program config with the new target profile (if updated above)
    config.profile = getTarget();

    result = await vscode.window.withProgress(
      {
        location: vscode.ProgressLocation.Notification,
        cancellable: true,
        title: "Q#: Generating QIR",
      },
      async (progress, token) => {
        token.onCancellationRequested(() => {
          worker.terminate();
        });

        const qir = await invokeAndReportCommandDiagnostics(() =>
          worker.getQir(config),
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

export function initCodegen(context: vscode.ExtensionContext) {
  compilerWorkerScriptPath = vscode.Uri.joinPath(
    context.extensionUri,
    "./out/compilerWorker.js",
  ).toString();

  context.subscriptions.push(
    vscode.commands.registerCommand(`${qsharpExtensionId}.getQir`, async () => {
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
    }),
  );
}
