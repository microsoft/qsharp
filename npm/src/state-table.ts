import {Dump} from "./common.js";

export const exampleDump: Dump = {
        "|0⟩": [
            -0.08980265101338736,
            0
        ],
        "|2⟩": [
            0.9878291611472623,
            0
        ],
        "|4⟩": [
            -0.08980265101338736,
            0
        ],
        "|6⟩": [
            -0.08980265101338754,
            0
        ]
};



// Takes the output of DumpMachine as a "Dump" object and renders the inner HTML of a
// table as a string. To use in a page you could use something like the below:
//
//     var dumpTable = document.querySelector('#theDumpTable');
//     dumpTable.innerHTML = renderDump(theDumpObject);
//
export function renderDump(dump: Dump): string {
  let table = `
    <thead>
      <tr>
        <th>Basis State<br/>(|&#x1D713;&#x2099;...&#x1D713;&#x2080;⟩)</th>
        <th>Amplitude</th>
        <th>Measurement Probability</th>
        <th colspan="2">Phase</th>
      </tr>
    </thead>
    <tbody>`;
    
  function probability(real: number, imag: number) {
    return (real * real + imag * imag);
  }

  function formatComplex(real: number, imag: number) {
    // toLocaleString() correctly identifies -0 in JavaScript
    // String interpolation drops minus sign from -0
    // &#x2212; is the unicode minus sign, &#x1D456; is the mathematical i
    const realPart = `${real.toLocaleString()[0] === "-" ? "&#x2212;" : ""}${Math.abs(real).toFixed(4)}`;
    const imagPart = `${imag.toLocaleString()[0] === "-" ? "&#x2212;" : "+"}${Math.abs(imag).toFixed(4)}&#x1D456;`;
    return `${realPart}${imagPart}`;
  }

  Object.keys(dump).forEach(basis => {
    const [real, imag] = dump[basis];
    const complex = formatComplex(real, imag)
    const probabilityPercent = probability(real, imag) * 100;
    const phase = Math.atan2(imag, real);
    table +=
      `<tr>
          <td><span>${basis}</span></td>
          <td><span">${complex}</span></td>
          <td>
              <progress max="100" value="${probabilityPercent}"></progress>
              <span>${probabilityPercent.toFixed(4)}%</span>
          </td>
          <td style="transform: rotate(${phase.toFixed(4)}rad)">&uarr;</td>
          <td>
              <span">${phase.toFixed(4)}</span>
          </td>
    </tr>`;
  });
  table += `</tbody>`;
  return table;
}
