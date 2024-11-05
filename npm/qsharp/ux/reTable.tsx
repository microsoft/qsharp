// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { useState } from "preact/hooks";
import { SingleEstimateResult } from "./data.js";
import { CreateReport } from "./report.js";
import { Markdown } from "./renderers.js";

export function ReTable(props: { estimatesData: SingleEstimateResult }) {
  const [showDetail, setShowDetail] = useState(false);
  const toggleDetail = () => {
    setShowDetail(!showDetail);
  };

  const reportData = CreateReport(props.estimatesData);

  return (
    <div>
      <div>
        <input
          type="checkbox"
          id="showDetail"
          checked={showDetail}
          onClick={toggleDetail}
        />
        <label htmlFor="showDetail"> Show detailed rows</label>
      </div>
      {reportData.groups.map((group) => {
        return (
          <details className="estimate-details">
            <summary>
              <strong>{group.title}</strong>
            </summary>
            <table className="estimate-table">
              {group.entries.map((entry) => {
                // entry.path is a '/' separated path to the value in the JSON to show
                const path = entry.path.split("/");
                let value = path.reduce(
                  (obj, key) => obj[key],
                  props.estimatesData as any,
                );
                // Check if value is not a primitive type
                if (typeof value === "object") {
                  value = JSON.stringify(value);
                }
                return (
                  <tr>
                    <td className="estimate-cell title-cell">{entry.label}</td>
                    <td className="estimate-cell value-cell">{value}</td>
                    <td className="estimate-cell">
                      {showDetail ? (
                        <>
                          <strong>{entry.description}</strong>
                          <hr />
                          <Markdown
                            className="estimate-explanation"
                            markdown={entry.explanation}
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
          {reportData.assumptions.map((assumption) => (
            <Markdown
              className="estimate-assumption"
              markdown={assumption}
              tagName="li"
            />
          ))}
        </ul>
      </details>
    </div>
  );
}
