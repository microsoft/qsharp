// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { type Circuit as CircuitData } from "@microsoft/quantum-viz.js/lib/circuit.js";
import {
  IOperationInfo,
  IRange,
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
      true, // reveal
      operation,
    );

    sendTelemetryEvent(EventType.CircuitEnd, {
      associationId,
      flowStatus: UserFlowStatus.Succeeded,
    });
  } catch (e: any) {
    log.error("Circuit error. ", e.toString());
    clearTimeout(compilerTimeout);

    const errors: [string, VSDiagnostic, string][] =
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
      false, // reveal
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
  reveal: boolean,
  operation?: IOperationInfo | undefined,
) {
  let title;
  let subtitle;
  if (operation) {
    title = `${operation.operation} with ${operation.totalNumQubits} input qubits`;
    subtitle = `Target profile: ${getTargetFriendlyName(targetProfile)} `;
  } else {
    title = basename(docPath) || "Circuit";
    subtitle = `Target profile: ${getTargetFriendlyName(targetProfile)}`;
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
  sendMessageToPanel("circuit", reveal, message);
}

/**
 * Formats an array of compiler/runtime errors into HTML to be presented to the user.
 *
 * @param errors
 *  The first string is the document URI or "<project>" if the error isn't associated with a specific document.
 *  The VSDiagnostic is the error information.
 *  The last string is the stack trace.
 *
 * @returns The HTML formatted errors, to be set as the inner contents of a container element.
 */
function errorsToHtml(
  errors: [string, VSDiagnostic, string | undefined][],
): string {
  let errorHtml = "";
  for (const error of errors) {
    const [document, diag, rawStack] = error;

    const location = documentHtml(document, diag.range);

    const message = escapeHtml(`(${diag.code}) ${diag.message}`).replace(
      "\n",
      "<br/><br/>",
    );

    errorHtml += `<p>${location}: ${message}<br/></p>`;

    if (rawStack) {
      const stack = rawStack
        .split("\n")
        .map((l) => {
          // Link-ify the document names in the stack trace
          const match = l.match(/^(\s*)at (.*) in (.*)/);
          if (match) {
            const [, leadingWs, callable, doc] = match;
            return `${leadingWs}at ${escapeHtml(callable)} in ${documentHtml(doc)}`;
          } else {
            return l;
          }
        })

        .join("\n");
      errorHtml += `<br/><pre>${stack}</pre>`;
    }
  }
  return errorHtml;
}

/**
 * If the input is a URI, turns it into a document open link.
 * Otherwise returns the HTML-escaped input
 */
function documentHtml(maybeUri: string, range?: IRange) {
  let location;
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
    // location. Then this would be a link to that command instead. Yet another
    // alternative is to have the webview pass a message back to the extension.
    const uri = Uri.parse(maybeUri, true);
    const openCommandUri = Uri.parse(
      `command:vscode.open?${encodeURIComponent(JSON.stringify([uri]))}`,
      true,
    );
    const fsPath = escapeHtml(uri.fsPath);
    const lineColumn = range
      ? escapeHtml(`:${range.start.line}:${range.start.character}`)
      : "";
    location = `<a href="${openCommandUri}">${fsPath}</a>${lineColumn}`;
  } catch (e) {
    // Likely could not parse document URI - it must be a project level error
    // or an error from stdlib, use the document name directly
    location = escapeHtml(maybeUri);
  }

  return location;
}

function escapeHtml(unsafe: string): string {
  return unsafe
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#039;");
}
