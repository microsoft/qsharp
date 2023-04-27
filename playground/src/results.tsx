// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { Dump } from "qsharp";

const dumpExample: Dump = {"|0000⟩":[0.4338641278491088,-0.0655721495701471],"|0001⟩":[0.4338641278491088,0.0655721495701471],"|0010⟩":[0.4338641278491088,-0.0655721495701471],"|0011⟩":[0.4338641278491088,0.0655721495701471],"|1100⟩":[-0.03582222857458079,-0.23702105329787282],"|1101⟩":[0.03582222857458079,-0.23702105329787282],"|1110⟩":[-0.03582222857458079,-0.23702105329787282],"|1111⟩":[0.03582222857458079,-0.23702105329787282]};

function probability(real: number, imag: number) {
    return (real * real + imag * imag);
}

function formatComplex(real: number, imag: number) {
    // toLocaleString() correctly identifies -0 in JavaScript
    // String interpolation drops minus sign from -0
    // &#x2212; is the unicode minus sign, &#x1D456; is the mathematical i
    const realPart = `${real.toLocaleString()[0] === "-" ? "−" : ""}${Math.abs(real).toFixed(4)}`;
    const imagPart = `${imag.toLocaleString()[0] === "-" ? "−" : "+"}${Math.abs(imag).toFixed(4)}𝑖`;
    return `${realPart}${imagPart}`;
}

export function StateTable(props: any) {
    return (
<table class="state-table">
  <thead>
    <tr>
      <th>Basis State<br/>(|𝜓ₙ…𝜓₁⟩)</th>
      <th>Amplitude</th>
      <th>Measurement Probability</th>
      <th colSpan={2}>Phase</th>
    </tr>
  </thead>
  <tbody>
{ Object.keys(dumpExample).map(basis => {
    const [real, imag] = dumpExample[basis];
    const complex = formatComplex(real, imag)
    const probabilityPercent = probability(real, imag) * 100;
    const phase = Math.atan2(imag, real);
    const phaseStyle = `transform: rotate(${phase.toFixed(4)}rad)`;
    return (
    <tr>
      <td style="text-align: center">{basis}</td>
      <td style="text-align: right">{complex}</td>
      <td style="display: flex; justify-content: space-between; padding: 8px 20px;">
        <progress style="width: 40%" max="100" value={probabilityPercent}></progress>
        <span>{probabilityPercent.toFixed(4)}%</span>
      </td>
      <td style={phaseStyle}>↑</td>
      <td style="text-align:right">{phase.toFixed(4)}</td>
    </tr>);
})}
  </tbody>
</table>);
}

export function Results(props: any) {
    return (
<div class="results-column">
  <div class="results-labels">
    <div class="results-active-tab">RESULTS</div>
    <div>AST</div>
    <div>LOGS</div>
  </div>
  <div id="histogram"></div>
  <div class="output-header">
    <div>Shot 21 of 100</div>
    <div class="prev-next">Prev | Next</div>
  </div>
  <StateTable></StateTable>
</div>);
}
