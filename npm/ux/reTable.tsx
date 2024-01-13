// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { useState } from "preact/hooks";
import { CreateReport } from "./report.js";

export type ReData = {
  status: string;
  jobParams: any;
  physicalCounts: any;
  physicalCountsFormatted: any;
  logicalQubit: any;
  tfactory: any;
  errorBudget: any;
  logicalCounts: any;
  new: boolean;
  frontierEntries: FrontierEntry[];
};

export type FrontierEntry = {
  logicalQubit: any;
  tfactory: any;
  errorBudget: any;
  physicalCounts: any;
  physicalCountsFormatted: any;
};

export function CreateReData(
  input: ReData,
  frontierEntryIndex: number | null = null,
): ReData {
  if (
    input.frontierEntries == null ||
    input.frontierEntries.length === 0 ||
    frontierEntryIndex == null
  ) {
    return input;
  } else {
    if (
      frontierEntryIndex < 0 ||
      frontierEntryIndex >= input.frontierEntries.length
    ) {
      frontierEntryIndex = 0;
    }

    const entry = input.frontierEntries[frontierEntryIndex];
    return {
      status: input.status,
      jobParams: input.jobParams,
      physicalCounts: entry.physicalCounts,
      physicalCountsFormatted: entry.physicalCountsFormatted,
      logicalQubit: entry.logicalQubit,
      tfactory: entry.tfactory,
      errorBudget: entry.errorBudget,
      logicalCounts: input.logicalCounts,
      frontierEntries: input.frontierEntries,
      new: input.new,
    };
  }
}

export function ReTable(props: {
  mdRenderer: (input: string) => string;
  estimatesData: ReData;
  index: number | null;
}) {
  const [showDetail, setShowDetail] = useState(false);
  const toggleDetail = () => {
    setShowDetail(!showDetail);
  };

  const estimatesData = CreateReData(props.estimatesData, props.index);
  const reportData = CreateReport(estimatesData);

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
                  estimatesData as any,
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
          {reportData.assumptions.map((assumption) => (
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
