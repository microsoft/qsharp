// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { QscEventTarget } from "qsharp-lang";

export function createDebugConsoleEventTarget(out: (message: string) => void) {
  const eventTarget = new QscEventTarget(false);

  eventTarget.addEventListener("Message", (evt) => {
    out(`Message: ${evt.detail}`);
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

    function formatProbabilityPercent(real: number, imag: number) {
      const probabilityPercent = (real * real + imag * imag) * 100;
      return `${probabilityPercent.toFixed(4)}%`;
    }

    function formatPhase(real: number, imag: number) {
      const phase = Math.atan2(imag, real);
      return phase.toFixed(4);
    }

    const dump = evt.detail;
    const basisStates = Object.keys(dump);
    const basisColumnWidth = Math.max(
      basisStates[0]?.length ?? 0,
      "Basis".length,
    );
    const basis = "Basis".padEnd(basisColumnWidth);

    let out_str = "\n";
    out_str += "DumpMachine:\n\n";
    out_str += ` ${basis} | Amplitude      | Probability | Phase\n`;
    out_str +=
      " ".padEnd(basisColumnWidth, "-") +
      "-------------------------------------------\n";

    for (const row of basisStates) {
      const [real, imag] = dump[row];
      const basis = row.padStart(basisColumnWidth);
      const amplitude = formatComplex(real, imag).padStart(16);
      const probability = formatProbabilityPercent(real, imag).padStart(11);
      const phase = formatPhase(real, imag).padStart(8);

      out_str += ` ${basis} | ${amplitude} | ${probability} | ${phase}\n`;
    }

    out(out_str);
  });

  eventTarget.addEventListener("Result", (evt) => {
    const resultJson = JSON.stringify(evt.detail.value, null, 2);
    out(`Result: ${resultJson}`);
  });
  return eventTarget;
}
