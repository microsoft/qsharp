// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as vscode from "vscode";

export const qsharpLanguageId = "qsharp";
export const qsharpExtensionId = "qsharp-vscode";

export interface FileAccessor {
  readFile(uri: vscode.Uri): Promise<Uint8Array>;
  readFileAsString(uri: vscode.Uri): Promise<string>;
  writeFile(uri: vscode.Uri, contents: Uint8Array): Promise<void>;
}
