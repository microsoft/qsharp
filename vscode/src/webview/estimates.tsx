// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { marked } from "marked";
import { estimatesData } from "./estimatesData";

const TeXZilla: {
  filterString: (tex: string) => string;
} = (window as any).TeXZilla;

// TODO: Function to change \dfrac to \frac, and remove \rm
function fixLatex(tex: string) {
  return tex.replace(/\\dfrac/g, "\\frac").replace(/\\\\rm/g, "");
}

export function Estimates() {
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
                  __html: TeXZilla.filterString(
                    marked(fixLatex(entry.explanation))
                  ),
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
    </div>
  );
}
