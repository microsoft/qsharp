// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { QscEventTarget, VSDiagnostic } from "qsharp";
import * as vscode from "vscode";
import { qsharpLanguageId } from "./common.js";

export function startCheckingQSharp(evtTarget: QscEventTarget) {
  const diagCollection =
    vscode.languages.createDiagnosticCollection(qsharpLanguageId);

  function onDiagnostics(evt: {
    detail: {
      uri: string;
      version: number;
      diagnostics: VSDiagnostic[];
    };
  }) {
    const diagnostics = evt.detail;

    const getPosition = (offset: number) => {
      // We need the document here to be able to map offsets to line/column positions.
      // The document may not be available if this event is to clear diagnostics
      // for an already-closed document from the problems list.
      // Note: This mapping will break down if we ever send diagnostics for closed files.
      const document = vscode.workspace.textDocuments.filter(
        (doc) => doc.uri.toString() === diagnostics.uri
      )[0];
      return document.positionAt(offset);
    };

    diagCollection.set(
      vscode.Uri.parse(evt.detail.uri),
      diagnostics.diagnostics.map((d) => {
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

  evtTarget.addEventListener("diagnostics", onDiagnostics);

  return {
    dispose: () => {
      evtTarget.removeEventListener("diagnostics", onDiagnostics);
    },
  };
}
