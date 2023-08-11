// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

export const qsharpLanguageId = "qsharp";
export const qsharpExtensionId = "qsharp-vscode";
export const QsLibraryUriScheme = "qsharp-source-request";

export interface FileAccessor {
  readFile(uri: string): Promise<Uint8Array>;
  readFileAsString(uri: string): Promise<string>;
  writeFile(uri: string, contents: Uint8Array): Promise<void>;
}
