// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { DocumentFilter, TextDocument, Uri } from "vscode";

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
  normalizePath(path: string): string;
  convertToWindowsPathSeparator(path: string): string;
  resolvePathToUri(path: string): Uri;
  openPath(path: string): Promise<TextDocument>;
  openUri(uri: Uri): Promise<TextDocument>;
}

export function basename(path: string): string | undefined {
  return path.replace(/\/+$/, "").split("/").pop();
}
