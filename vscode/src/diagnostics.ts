// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import {
  ILanguageService,
  VSDiagnostic,
  log,
  qsharpLibraryUriScheme,
} from "qsharp-lang";
import * as vscode from "vscode";
import { loadDocument, qsharpLanguageId } from "./common.js";

export function startCheckingQSharp(
  languageService: ILanguageService,
): vscode.Disposable[] {
  return [
    ...startLanguageServiceDiagnostics(languageService),
    ...startQsharpJsonDiagnostics(),
  ];
}

function startLanguageServiceDiagnostics(
  languageService: ILanguageService,
): vscode.Disposable[] {
  const diagCollection =
    vscode.languages.createDiagnosticCollection(qsharpLanguageId);

  async function onDiagnostics(evt: {
    detail: {
      uri: string;
      version: number;
      diagnostics: VSDiagnostic[];
    };
  }) {
    const diagnostics = evt.detail;
    const uri = vscode.Uri.parse(diagnostics.uri);

    if (uri.scheme === qsharpLibraryUriScheme) {
      // Don't report diagnostics for library files.
      return;
    }

    let document: vscode.TextDocument | undefined = undefined;
    if (diagnostics.diagnostics.length > 0) {
      // We need the document here to be able to map offsets to line/column positions.
      // The document may not be available if this event is to clear diagnostics
      // for an already-closed document from the problems list.
      document = await loadDocument(vscode.Uri.parse(diagnostics.uri));
    }

    const getPosition = (offset: number) => {
      return document!.positionAt(offset);
    };

    diagCollection.set(
      uri,
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
        const vscodeDiagnostic = new vscode.Diagnostic(
          new vscode.Range(getPosition(d.start_pos), getPosition(d.end_pos)),
          d.message,
          severity,
        );
        if (d.code) {
          vscodeDiagnostic.code = d.code;
        }
        if (d.related) {
          vscodeDiagnostic.relatedInformation = d.related.map((r) => {
            return new vscode.DiagnosticRelatedInformation(
              new vscode.Location(
                vscode.Uri.parse(r.source),
                new vscode.Range(
                  getPosition(r.start_pos),
                  getPosition(r.end_pos),
                ),
              ),
              r.message,
            );
          });
        }
        return vscodeDiagnostic;
      }),
    );
  }

  languageService.addEventListener("diagnostics", onDiagnostics);

  return [
    {
      dispose: () => {
        languageService.removeEventListener("diagnostics", onDiagnostics);
      },
    },
    diagCollection,
  ];
}

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
