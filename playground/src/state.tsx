// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { Dump } from "qsharp-lang";

function probability(real: number, imag: number) {
  return real * real + imag * imag;
}

function formatComplex(real: number, imag: number) {
  // Format -0 as 0
  // Also using Unicode Minus Sign instead of ASCII Hyphen-Minus
  // and Unicode Mathematical Italic Small I instead of ASCII i.
  const r = `${real <= -0.00005 ? "−" : ""}${Math.abs(real).toFixed(4)}`;
  const i = `${imag <= -0.00005 ? "−" : "+"}${Math.abs(imag).toFixed(4)}𝑖`;
  return `${r}${i}`;
}

export function StateTable(props: { dump: Dump }) {
  return (
    <table class="state-table">
      <thead>
        <tr>
          <th>
            Basis State
            <br />
            (|𝜓₁…𝜓ₙ⟩)
          </th>
          <th>Amplitude</th>
          <th>Measurement Probability</th>
          <th colSpan={2}>Phase</th>
        </tr>
      </thead>
      <tbody>
        {Object.keys(props.dump).map((basis) => {
          const [real, imag] = props.dump[basis];
          const complex = formatComplex(real, imag);
          const probabilityPercent = probability(real, imag) * 100;
          const phase = Math.atan2(imag, real);
          const phaseStyle = `transform: rotate(${phase.toFixed(4)}rad)`;
          return (
            <tr>
              <td style="text-align: center">{basis}</td>
              <td style="text-align: right">{complex}</td>
              <td style="display: flex; justify-content: space-between; padding: 8px 20px;">
                <progress
                  style="width: 40%"
                  max="100"
                  value={probabilityPercent}
                ></progress>
                <span>{probabilityPercent.toFixed(4)}%</span>
              </td>
              <td style={phaseStyle}>↑</td>
              <td style="text-align:right">{phase.toFixed(4)}</td>
            </tr>
          );
        })}
      </tbody>
    </table>
  );
}
