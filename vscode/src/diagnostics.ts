// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { IQSharpError, log, qsharpLibraryUriScheme } from "qsharp-lang";
import * as vscode from "vscode";
import {
  qsharpExtensionId,
  qsharpLanguageId,
  toVsCodeDiagnostic,
} from "./common.js";

/**
 * Initialize diagnostics for `qsharp.json` files and failures
 * that get reported from various Q# commands.
 *
 * These are distinct from the errors reported by the Q# language
 * service, (a.k.a. compiler errors that get reported as you type).
 * Those are initialized in `language-service/diagnostics.js`
 */
export function startOtherQSharpDiagnostics(): vscode.Disposable[] {
  return [...startQsharpJsonDiagnostics(), ...startCommandDiagnostics()];
}

//
// qsharp.json diagnostics.
//
// These are reported whenever problems are detected in a qsharp.json file.
//

let qsharpJsonDiagnostics: vscode.DiagnosticCollection | undefined;
let deleteQsharpJsonListener: vscode.Disposable | undefined;
const trackedQsharpJsons = new Set<string>();

function startQsharpJsonDiagnostics(): vscode.Disposable[] {
  qsharpJsonDiagnostics =
    vscode.languages.createDiagnosticCollection(qsharpLanguageId);

  deleteQsharpJsonListener = vscode.workspace.onDidDeleteFiles((event) => {
    for (const uri of event.files) {
      if (trackedQsharpJsons.delete(uri.toString())) {
        // Clear the diagnostics when the qsharp.json is deleted
        qsharpJsonDiagnostics?.set(uri, []);
      }
    }
  });

  return [qsharpJsonDiagnostics, deleteQsharpJsonListener];
}

export function updateQSharpJsonDiagnostics(
  qsharpJson: vscode.Uri,
  error?: string,
) {
  if (!qsharpJsonDiagnostics) {
    log.warn("no diagnostic collection for qsharp.json, not reporting");
  }

  trackedQsharpJsons.add(qsharpJson.toString());

  const errors = error
    ? [
        new vscode.Diagnostic(
          new vscode.Range(0, 0, 0, 0),
          error,
          vscode.DiagnosticSeverity.Error,
        ),
      ]
    : [];

  qsharpJsonDiagnostics?.set(qsharpJson, errors);
}

//
// Command diagnostics.
//
// These are Q# diagnostics that are caused by the invocation of a command, e.g. "generate QIR".
//
// These behave differently than diagnostics coming from the language service. They are not live.
// They pertain to the last-run command. They don't get cleared or refreshed when the
// user edits the document. They will be cleared when a new command is run.
// They can also be dismissed by the user with a code action.
//

let commandDiagnostics: vscode.DiagnosticCollection | undefined;

function startCommandDiagnostics(): vscode.Disposable[] {
  commandDiagnostics =
    vscode.languages.createDiagnosticCollection(qsharpLanguageId);

  const dismissCommand = vscode.commands.registerCommand(
    `${qsharpExtensionId}.dismissCommandDiagnostics`,
    clearCommandDiagnostics,
  );

  const dismissActionProvider = vscode.languages.registerCodeActionsProvider(
    [qsharpLanguageId, { language: "json", pattern: "**/qsharp.json" }],
    {
      provideCodeActions(doc, range, context): vscode.CodeAction[] | undefined {
        const commandErrors = context.diagnostics.filter((d) =>
          d.message.startsWith("Q# command error: "),
        );

        if (commandErrors.length > 0) {
          const action = new vscode.CodeAction(
            "Dismiss errors for the last run Q# command",
          );
          action.diagnostics = commandErrors;
          action.command = {
            command: `${qsharpExtensionId}.dismissCommandDiagnostics`,
            title: "Dismiss errors for the last run Q# command",
          };
          action.isPreferred = true;
          return [action];
        }
      },
    },
  );

  return [commandDiagnostics, dismissCommand, dismissActionProvider];
}

export function clearCommandDiagnostics() {
  if (!commandDiagnostics) {
    log.warn(`no diagnostic collection for commands, not clearing`);
  }

  commandDiagnostics?.clear();
}

/**
 * Wrapper that invokes a function. This function would be something that calls into
 * the Q# compiler. If the function throws an exception that contains Q# diagnostics,
 * reports the diagnostics to VS Code before rethrowing the original exception.
 */
export async function invokeAndReportCommandDiagnostics<T>(
  fn: () => T | Promise<T>,
): Promise<T> {
  try {
    // Clear the diagnostics from the last command run.
    clearCommandDiagnostics();
    return await fn();
  } catch (e: unknown) {
    reportIfQSharpErrors(e);
    throw e;
  }
}

/**
 * Given an exception, checks if it's a JSON string representation of IQSharpError[],
 * and if so, reports the diagnostics to VS Code.
 *
 * @param e an exception originating from the qsharp-lang package
 */
function reportIfQSharpErrors(e: unknown) {
  if (!commandDiagnostics) {
    log.warn(`diagnostic collection for commands was not initialized`);
    return;
  }

  let qsharpErrors: IQSharpError[] | undefined;
  if (typeof e === "string") {
    try {
      const errors = JSON.parse(e);
      // Check for the shape of IQSharpError[]
      if (Array.isArray(errors) && errors.length > 0 && errors[0].document) {
        qsharpErrors = errors;
      }
    } catch {
      // Couldn't parse the error as JSON.
      log.warn(`could not parse error string ${e}`);
    }
  }

  if (qsharpErrors) {
    const byUri = new Map<vscode.Uri, vscode.Diagnostic[]>();

    for (const error of qsharpErrors) {
      const uri = getSourceUri(error.document);

      const diagnostics = byUri.get(uri) || [];
      error.diagnostic.message = `Q# command error: ${error.diagnostic.message}`;
      diagnostics.push(toVsCodeDiagnostic(error.diagnostic));
      byUri.set(uri, diagnostics);
    }

    for (const [uri, diags] of byUri) {
      commandDiagnostics.set(uri, diags);
    }

    // Focus on Problems view
    vscode.commands.executeCommand("workbench.action.problems.focus");

    vscode.window.showErrorMessage(
      "The Q# command returned errors. Please see the Problems view.",
      { modal: true },
    );
  }
}

/**
 * This is temporary until we're able to report proper stdlib and project URIs from
 * the wasm layer. See https://github.com/microsoft/qsharp/blob/f8d344b32a1f1f918f3c91edf58c975db10f4370/wasm/src/diagnostic.rs
 *
 * @param maybeUri A source name returned from a Q# diagnostic
 * @returns A VS code URI that's okay to use in a Diagnostic object
 */
function getSourceUri(maybeUri: string): vscode.Uri {
  // An error without a span (e.g. "no entrypoint found") gets reported as a "project-level" error.
  // See: https://github.com/microsoft/qsharp/blob/f8d344b32a1f1f918f3c91edf58c975db10f4370/wasm/src/diagnostic.rs#L191
  // Ideally this would be a proper URI pointing to the project root or root document.
  // For now, make up a fake file path for display purposes.
  if (maybeUri === "<project>") {
    return vscode.Uri.file("Q# project");
  }

  try {
    return vscode.Uri.parse(maybeUri, true);
  } catch {
    // Not a URI, assume it's a filename from the stdlib
    // This URI should ideally be properly propagated from
    // https://github.com/microsoft/qsharp/blob/f8d344b32a1f1f918f3c91edf58c975db10f4370/wasm/src/diagnostic.rs#L105
    return vscode.Uri.from({ scheme: qsharpLibraryUriScheme, path: maybeUri });
  }
}
