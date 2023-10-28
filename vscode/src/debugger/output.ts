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

    function probability(real: number, imag: number) {
      return real * real + imag * imag;
    }

    const dump = evt.detail;
    out("");
    out("DumpMachine:");
    out("");
    out("  Basis | Amplitude     | Probability   | Phase");
    out("  ---------------------------------------------");
    Object.keys(dump).map((basis) => {
      const [real, imag] = dump[basis];
      const complex = formatComplex(real, imag);
      const probabilityPercent = probability(real, imag) * 100;
      const phase = Math.atan2(imag, real);

      out(
        `  ${basis}  | ${complex} | ${probabilityPercent.toFixed(
          4,
        )}%     | ${phase.toFixed(4)}`,
      );
    });
    out("");
    out("");
  });

  eventTarget.addEventListener("Result", (evt) => {
    const resultJson = JSON.stringify(evt.detail.value, null, 2);
    out(`Result: ${resultJson}`);
  });
  return eventTarget;
}
