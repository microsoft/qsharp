// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as vscode from "vscode";
export type Target = "base" | "full";

export function getTarget(): Target {
  const target = vscode.workspace
    .getConfiguration("Q#")
    .get<Target>("targetProfile", "full");
  return target;
}

export async function setTarget(target: Target) {
  const config = vscode.workspace.getConfiguration("Q#");
  await config.update(
    "targetProfile",
    target,
    vscode.ConfigurationTarget.Global,
  );
}
