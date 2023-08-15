// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { DocumentFilter } from "vscode";

export const qsharpLanguageId = "qsharp";
// Matches all Q# documents, including unsaved files, notebook cells, etc.
export const qsharpDocumentFilter: DocumentFilter = {
  language: qsharpLanguageId,
};
// Matches only Q# notebook cell documents.
export const qsharpNotebookCellDocumentFilter: DocumentFilter = {
  language: qsharpLanguageId,
  notebookType: "jupyter-notebook",
};
export const qsharpExtensionId = "qsharp-vscode";

export interface FileAccessor {
  readFile(uri: string): Promise<Uint8Array>;
  readFileAsString(uri: string): Promise<string>;
  writeFile(uri: string, contents: Uint8Array): Promise<void>;
}
