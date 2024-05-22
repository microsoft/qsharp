// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as vscode from "vscode";
import { getCompilerWorker, log, ProgramConfig } from "qsharp-lang";
import { isQsharpDocument } from "./common";
import { EventType, sendTelemetryEvent } from "./telemetry";
import { getRandomGuid } from "./utils";
import { getTarget, setTarget } from "./config";
import { loadProject } from "./projectSystem";

const generateQirTimeoutMs = 30000;

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
  const editor = vscode.window.activeTextEditor;

  if (!editor || !isQsharpDocument(editor.document)) {
    throw new QirGenerationError(
      "The currently active window is not a Q# file",
    );
  }

  const targetProfile = getTarget();
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
  let sources: [string, string][] = [];
  let languageFeatures: string[] = [];
  try {
    const result = await loadProject(editor.document.uri);
    sources = result.sources;
    languageFeatures = result.languageFeatures || [];
  } catch (e: any) {
    throw new QirGenerationError(e.message);
  }
  for (const source of sources) {
    const diagnostics = vscode.languages.getDiagnostics(
      vscode.Uri.parse(source[0]),
    );
    if (diagnostics?.length > 0) {
      throw new QirGenerationError(
        "The current program contains errors that must be fixed before submitting to Azure",
      );
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
    sendTelemetryEvent(EventType.GenerateQirStart, { associationId }, {});

    const config = {
      sources,
      languageFeatures,
      profile: getTarget(),
    } as ProgramConfig;
    result = await worker.getQir(config);

    sendTelemetryEvent(
      EventType.GenerateQirEnd,
      { associationId },
      { qirLength: result.length, timeToCompleteMs: performance.now() - start },
    );
    clearTimeout(compilerTimeout);
  } catch (e: any) {
    log.error("Codegen error. ", e.toString());
    throw new QirGenerationError(
      `Code generation failed due to error: "${e.toString()}". Please ensure the code is compatible with a QIR profile ` +
        "by setting the target QIR profile to 'base' or 'adaptive_ri' and fixing any errors.",
    );
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
    vscode.commands.registerCommand("qsharp-vscode.getQir", async () => {
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
