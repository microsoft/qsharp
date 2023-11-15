// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import {
  DocumentFilter,
  TextDocument,
  Uri,
  languages,
  workspace,
} from "vscode";

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

export function isQsharpDocument(document: TextDocument): boolean {
  return languages.match(qsharpDocumentFilter, document) > 0;
}

export function isQsharpNotebookCell(document: TextDocument): boolean {
  return languages.match(qsharpNotebookCellDocumentFilter, document) > 0;
}

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

/**
 * Loads a Q# document or notebook cell.
 *
 * This does *not* open a visible document in the editor.
 */
export async function loadDocument(uri: Uri) {
  const uriString = uri.toString();
  // Search both text documents and notebook cell documents
  const doc = workspace.textDocuments
    .concat(
      workspace.notebookDocuments.flatMap((doc) =>
        doc
          .getCells()
          .filter((cell) => isQsharpDocument(cell.document))
          .map((cell) => cell.document),
      ),
    )
    .find((doc) => doc.uri.toString() === uriString);

  return doc || (await workspace.openTextDocument(uri));
}
