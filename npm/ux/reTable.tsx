// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { useState } from "preact/hooks";

export type ReData = {
  status: string;
  jobParams: any;
  physicalCounts: any;
  physicalCountsFormatted: any;
  logicalQubit: any;
  tfactory: any;
  errorBudget: any;
  logicalCounts: any;
  reportData: {
    groups: {
      title: string;
      alwaysVisible: boolean;
      entries: {
        path: string;
        label: string;
        description: string;
        explanation: string;
      }[];
    }[];
    assumptions: string[];
  };
  new: boolean;
};

export function ReTable(props: {
  mdRenderer: (input: string) => string;
  estimatesData: ReData;
}) {
  const [showDetail, setShowDetail] = useState(false);
  const toggleDetail = () => {
    setShowDetail(!showDetail);
  };

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
      {props.estimatesData.reportData.groups.map((group) => {
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
                const renderedExplanation = {
                  __html: props.mdRenderer(entry.explanation),
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
          {props.estimatesData.reportData.assumptions.map((assumption) => (
            <li
              className="estimate-assumption"
              dangerouslySetInnerHTML={{ __html: props.mdRenderer(assumption) }}
            />
          ))}
        </ul>
      </details>
    </div>
  );
}
