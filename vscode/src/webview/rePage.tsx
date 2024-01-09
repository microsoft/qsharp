// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { useState } from "preact/hooks";
import {
  CreateReData,
  ColumnNames,
  InitialColumns,
  ResultsTable,
  SpaceChart,
  ReTable,
  ReDataToRow,
  ReDataToRowScatter,
  type ReData,
  HideTooltip,
  ScatterChart,
  xAxis,
  yAxis,
  GetColor,
} from "qsharp-lang/ux";

export function RePage(props: {
  estimatesData: ReData[];
  calculating: boolean;
  renderer: (input: string) => string;
  onRowDeleted: (rowId: string) => void;
}) {
  const [estimate, setEstimate] = useState<ReData | null>(null);

  function onRowSelected(rowId: string) {
    // On any selection, clear the "new" flag on all rows. This ensures that
    // new rows do not steal focus from the user selected row.
    props.estimatesData.forEach((data) => (data.new = false));
    if (!rowId) {
      setEstimate(null);
    } else {
      const estimateFound =
        props.estimatesData.find((data) => data.jobParams.runName === rowId) ||
        null;
      if (estimateFound == null) {
        setEstimate(null);
      } else {
        setEstimate(CreateReData(estimateFound, 0));
      }
    }

    HideTooltip();
  }

  function onRowDeleted(rowId: string) {
    props.onRowDeleted(rowId);
  }

  function onPointSelected(seriesIndex: number, pointIndex: number): void {
    const data = props.estimatesData[seriesIndex];
    setEstimate(CreateReData(data, pointIndex));
  }

  function getColor(index: number) {
    return GetColor(index, props.estimatesData.length);
  }

  return (
    <>
      <div style="display:flex; height:64px; align-items: center; position: relative">
        <svg
          width="48"
          height="48"
          viewBox="96 96 828 828"
          xmlns="http://www.w3.org/2000/svg"
        >
          <g fill="none" stroke="gray" stroke-width="8">
            <path d="M 512 135 L 819 313 L 819 667 L 512 845 L 205 667 L 205 313 L 512 135 Z" />
            <path d="M 205 580 L 742 890 L 819 845 L 818 756 L 205 402" />
            <path d="M 204 579 L 743 268" />
            <path d="M 664 224 L 207 489" />
            <path d="M 206 400 L 588 180" />
            <path d="M 205 667 L 818 313" />
            <path d="M 205 490 L 820 845" />
            <path d="M 207 314 L 818 667" />
            <path d="M 282 269 L 820 580" />
            <path d="M 820 490 L 357 223" />
            <path d="M 435 180 L 818 400" />
            <path d="M 281 710 L 281 271" />
            <path d="M 357 755 L 357 223" />
            <path d="M 283 711 L 820 400" />
            <path d="M 434 797 L 434 181" />
            <path d="M 511 136 L 511 844" />
            <path d="M 588 799 L 588 182" />
            <path d="M 665 223 L 665 845" />
            <path d="M 742 887 L 742 267" />
            <path d="M 665 845 L 816 758" />
            <path d="M 433 801 L 820 577" />
            <path d="M 820 489 L 360 755" />
          </g>
        </svg>
        {props.calculating ? (
          <svg
            width="40"
            height="40"
            viewBox="0 0 16 16"
            xmlns="http://www.w3.org/2000/svg"
            class="codicon-modifier-spin"
            style="position: absolute; top: 11px; left: 4px;"
          >
            <path d="M2.006 8.267L.78 9.5 0 8.73l2.09-2.07.76.01 2.09 2.12-.76.76-1.167-1.18a5 5 0 0 0 9.4 1.983l.813.597a6 6 0 0 1-11.22-2.683zm10.99-.466L11.76 6.55l-.76.76 2.09 2.11.76.01 2.09-2.07-.75-.76-1.194 1.18a6 6 0 0 0-11.11-2.92l.81.594a5 5 0 0 1 9.3 2.346z"></path>
          </svg>
        ) : null}
        <h1>Azure Quantum Resource Estimator</h1>
      </div>
      <details open>
        <summary style="font-size: 1.5em; font-weight: bold; margin: 24px 8px;">
          Results
        </summary>
        <ResultsTable
          columnNames={ColumnNames}
          rows={props.estimatesData.map((dataItem, index) =>
            ReDataToRow(dataItem, getColor(index)),
          )}
          initialColumns={InitialColumns}
          ensureSelected={true}
          onRowSelected={onRowSelected}
          onRowDeleted={onRowDeleted}
        />
      </details>
      <details open>
        <summary style="font-size: 1.5em; font-weight: bold; margin: 24px 8px;">
          Scatter chart
        </summary>
        <ScatterChart
          xAxis={xAxis}
          yAxis={yAxis}
          data={props.estimatesData.map((dataItem, index) =>
            ReDataToRowScatter(dataItem, getColor(index)),
          )}
          onPointSelected={onPointSelected}
        />
      </details>
      {!estimate ? null : (
        <>
          <details open>
            <summary style="font-size: 1.5em; font-weight: bold; margin: 24px 8px;">
              Space diagram
            </summary>
            <SpaceChart estimatesData={estimate} />
          </details>
          <details open>
            <summary style="font-size: 1.5em; font-weight: bold; margin: 24px 8px;">
              Resource Estimates
            </summary>
            <ReTable mdRenderer={props.renderer} estimatesData={estimate} />
          </details>
        </>
      )}
    </>
  );
}
