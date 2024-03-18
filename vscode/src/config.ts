// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { TargetProfile, log } from "qsharp-lang";
import * as vscode from "vscode";

export function getTarget(): TargetProfile {
  const target = vscode.workspace
    .getConfiguration("Q#")
    .get<TargetProfile>("targetProfile", "unrestricted");
  switch (target) {
    case "base":
    case "unrestricted":
      return target;
    default:
      log.error("invalid target found: %s", target);
      return "unrestricted";
  }
}

export async function setTarget(target: TargetProfile) {
  const config = vscode.workspace.getConfiguration("Q#");
  await config.update(
    "targetProfile",
    target,
    vscode.ConfigurationTarget.Global,
  );
}

export function getEnableFormating(): boolean {
  return vscode.workspace
    .getConfiguration("Q#")
    .get<boolean>("enableFormatting", true);
}
