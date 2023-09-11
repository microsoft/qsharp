// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

import * as vscode from "vscode";

export interface PackageInfo {
  aiKey: string;
  version: string;
}

export function getPackageInfo(
  context: vscode.ExtensionContext
): PackageInfo | undefined {
  let extensionPackage = require(context.asAbsolutePath("./package.json"));
  if (extensionPackage) {
    return {
      version: extensionPackage.version as string,
      aiKey: extensionPackage.aiKey as string,
    };
  }
  return;
}
