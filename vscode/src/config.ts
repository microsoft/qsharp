// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { TargetProfile, log } from "qsharp-lang";
import * as vscode from "vscode";

export function getTarget(): TargetProfile {
  const target = vscode.workspace
    .getConfiguration("Q#")
    .get<TargetProfile>("targetProfile", "unrestricted");
  switch (target) {
    case "adaptive":
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

export function getTargetFriendlyName(targetProfile?: string) {
  switch (targetProfile) {
    case "base":
      return "Q#: QIR base";
    case "adaptive":
      return "Q#: QIR adaptive";
    case "unrestricted":
      return "Q#: unrestricted";
    default:
      log.error("invalid target profile found");
      return "Q#: invalid";
  }
}

export function getEnableFormating(): boolean {
  return vscode.workspace.getConfiguration("Q#").get<boolean>(
    "enableFormatting",
    true, // The default value should be set in `package.json` as well.
  );
}

export function getShowCircuitCodeLens(): boolean {
  return vscode.workspace.getConfiguration("Q#").get<boolean>(
    "showCircuitCodeLens",
    true, // The default value should be set in `package.json` as well.
  );
}

export function getUseQirGenPreview(): boolean {
  return vscode.workspace.getConfiguration("Q#").get<boolean>(
    "useQirGenPreview",
    false, // The default value should be set in `package.json` as well.
  );
}

export function getEnableAdaptiveProfile(): boolean {
  return vscode.workspace.getConfiguration("Q#").get<boolean>(
    "enableAdaptiveProfile",
    false, // The default value should be set in `package.json` as well.
  );
}
