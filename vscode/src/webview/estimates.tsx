// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { marked } from "marked";
import { estimatesData } from "./estimatesData";

import { useEffect, useState } from "preact/hooks";
import { SpaceChart } from "./spaceChart";

export function Estimates() {
  const [showDetail, setShowDetail] = useState(false);

  useEffect(() => {
    (window as any).MathJax.typeset();
  });

  return (
    <div>
      <h2 style="margin: 24px 8px;">Resource Estimates</h2>
      <a
        href="#"
        style="font-size: 10px"
        onClick={() => setShowDetail(!showDetail)}
      >
        {showDetail ? "Hide details" : "Show details"}
      </a>
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
                return (
                  <tr>
                    <td className="estimate-cell title-cell">{entry.label}</td>
                    <td className="estimate-cell value-cell">{value}</td>
                    <td className="estimate-cell">
                      {showDetail ? (
                        <>
                          <strong>{entry.description}</strong>
                          <hr />
                          <div
                            className="estimate-explanation"
                            dangerouslySetInnerHTML={renderedExplanation}
                          />
                        </>
                      ) : (
                        <>
                          <span>{entry.description}</span>
                        </>
                      )}
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
      <h2 style="margin: 24px 8px;">Space diagram</h2>
      <SpaceChart />
    </div>
  );
}
