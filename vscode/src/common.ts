// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { TextDocument, Uri, workspace } from "vscode";

export const qsharpLanguageId = "qsharp";

// Returns true for all Q# documents, including unsaved files, notebook cells, etc.
export function isQsharpDocument(document: TextDocument): boolean {
  return document.languageId === qsharpLanguageId;
}

// Returns true for only Q# notebook cell documents.
export function isQsharpNotebookCell(document: TextDocument): boolean {
  return (
    document.languageId === qsharpLanguageId &&
    document.uri.scheme === "vscode-notebook-cell"
  );
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
 *
 * Will throw if the document cannot be opened.
 * This utility is indented to be used to map offsets to line/column
 * positions. If/when the compiler returns line/column positions
 * directly, we can skip this async and fallible step of loading
 * the document just to map offsets. See: https://github.com/microsoft/qsharp/issues/851
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
