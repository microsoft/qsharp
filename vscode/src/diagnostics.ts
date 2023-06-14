// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { type getCompiler } from "qsharp";
import * as vscode from "vscode";
import { qsharpLanguageId } from "./common.js";

/**
 * Checks the currently open Q# documents and publishes diagnostics.
 * Then subscribes to document updates to check code and update the diagnostics.
 */
export async function startCheckingQSharp(
  compiler: Awaited<ReturnType<typeof getCompiler>>
) {
  const diagCollection =
    vscode.languages.createDiagnosticCollection(qsharpLanguageId);

  // This here is OK, but be aware that async functions in map() are not
  // necessarily executed sequentially
  await Promise.all(
    vscode.workspace.textDocuments.map(async (document) => {
      await updateDiagnosticsIfQSharp(document);
    })
  );

  const subscriptions = [];

  subscriptions.push(
    vscode.workspace.onDidOpenTextDocument((document) => {
      updateDiagnosticsIfQSharp(document);
    })
  );

  subscriptions.push(
    vscode.workspace.onDidChangeTextDocument((evt) => {
      updateDiagnosticsIfQSharp(evt.document);
    })
  );

  subscriptions.push(
    vscode.workspace.onDidCloseTextDocument((document) => {
      if (vscode.languages.match(qsharpLanguageId, document)) {
        // Clear diagnostics for a closed document
        diagCollection.delete(document.uri);
      }
    })
  );

  async function updateDiagnosticsIfQSharp(document: vscode.TextDocument) {
    if (vscode.languages.match(qsharpLanguageId, document)) {
      const diags = await compiler.checkCode(document.getText());

      const getPosition = (offset: number) => {
        return document.positionAt(offset);
      };

      diagCollection.set(
        document.uri,
        diags.map((d) => {
          let severity;
          switch (d.severity) {
            case "error":
              severity = vscode.DiagnosticSeverity.Error;
              break;
            case "warning":
              severity = vscode.DiagnosticSeverity.Warning;
              break;
            case "info":
              severity = vscode.DiagnosticSeverity.Information;
              break;
          }
          return new vscode.Diagnostic(
            new vscode.Range(getPosition(d.start_pos), getPosition(d.end_pos)),
            d.message,
            severity
          );
        })
      );
    }
  }

  return subscriptions;
}
