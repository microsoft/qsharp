// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// A component including the results table and the scatter chart together.
// The results table is also a legend for the scatter chart.

import { ColorMap } from "./colormap.js";
import { FrontierEntry, ReData } from "./reTable.js";
import { ResultsTable, Row } from "./resultsTable.js";
import {
  Axis,
  HideTooltip,
  PlotItem,
  ScatterChart,
  ScatterSeries,
} from "./scatterChart.js";

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

export function CreateReData(
  input: ReData,
  frontierEntryIndex: number,
): ReData {
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

function reDataToRow(input: ReData, color: string): Row {
  const data = CreateReData(input, 0);
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

export function Summary(props: {
  estimatesData: ReData[];
  colors: string[] | null;
  isSimplifiedView: boolean;
  onRowDeleted: (rowId: string) => void;
  setEstimate: (estimate: ReData | null) => void;
}) {
  props.estimatesData.forEach((item, idx) => {
    if (item.jobParams.runName == null) {
      if (item.jobParams.label == null) {
        item.jobParams.runName = `(${String.fromCharCode(0x61 + idx)})`;
      } else {
        item.jobParams.runName = item.jobParams.label;
      }
    }
  });

  function onPointSelected(seriesIndex: number, pointIndex: number): void {
    const data = props.estimatesData[seriesIndex];
    props.setEstimate(CreateReData(data, pointIndex));
  }

  function onRowSelected(rowId: string) {
    // On any selection, clear the "new" flag on all rows. This ensures that
    // new rows do not steal focus from the user selected row.
    props.estimatesData.forEach((data) => (data.new = false));
    if (!rowId) {
      props.setEstimate(null);
    } else {
      const estimateFound =
        props.estimatesData.find((data) => data.jobParams.runName === rowId) ||
        null;
      if (estimateFound == null) {
        props.setEstimate(null);
      } else {
        props.setEstimate(CreateReData(estimateFound, 0));
      }
    }

    HideTooltip();
  }

  const colormap =
    props.colors != null && props.colors.length == props.estimatesData.length
      ? props.colors
      : ColorMap(props.estimatesData.length);

  if (props.isSimplifiedView) {
    return (
      <>
        <ResultsTable
          columnNames={columnNames}
          rows={props.estimatesData.map((dataItem, index) =>
            reDataToRow(dataItem, colormap[index]),
          )}
          initialColumns={initialColumns}
          ensureSelected={true}
          onRowSelected={onRowSelected}
          onRowDeleted={props.onRowDeleted}
        />
        <ScatterChart
          xAxis={xAxis}
          yAxis={yAxis}
          data={props.estimatesData.map((dataItem, index) =>
            reDataToRowScatter(dataItem, colormap[index]),
          )}
          onPointSelected={onPointSelected}
        />
      </>
    );
  }

  return (
    <>
      <details open>
        <summary style="font-size: 1.5em; font-weight: bold; margin: 24px 8px;">
          Results
        </summary>
        <ResultsTable
          columnNames={columnNames}
          rows={props.estimatesData.map((dataItem, index) =>
            reDataToRow(dataItem, colormap[index]),
          )}
          initialColumns={initialColumns}
          ensureSelected={true}
          onRowSelected={onRowSelected}
          onRowDeleted={props.onRowDeleted}
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
            reDataToRowScatter(dataItem, colormap[index]),
          )}
          onPointSelected={onPointSelected}
        />
      </details>
    </>
  );
}
