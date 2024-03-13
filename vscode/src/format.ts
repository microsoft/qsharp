// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { ILanguageService } from "qsharp-lang";
import * as vscode from "vscode";
import { toVscodeRange } from "./common";
import { EventType, sendTelemetryEvent } from "./telemetry";
import { getRandomGuid } from "./utils";

export function createFormatProvider(languageService: ILanguageService) {
  return new QSharpFormatProvider(languageService);
}

class QSharpFormatProvider implements vscode.DocumentFormattingEditProvider {
  constructor(public languageService: ILanguageService) {}

  async provideDocumentFormattingEdits(document: vscode.TextDocument) {
    // telemetry start format
    const associationId = getRandomGuid();
    sendTelemetryEvent(EventType.FormatStart, { associationId }, {});
    const start = performance.now();

    const lsEdits = await this.languageService.getFormatChanges(
      document.uri.toString(),
    );

    if (!lsEdits) return [];
    const edits = lsEdits.map(
      (edit) => new vscode.TextEdit(toVscodeRange(edit.range), edit.newText),
    );

    // telemetry end format
    sendTelemetryEvent(
      EventType.FormatEnd,
      { associationId },
      {
        timeToCompleteMs: performance.now() - start,
        numberOfEdits: edits.length,
      },
    );

    return edits;
  }
}
