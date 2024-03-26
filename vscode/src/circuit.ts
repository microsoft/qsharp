// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { type Circuit as CircuitData } from "@microsoft/quantum-viz.js/lib/circuit.js";
import {
  IOperationInfo,
  VSDiagnostic,
  getCompilerWorker,
  log,
} from "qsharp-lang";
import { Uri, window } from "vscode";
import { basename, isQsharpDocument } from "./common";
import { getTarget, getTargetFriendlyName } from "./config";
import { loadProject } from "./projectSystem";
import { EventType, UserFlowStatus, sendTelemetryEvent } from "./telemetry";
import { getRandomGuid } from "./utils";
import { sendMessageToPanel } from "./webviewPanel";

const compilerRunTimeoutMs = 1000 * 60 * 5; // 5 minutes

export async function showCircuitCommand(
  extensionUri: Uri,
  operation: IOperationInfo | undefined,
) {
  const associationId = getRandomGuid();
  sendTelemetryEvent(EventType.TriggerCircuit, { associationId }, {});

  const compilerWorkerScriptPath = Uri.joinPath(
    extensionUri,
    "./out/compilerWorker.js",
  ).toString();

  const editor = window.activeTextEditor;
  if (!editor || !isQsharpDocument(editor.document)) {
    throw new Error("The currently active window is not a Q# file");
  }

  sendMessageToPanel("circuit", true, undefined);

  let timeout = false;

  // Start the worker, run the code, and send the results to the webview
  const worker = getCompilerWorker(compilerWorkerScriptPath);
  const compilerTimeout = setTimeout(() => {
    timeout = true;
    sendTelemetryEvent(EventType.CircuitEnd, {
      associationId,
      reason: "timeout",
      flowStatus: UserFlowStatus.Aborted,
    });
    log.info("terminating circuit worker due to timeout");
    worker.terminate();
  }, compilerRunTimeoutMs);
  const sources = await loadProject(editor.document.uri);
  const targetProfile = getTarget();

  try {
    sendTelemetryEvent(EventType.CircuitStart, { associationId }, {});

    const circuit = await worker.getCircuit(sources, targetProfile, operation);
    clearTimeout(compilerTimeout);

    updateCircuitPanel(
      targetProfile,
      editor.document.uri.path,
      circuit,
      operation,
    );

    sendTelemetryEvent(EventType.CircuitEnd, {
      associationId,
      flowStatus: UserFlowStatus.Succeeded,
    });
  } catch (e: any) {
    log.error("Circuit error. ", e.toString());
    clearTimeout(compilerTimeout);

    const errors: [string, VSDiagnostic][] =
      typeof e === "string" ? JSON.parse(e) : undefined;
    let errorHtml = "There was an error generating the circuit.";
    if (errors) {
      errorHtml = errorsToHtml(errors);
    }

    if (!timeout) {
      sendTelemetryEvent(EventType.CircuitEnd, {
        associationId,
        reason: errors && errors[0] ? errors[0][1].code : undefined,
        flowStatus: UserFlowStatus.Failed,
      });
    }

    updateCircuitPanel(
      targetProfile,
      editor.document.uri.path,
      errorHtml,
      operation,
    );
  } finally {
    log.info("terminating circuit worker");
    worker.terminate();
  }
}

export function updateCircuitPanel(
  targetProfile: string,
  docPath: string,
  circuitOrErrorHtml: CircuitData | string,
  operation?: IOperationInfo | undefined,
) {
  let title;
  let subtitle;
  if (operation) {
    title = `${operation.operation} with ${operation.totalNumQubits} input qubits`;
    subtitle = `${getTargetFriendlyName(targetProfile)} `;
  } else {
    title = basename(docPath) || "Circuit";
    subtitle = `${getTargetFriendlyName(targetProfile)}`;
  }

  const message = {
    command: "circuit",
    title,
    subtitle,
    circuit:
      typeof circuitOrErrorHtml === "object" ? circuitOrErrorHtml : undefined,
    errorHtml:
      typeof circuitOrErrorHtml === "string" ? circuitOrErrorHtml : undefined,
  };
  sendMessageToPanel("circuit", false, message);
}

/**
 * Formats an array of compiler/runtime errors into HTML to be presented to the user.
 *
 * @param {[string, VSDiagnostic][]} errors
 *  The string is the document URI or "<project>" if the error isn't associated with a specific document.
 *  The VSDiagnostic is the error information.
 *
 * @returns {string} - The HTML formatted errors, to be set as the inner contents of a container element.
 */
function errorsToHtml(errors: [string, VSDiagnostic][]): string {
  let errorHtml = "";
  for (const error of errors) {
    let location;
    const document = error[0];
    try {
      // If the error location is a document URI, create a link to that document.
      // We use the `vscode.open` command (https://code.visualstudio.com/api/references/commands#commands)
      // to open the document in the editor.
      // The line and column information is displayed, but are not part of the link.
      //
      // At the time of writing this is the only way we know to create a direct
      // link to a Q# document from a Web View.
      //
      // If we wanted to handle line/column information from the link, an alternate
      // implementation might be having our own command that navigates to the correct
      // location. Then this would be a link to that command instead.
      const uri = Uri.parse(document, true);
      const openCommandUri = Uri.parse(
        `command:vscode.open?${encodeURIComponent(JSON.stringify([uri]))}`,
        true,
      );
      const fsPath = escapeHtml(uri.fsPath);
      const lineColumn = escapeHtml(
        `:${error[1].range.start.line}:${error[1].range.start.character}`,
      );
      location = `<a href="${openCommandUri}">${fsPath}</a>${lineColumn}`;
    } catch (e) {
      // Likely could not parse document URI - it must be a project level error,
      // use the document name directly
      location = escapeHtml(error[0]);
    }

    const message = escapeHtml(
      `(${error[1].code}) ${error[1].message}`,
    ).replace("\n", "<br/>");

    errorHtml += `${location}: ${message}<br/>`;
  }
  return errorHtml;
}

function escapeHtml(unsafe: string): string {
  return unsafe
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#039;");
}
