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

    function column(header: string, values: string[]) {
      // The length of the longest value, or the header length, whichever is longer
      const columnWidth = values.reduce((max, value) => {
        if (value.length > max) {
          return value.length;
        }
        return max;
      }, header.length);
      // Left-align the header, right-align the values
      return {
        header: header.padEnd(columnWidth),
        values: values.map((v) => v.padStart(columnWidth)),
      };
    }

    const dump = evt.detail;
    const basisStates = Object.keys(dump);
    const rows = basisStates.map((basis) => {
      const [real, imag] = dump[basis];
      const amplitude = formatComplex(real, imag);
      const probability = formatProbabilityPercent(real, imag);
      const phase = formatPhase(real, imag);
      return { amplitude, probability, phase };
    });

    const basis = column("Basis", basisStates);
    const amplitude = column(
      "Amplitude",
      rows.map((r) => r.amplitude),
    );
    const probability = column(
      "Probability",
      rows.map((r) => r.probability),
    );
    const phase = column(
      "Phase",
      rows.map((r) => r.phase),
    );

    let out_str = "\n";
    out_str += "DumpMachine:\n\n";
    const headerRow = `${basis.header} | ${amplitude.header} | ${probability.header} | ${phase.header}`;
    out_str += ` ${headerRow}\n`;
    out_str += " ".padEnd(headerRow.length, "-") + "--\n";
    for (let i = 0; i < basis.values.length; i++) {
      // The special characters skew the visual alignment of the columns, just add extra spaces to correct for that
      out_str += ` ${basis.values[i]}  |  ${amplitude.values[i]} | ${probability.values[i]} | ${phase.values[i]}\n`;
    }

    out(out_str);
  });

  eventTarget.addEventListener("Result", (evt) => {
    const resultJson = JSON.stringify(evt.detail.value, null, 2);
    out(`Result: ${resultJson}`);
  });
  return eventTarget;
}
