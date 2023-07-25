// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as vscode from "vscode";

import { QscEventTarget } from "qsharp";

export function createDebugConsoleEventTarget() {
  const eventTarget = new QscEventTarget(false);

  eventTarget.addEventListener("Message", (evt) => {
    vscode.debug.activeDebugConsole.appendLine(`Message: ${evt.detail}`);
  });

  eventTarget.addEventListener("DumpMachine", (evt) => {
    function formatComplex(real: number, imag: number) {
      // Format -0 as 0
      // Also using Unicode Minus Sign instead of ASCII Hyphen-Minus
      // and Unicode Mathematical Italic Small I instead of ASCII i.
      const r = `${real <= -0.00005 ? "−" : ""}${Math.abs(real).toFixed(4)}`;
      const i = `${imag <= -0.00005 ? "−" : "+"}${Math.abs(imag).toFixed(4)}𝑖`;
      return `${r}${i}`;
    }

    function probability(real: number, imag: number) {
      return real * real + imag * imag;
    }

    const dump = evt.detail;
    vscode.debug.activeDebugConsole.appendLine("");
    vscode.debug.activeDebugConsole.appendLine("DumpMachine:");
    vscode.debug.activeDebugConsole.appendLine("");
    vscode.debug.activeDebugConsole.appendLine(
      "  Basis | Amplitude     | Probability   | Phase"
    );
    vscode.debug.activeDebugConsole.appendLine(
      "  ---------------------------------------------"
    );
    Object.keys(dump).map((basis) => {
      const [real, imag] = dump[basis];
      const complex = formatComplex(real, imag);
      const probabilityPercent = probability(real, imag) * 100;
      const phase = Math.atan2(imag, real);

      vscode.debug.activeDebugConsole.appendLine(
        `  ${basis}  | ${complex} | ${probabilityPercent.toFixed(
          4
        )}%     | ${phase.toFixed(4)}`
      );
    });
    vscode.debug.activeDebugConsole.appendLine("");
    vscode.debug.activeDebugConsole.appendLine("");
  });

  eventTarget.addEventListener("Result", (evt) => {
    const resultJson = JSON.stringify(evt.detail.value, null, 2);
    vscode.debug.activeDebugConsole.appendLine(`Result: ${resultJson}`);
  });
  return eventTarget;
}
