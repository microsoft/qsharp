// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { type Circuit as CircuitData } from "@microsoft/quantum-viz.js/lib";
import { escapeHtml } from "markdown-it/lib/common/utils";
import {
  ICompilerWorker,
  IOperationInfo,
  IRange,
  ProgramConfig,
  TargetProfile,
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

/**
 * Input parameters for generating a circuit.
 */
type CircuitParams = {
  program: ProgramConfig;
  targetProfile: TargetProfile;
  operation?: IOperationInfo;
};

/**
 * Result of a circuit generation attempt.
 */
type CircuitOrError = {
  simulated: boolean;
} & (
  | {
      result: "success";
      circuit: CircuitData;
    }
  | {
      result: "error";
      errors: {
        document: string;
        diag: VSDiagnostic;
        stack: string;
      }[];
      hasResultComparisonError: boolean;
      timeout: boolean;
    }
);

export async function showCircuitCommand(
  extensionUri: Uri,
  operation: IOperationInfo | undefined,
) {
  const associationId = getRandomGuid();
  sendTelemetryEvent(EventType.TriggerCircuit, { associationId }, {});

  const editor = window.activeTextEditor;
  if (!editor || !isQsharpDocument(editor.document)) {
    throw new Error("The currently active window is not a Q# file");
  }

  const docUri = editor.document.uri;
  const program = await loadProject(docUri);
  const targetProfile = getTarget();

  sendTelemetryEvent(
    EventType.CircuitStart,
    {
      associationId,
      targetProfile,
      isOperation: (!!operation).toString(),
    },
    {},
  );

  // Generate the circuit and update the panel.
  // generateCircuits() takes care of handling timeouts and
  // falling back to the simulator for dynamic circuits.
  const result = await generateCircuit(extensionUri, docUri, {
    program: program,
    targetProfile,
    operation,
  });

  if (result.result === "success") {
    sendTelemetryEvent(EventType.CircuitEnd, {
      simulated: result.simulated.toString(),
      associationId,
      flowStatus: UserFlowStatus.Succeeded,
    });
  } else {
    if (result.timeout) {
      sendTelemetryEvent(EventType.CircuitEnd, {
        simulated: result.simulated.toString(),
        associationId,
        reason: "timeout",
        flowStatus: UserFlowStatus.Aborted,
      });
    } else {
      const reason =
        result.errors.length > 0 ? result.errors[0].diag.code : "unknown";

      sendTelemetryEvent(EventType.CircuitEnd, {
        simulated: result.simulated.toString(),
        associationId,
        reason,
        flowStatus: UserFlowStatus.Failed,
      });
    }
  }
}

/**
 * Generate the circuit and update the panel with the results.
 * We first attempt to generate a circuit without running the simulator,
 * which should be fast.
 *
 * If that fails, specifically due to a result comparison error,
 * that means this is a dynamic circuit. We fall back to using the
 * simulator in this case ("trace" mode), which is slower.
 */
async function generateCircuit(
  extensionUri: Uri,
  docUri: Uri,
  params: CircuitParams,
): Promise<CircuitOrError> {
  const programPath = docUri.path;

  // Before we start, reveal the panel with the "calculating" spinner
  updateCircuitPanel(
    params.targetProfile,
    programPath,
    true, // reveal
    { operation: params.operation, calculating: true },
  );

  // First, try without simulating
  let result = await getCircuitOrErrorWithTimeout(
    extensionUri,
    params,
    false, // simulate
  );

  if (result.result === "error" && result.hasResultComparisonError) {
    // Retry with the simulator if circuit generation failed because
    // there was a result comparison (i.e. if this is a dynamic circuit)

    updateCircuitPanel(
      params.targetProfile,
      programPath,
      false, // reveal
      {
        operation: params.operation,
        calculating: true,
        simulated: true,
      },
    );

    // try again with the simulator
    result = await getCircuitOrErrorWithTimeout(
      extensionUri,
      params,
      true, // simulate
    );
  }

  // Update the panel with the results

  if (result.result === "success") {
    updateCircuitPanel(
      params.targetProfile,
      programPath,
      false, // reveal
      {
        circuit: result.circuit,
        operation: params.operation,
        simulated: result.simulated,
      },
    );
  } else {
    log.error("Circuit error. ", result);
    let errorHtml = "There was an error generating the circuit.";
    if (result.errors.length > 0) {
      errorHtml = errorsToHtml(result.errors);
    } else if (result.timeout) {
      errorHtml = `The circuit generation exceeded the timeout of ${compilerRunTimeoutMs}ms.`;
    }

    updateCircuitPanel(
      params.targetProfile,
      programPath,
      false, // reveal
      {
        errorHtml,
        operation: params.operation,
        simulated: result.simulated,
      },
    );
  }

  return result;
}

/**
 * Wrapper around getCircuit() that enforces a timeout.
 * Won't throw for known errors.
 */
async function getCircuitOrErrorWithTimeout(
  extensionUri: Uri,
  params: CircuitParams,
  simulate: boolean,
): Promise<CircuitOrError> {
  let timeout = false;

  const compilerWorkerScriptPath = Uri.joinPath(
    extensionUri,
    "./out/compilerWorker.js",
  ).toString();

  const worker = getCompilerWorker(compilerWorkerScriptPath);
  const compilerTimeout = setTimeout(() => {
    timeout = true;
    log.info("terminating circuit worker due to timeout");
    worker.terminate();
  }, compilerRunTimeoutMs);

  const result = await getCircuitOrError(worker, params, simulate);
  clearTimeout(compilerTimeout);

  if (result.result === "error") {
    return {
      ...result,
      timeout,
    };
  } else {
    return result;
  }
}

/**
 * Wrapper around compiler getCircuit() that handles exceptions
 * and converts to strongly typed error object.
 * Won't throw for known errors.
 */
async function getCircuitOrError(
  worker: ICompilerWorker,
  params: CircuitParams,
  simulate: boolean,
): Promise<CircuitOrError> {
  try {
    const circuit = await worker.getCircuit(
      params.program,
      params.targetProfile,
      simulate,
      params.operation,
    );
    return { result: "success", simulated: simulate, circuit };
  } catch (e: any) {
    let errors: { document: string; diag: VSDiagnostic; stack: string }[] = [];
    let resultCompError = false;
    if (typeof e === "string") {
      try {
        const rawErrors: [string, VSDiagnostic, string][] = JSON.parse(e);
        errors = rawErrors.map(([document, diag, stack]) => ({
          document,
          diag,
          stack,
        }));
        resultCompError = hasResultComparisonError(e);
      } catch (e) {
        // couldn't parse the error - would indicate a bug.
        // will get reported up the stack as a generic error
      }
    }
    return {
      result: "error",
      simulated: simulate,
      errors,
      hasResultComparisonError: resultCompError,
      timeout: false,
    };
  }
}

function hasResultComparisonError(e: unknown) {
  const errors: [string, VSDiagnostic, string][] =
    typeof e === "string" ? JSON.parse(e) : undefined;
  const hasResultComparisonError =
    errors &&
    errors.findIndex(
      ([, diag]) => diag.code === "Qsc.Eval.ResultComparisonUnsupported",
    ) >= 0;
  return hasResultComparisonError;
}

/**
 * Formats an array of compiler/runtime errors into HTML to be presented to the user.
 *
 * @param errors The list of errors to format.
 * @returns The HTML formatted errors, to be set as the inner contents of a container element.
 */
function errorsToHtml(
  errors: { document: string; diag: VSDiagnostic; stack: string }[],
) {
  let errorHtml = "";
  for (const error of errors) {
    const { document, diag, stack: rawStack } = error;

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

export function updateCircuitPanel(
  targetProfile: string,
  programPath: string,
  reveal: boolean,
  params: {
    circuit?: CircuitData;
    errorHtml?: string;
    simulated?: boolean;
    operation?: IOperationInfo | undefined;
    calculating?: boolean;
  },
) {
  const title = params?.operation
    ? `${params.operation.operation} with ${params.operation.totalNumQubits} input qubits`
    : basename(programPath) || "Circuit";

  // Trim the Q#: prefix from the target profile name - that's meant for the ui text in the status bar
  const target = `Target profile: ${getTargetFriendlyName(targetProfile).replace("Q#: ", "")} `;

  const props = {
    title,
    targetProfile: target,
    simulated: params?.simulated || false,
    calculating: params?.calculating || false,
    circuit: params?.circuit,
    errorHtml: params?.errorHtml,
  };

  const message = {
    command: "circuit",
    props,
  };
  sendMessageToPanel("circuit", reveal, message);
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
      ? escapeHtml(`:${range.start.line + 1}:${range.start.character + 1}`)
      : "";
    location = `<a href="${openCommandUri}">${fsPath}</a>${lineColumn}`;
  } catch (e) {
    // Likely could not parse document URI - it must be a project level error
    // or an error from stdlib, use the document name directly
    location = escapeHtml(maybeUri);
  }

  return location;
}
