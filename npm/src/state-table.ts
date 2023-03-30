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
      <th>Basis state</th>
      <th>Amplitude</th>
      <th>Probability</th>
    </tr>
  </thead>
  <tbody>`;
    function probability(real: number, imag: number) {
        return (real * real + imag * imag);
    }
    Object.keys(dump).forEach(basis => {
        let [real, imag] = dump[basis];
        let prob = probability(real, imag);
        table += `<tr>
            <td>${basis}</td><td>${real.toFixed(4)} + ${imag.toFixed(4)}i</td><td>${prob.toFixed(4)}</td>
          </tr>`;
    });
    table += `</tbody>`;
    return table;
}
