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
    case "adaptive_rif":
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
      return "QIR base";
    case "adaptive_ri":
      return "QIR Adaptive RI";
    case "adaptive_rif":
      return "QIR Adaptive RIF";
    case "unrestricted":
      return "QIR unrestricted";
    default:
      log.error("invalid target profile found");
      return "QIR invalid";
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

export function getQubitLossSetting(): number {
  const qubitLoss = vscode.workspace
    .getConfiguration("Q#.simulation")
    .get<number>("qubitLoss", 0);
  return qubitLoss;
}

export function getShowDevDiagnostics(): boolean {
  return vscode.workspace
    .getConfiguration("Q#")
    .get<boolean>("dev.showDevDiagnostics", false);
}
