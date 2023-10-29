// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { marked } from "marked";
import { estimatesData } from "./estimatesData";

import { useEffect } from "preact/hooks";

const testString = `
$$H = \\frac{1}{\\sqrt{2}} \\left[\\begin{matrix}1 & 1 \\\\ 1 & -1\\end{matrix}\\right]$$
$$CNOT = \\begin{bmatrix}1 & 0 & 0 & 0 \\\\ 0 & 1 & 0 & 0 \\\\ 0 & 0 & 0 & 1 \\\\ 0 & 0 & 1 & 0\\end{bmatrix}$$
`;

export function Estimates() {
  useEffect(() => {
    (window as any).MathJax.typeset();
  });

  return (
    <div>
      <h2>Resource Estimates</h2>
      {estimatesData.reportData.groups.map((group) => {
        return (
          <details>
            <summary>
              <strong>{group.title}</strong>
            </summary>
            <table>
              {group.entries.map((entry) => {
                // entry.path is a '/' separated path to the value in the JSON to show
                const path = entry.path.split("/");
                const value = path.reduce(
                  (obj, key) => obj[key],
                  estimatesData as any
                );
                const renderedExplanation = {
                  __html: marked(entry.explanation),
                };
                console.log(entry.explanation);
                console.log(renderedExplanation);
                return (
                  <tr>
                    <td>{entry.label}</td>
                    <td>{value}</td>
                    <td>
                      <strong>{entry.description}</strong>
                      <hr />
                      <div dangerouslySetInnerHTML={renderedExplanation} />
                    </td>
                  </tr>
                );
              })}
            </table>
          </details>
        );
      })}
      <details>
        <summary>
          <strong>Assumptions</strong>
        </summary>
        <ul>
          {estimatesData.reportData.assumptions.map((assumption) => (
            <li dangerouslySetInnerHTML={{ __html: marked(assumption) }} />
          ))}
        </ul>
      </details>
      <div>{testString}</div>
    </div>
  );
}
