// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { log } from "qsharp-lang";
import * as vscode from "vscode";
export type Target = "base" | "full";

export function getTarget(): Target {
  const target = vscode.workspace
    .getConfiguration("Q#")
    .get<string>("targetProfile", "full");
  switch (target) {
    case "base":
    case "full":
      return target;
    default:
      log.error("invalid target found: %s", target);
      return "full";
  }
}

export async function setTarget(target: Target) {
  const config = vscode.workspace.getConfiguration("Q#");
  await config.update(
    "targetProfile",
    target,
    vscode.ConfigurationTarget.Global,
  );
}
