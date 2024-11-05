// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { TargetProfile, log } from "qsharp-lang";
import * as vscode from "vscode";

export function getTarget(): TargetProfile {
  const target = vscode.workspace
    .getConfiguration("Q#")
    .get<TargetProfile>("qir.targetProfile", "unrestricted");
  switch (target) {
    case "base":
    case "adaptive_ri":
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
    "qir.targetProfile",
    target,
    vscode.ConfigurationTarget.Global,
  );
}

export function getTargetFriendlyName(targetProfile?: string) {
  switch (targetProfile) {
    case "base":
      return "Q#: QIR base";
    case "adaptive_ri":
      return "Q#: QIR Adaptive RI";
    case "unrestricted":
      return "Q#: unrestricted";
    default:
      log.error("invalid target profile found");
      return "Q#: invalid";
  }
}

export function getPauliNoiseModel(): number[] {
  const pauliNoiseSettings = vscode.workspace.getConfiguration(
    "Q#.simulation.pauliNoise",
  );
  const noiseTuple = [
    pauliNoiseSettings.get("X", 0),
    pauliNoiseSettings.get("Y", 0),
    pauliNoiseSettings.get("Z", 0),
  ];
  return noiseTuple;
}
