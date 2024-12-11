// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { ILanguageService } from "qsharp-lang";
import * as vscode from "vscode";
import { toVscodeRange } from "../common";
import { EventType, FormatEvent, sendTelemetryEvent } from "../telemetry";
import { getRandomGuid } from "../utils";

export function createFormattingProvider(languageService: ILanguageService) {
  return new QSharpFormattingProvider(languageService);
}

class QSharpFormattingProvider
  implements
    vscode.DocumentFormattingEditProvider,
    vscode.DocumentRangeFormattingEditProvider
{
  constructor(public languageService: ILanguageService) {}

  private async getFormatChanges(
    document: vscode.TextDocument,
    eventKind: FormatEvent,
    range?: vscode.Range,
  ) {
    // telemetry start format
    const associationId = getRandomGuid();
    sendTelemetryEvent(
      EventType.FormatStart,
      { associationId, event: eventKind },
      {},
    );
    const start = performance.now();

    const lsEdits = await this.languageService.getFormatChanges(
      document.uri.toString(),
    );

    if (!lsEdits) {
      // telemetry end format
      sendTelemetryEvent(
        EventType.FormatEnd,
        { associationId },
        {
          timeToCompleteMs: performance.now() - start,
          numberOfEdits: 0,
        },
      );
      return [];
    }

    let edits = lsEdits.map(
      (edit) => new vscode.TextEdit(toVscodeRange(edit.range), edit.newText),
    );

    if (range) {
      edits = edits.filter(
        (e) =>
          range.start.isBefore(e.range.end) && range.end.isAfter(e.range.start),
      );
    }

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

  async provideDocumentFormattingEdits(document: vscode.TextDocument) {
    return await this.getFormatChanges(document, FormatEvent.OnDocument);
  }

  async provideDocumentRangeFormattingEdits(
    document: vscode.TextDocument,
    range: vscode.Range,
  ) {
    return await this.getFormatChanges(document, FormatEvent.OnRange, range);
  }
}
