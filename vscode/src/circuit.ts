// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { getCompilerWorker, log } from "qsharp-lang";
import { Uri, window } from "vscode";
import { basename, isQsharpDocument } from "./common";
import { loadProject } from "./projectSystem";
import type { IOperationInfo, VSDiagnostic } from "../../npm/lib/web/qsc_wasm";
import { getTarget, getTargetFriendlyName } from "./config";
import { sendMessageToPanel } from "./webviewPanel";

const compilerRunTimeoutMs = 1000 * 60 * 5; // 5 minutes

export async function showCircuitCommand(
  extensionUri: Uri,
  operation: IOperationInfo | undefined,
) {
  const compilerWorkerScriptPath = Uri.joinPath(
    extensionUri,
    "./out/compilerWorker.js",
  ).toString();

  const editor = window.activeTextEditor;
  if (!editor || !isQsharpDocument(editor.document)) {
    throw new Error("The currently active window is not a Q# file");
  }

  sendMessageToPanel("circuit", true, undefined);

  // Start the worker, run the code, and send the results to the webview
  const worker = getCompilerWorker(compilerWorkerScriptPath);
  const compilerTimeout = setTimeout(() => {
    log.info("terminating circuit worker due to timeout");
    worker.terminate();
  }, compilerRunTimeoutMs);
  let title;
  let subtitle;
  const targetProfile = getTarget();
  const sources = await loadProject(editor.document.uri);
  if (operation) {
    title = `${operation.operation} with ${operation.totalNumQubits} input qubits`;
    subtitle = `${getTargetFriendlyName(targetProfile)} `;
  } else {
    title = basename(editor.document.uri.path) || "Circuit";
    subtitle = `${getTargetFriendlyName(targetProfile)}`;
  }

  try {
    const circuit = await worker.getCircuit(sources, targetProfile, operation);
    clearTimeout(compilerTimeout);

    const message = {
      command: "circuit",
      circuit,
      title,
      subtitle,
    };
    sendMessageToPanel("circuit", false, message);
  } catch (e: any) {
    log.error("Circuit error. ", e.toString());
    clearTimeout(compilerTimeout);

    const errors: [string, VSDiagnostic][] =
      typeof e === "string" ? JSON.parse(e) : undefined;
    let errorHtml = "There was an error generating the circuit.";
    if (errors) {
      errorHtml = errorsToHtml(errors);
    }

    const message = {
      command: "circuit",
      title,
      subtitle,
      errorHtml,
    };
    sendMessageToPanel("circuit", false, message);
  } finally {
    log.info("terminating circuit worker");
    worker.terminate();
  }
}

function errorsToHtml(errors: [string, VSDiagnostic][]): string {
  let errorHtml = "";
  for (const error of errors) {
    const uri = Uri.parse(error[0]);
    const openCommandUri = Uri.parse(
      `command:vscode.open?${encodeURIComponent(JSON.stringify([uri]))}`,
    );

    errorHtml += `<a href="${openCommandUri}">${uri.fsPath}</a>:${error[1].range.start.line}:${error[1].range.start.character}: ${error[1].message} (${error[1].code})<br/>`;
  }
  return errorHtml;
}
