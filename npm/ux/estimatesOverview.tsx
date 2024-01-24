// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// A component including the results table and the scatter chart together.
// The results table is also a legend for the scatter chart.

import { useState } from "preact/hooks";
import { ColorMap } from "./colormap.js";
import {
  CreateSingleEstimateResult,
  FrontierEntry,
  ReData,
  SingleEstimateResult,
} from "./data.js";
import { ResultsTable, Row } from "./resultsTable.js";
import {
  Axis,
  HideTooltip,
  PlotItem,
  ScatterChart,
  ScatterSeries,
  SelectPoint,
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

const initialColumns = [0, 2, 3, 10, 11, 12];

const xAxis: Axis = {
  isTime: true,
  label: "Runtime",
};

const yAxis: Axis = {
  isTime: false,
  label: "Physical qubits",
};

function reDataToRow(input: ReData, color: string): Row {
  const data = CreateSingleEstimateResult(input, 0);
  const estimateType =
    input.frontierEntries == null
      ? "Single"
      : "Frontier (" + input.frontierEntries.length + "  items)";

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
      ", physical qubits: " +
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
            ", physical qubits: " +
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

export function EstimatesOverview(props: {
  estimatesData: ReData[];
  colors: string[] | null;
  runNames: string[] | null;
  isSimplifiedView: boolean;
  onRowDeleted: (rowId: string) => void;
  setEstimate: (estimate: SingleEstimateResult | null) => void;
}) {
  const [selectedRow, setSelectedRow] = useState<string | null>(null);

  props.estimatesData.forEach((item, idx) => {
    if (
      props.runNames != null &&
      props.runNames.length == props.estimatesData.length
    ) {
      item.jobParams.runName = props.runNames[idx];
    } else {
      if (item.jobParams.runName == null) {
        // Start indexing with 0 to match with the original object indexing.
        item.jobParams.runName = `(${idx})`;
      }
    }
  });

  function onPointSelected(seriesIndex: number, pointIndex: number): void {
    const data = props.estimatesData[seriesIndex];
    props.setEstimate(CreateSingleEstimateResult(data, pointIndex));
    const rowId = props.estimatesData[seriesIndex].jobParams.runName;
    setSelectedRow(rowId);
  }

  function onRowSelected(rowId: string) {
    setSelectedRow(rowId);
    // On any selection, clear the "new" flag on all rows. This ensures that
    // new rows do not steal focus from the user selected row.
    props.estimatesData.forEach((data) => (data.new = false));
    HideTooltip();
    if (!rowId) {
      props.setEstimate(null);
    } else {
      const index = props.estimatesData.findIndex(
        (data) => data.jobParams.runName === rowId,
      );

      if (index == -1) {
        props.setEstimate(null);
      } else {
        const estimateFound = props.estimatesData[index];
        props.setEstimate(CreateSingleEstimateResult(estimateFound, 0));
        SelectPoint(index, 0);
      }
    }
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
          onRowDeleted={props.onRowDeleted}
          selectedRow={selectedRow}
          setSelectedRow={onRowSelected}
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
          selectedRow={selectedRow}
          setSelectedRow={onRowSelected}
          ensureSelected={true}
          onRowDeleted={props.onRowDeleted}
        />
      </details>
      <details open>
        <summary style="font-size: 1.5em; font-weight: bold; margin: 24px 8px;">
          Space-time diagram
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
