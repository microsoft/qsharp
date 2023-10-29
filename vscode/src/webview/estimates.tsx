// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { marked } from "marked";
import { estimatesData } from "./estimatesData";

import { useEffect } from "preact/hooks";

export function Estimates() {
  useEffect(() => {
    (window as any).MathJax.typeset();
  });

  return (
    <div>
      <h2>Resource Estimates</h2>
      {estimatesData.reportData.groups.map((group) => {
        return (
          <details className="estimate-details">
            <summary>
              <strong>{group.title}</strong>
            </summary>
            <table className="estimate-table">
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
                    <td className="estimate-cell title-cell">
                      <strong>{entry.label}</strong>
                    </td>
                    <td className="estimate-cell value-cell">{value}</td>
                    <td className="estimate-cell">
                      <strong>{entry.description}</strong>
                      <hr />
                      <div
                        className="estimate-explanation"
                        dangerouslySetInnerHTML={renderedExplanation}
                      />
                    </td>
                  </tr>
                );
              })}
            </table>
          </details>
        );
      })}
      <details className="estimate-details">
        <summary>
          <strong>Assumptions</strong>
        </summary>
        <ul className="estimate-table">
          {estimatesData.reportData.assumptions.map((assumption) => (
            <li
              className="estimate-assumption"
              dangerouslySetInnerHTML={{ __html: marked(assumption) }}
            />
          ))}
        </ul>
      </details>
    </div>
  );
}
