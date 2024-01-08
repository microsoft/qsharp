// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { useState } from "preact/hooks";
import {
  ResultsTable,
  SpaceChart,
  ReTable,
  type ReData,
  type FrontierEntry,
  type Row,
  HideTooltip,
  ScatterChart,
  type ScatterSeries,
  type PlotItem,
  type Axis,
} from "qsharp-lang/ux";

import { colormap } from "colormap";

const columnNames = [
  "Run name",
  "Estimate type",
  "Qubit type",
  "QEC scheme",
  "Error budget",
  "Logical qubits",
  "Logical depth",
  "Code distance",
  "T states",
  "T factories",
  "T factory fraction",
  "Runtime",
  "rQOPS",
  "Physical qubits",
];

const initialColumns = [0, 1, 2, 3, 4, 10, 11, 12];

const xAxis: Axis = {
  isTime: true,
  label: "Runtime",
};

const yAxis: Axis = {
  isTime: false,
  label: "Physical Qubits",
};

function reDataToRow(input: ReData, color: string): Row {
  const data = createReData(input, 0);
  const estimateType =
    data.frontierEntries == null
      ? "Single"
      : "Frontier (" + data.frontierEntries.length + "  items)";

  return {
    cells: [
      data.jobParams.runName,
      estimateType,
      data.jobParams.qubitParams.name,
      data.jobParams.qecScheme.name,
      data.jobParams.errorBudget,
      data.physicalCounts.breakdown.algorithmicLogicalQubits,
      data.physicalCounts.breakdown.algorithmicLogicalDepth,
      data.logicalQubit.codeDistance,
      data.physicalCounts.breakdown.numTstates,
      data.physicalCounts.breakdown.numTfactories,
      data.physicalCountsFormatted.physicalQubitsForTfactoriesPercentage,
      {
        value: data.physicalCountsFormatted.runtime,
        sortBy: data.physicalCounts.runtime,
      },
      data.physicalCounts.rqops,
      data.physicalCounts.physicalQubits,
      data.new ? "New" : "Cached",
    ],
    color: color,
  };
}

function frontierEntryToPlotEntry(entry: FrontierEntry): PlotItem {
  return {
    x: entry.physicalCounts.runtime,
    y: entry.physicalCounts.physicalQubits,
    label:
      entry.physicalCountsFormatted.runtime +
      " " +
      entry.physicalCountsFormatted.physicalQubits,
  };
}

function reDataToRowScatter(data: ReData, color: string): ScatterSeries {
  if (data.frontierEntries == null || data.frontierEntries.length === 0) {
    return {
      color: color,
      items: [
        {
          x: data.physicalCounts.runtime,
          y: data.physicalCounts.physicalQubits,
          label:
            data.physicalCountsFormatted.runtime +
            " " +
            data.physicalCountsFormatted.physicalQubits,
        },
      ],
    };
  }

  return {
    color: color,
    items: data.frontierEntries.map(frontierEntryToPlotEntry),
  };
}

function createReData(input: ReData, frontierEntryIndex: number): ReData {
  if (input.frontierEntries == null || input.frontierEntries.length === 0) {
    return input;
  } else {
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
        setEstimate(createReData(estimateFound, 0));
      }
    }

    HideTooltip();
  }

  function onRowDeleted(rowId: string) {
    props.onRowDeleted(rowId);
  }

  function onPointSelected(seriesIndex: number, pointIndex: number): void {
    const data = props.estimatesData[seriesIndex];
    setEstimate(createReData(data, pointIndex));
  }

  const predefinedColors = [
    "#FF0000", // Red
    "#0000FF", // Blue
    "#00FF00", // Green
    "#800080", // Purple
    "#FFA500", // Orange
    "#008080", // Teal
    "#FFC0CB", // Pink
    "#FFFF00", // Yellow
    "#A52A2A", // Brown
    "#00FFFF", // Cyan
  ];

  let colors = predefinedColors;

  function getColor(index: number) {
    if (props.estimatesData != null && props.estimatesData.length > 0) {
      if (props.estimatesData.length > predefinedColors.length) {
        if (props.estimatesData.length != colors.length) {
          colors = colormap({
            colormap: "jet",
            nshades: Math.max(6, props.estimatesData.length), // 6 is the minimum number of colors in the colormap 'jet'
            format: "hex",
            alpha: 1,
          });
        }
      } else {
        colors = predefinedColors;
      }
    }
    return colors[index];
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
          columnNames={columnNames}
          rows={props.estimatesData.map((dataItem, index) =>
            reDataToRow(dataItem, getColor(index)),
          )}
          initialColumns={initialColumns}
          ensureSelected={true}
          onRowSelected={onRowSelected}
          onRowDeleted={onRowDeleted}
        />
        <ScatterChart
          xAxis={xAxis}
          yAxis={yAxis}
          data={props.estimatesData.map((dataItem, index) =>
            reDataToRowScatter(dataItem, getColor(index)),
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
