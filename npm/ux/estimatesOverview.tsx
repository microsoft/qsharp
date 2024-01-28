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
import { Axis, PlotItem, ScatterChart, ScatterSeries } from "./scatterChart.js";

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

const initialColumns = [0, 10, 13, 11, 12];

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

function createRunNames(estimatesData: ReData[]): string[] {
  // If there's only 1 entry, use the shared run name
  if (estimatesData.length === 1) {
    return [estimatesData[0].jobParams.sharedRunName];
  }

  const fields: string[][] = [];

  estimatesData.forEach(() => {
    fields.push([]);
  });

  // Could be multiple runs, e.g. against different algorithms.
  addIfDifferent(fields, estimatesData, (data) => data.jobParams.sharedRunName);

  addIfDifferent(
    fields,
    estimatesData,
    (data) => data.jobParams.qubitParams.name,
  );

  addIfDifferent(
    fields,
    estimatesData,
    (data) => data.jobParams.qecScheme.name,
  );

  addIfDifferent(fields, estimatesData, (data) =>
    String(data.jobParams.errorBudget),
  );

  const proposedRunNames = fields.map((field) => field.join(", "));
  if (new Set(proposedRunNames).size != proposedRunNames.length) {
    // If there are duplicates, add the run index to the name.
    return proposedRunNames.map(
      (runName, index) => runName + " (" + index + ")",
    );
  }

  return proposedRunNames;
}

function addIfDifferent(
  fields: string[][],
  estimatesData: ReData[],
  fieldMethod: (data: ReData) => string,
): void {
  const arr = estimatesData.map(fieldMethod);

  const set = new Set(arr);
  if (set.size > 1) {
    arr.forEach((field, index) => {
      fields[index].push(field);
    });
  }
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
  const [selectedPoint, setSelectedPoint] = useState<[number, number]>();

  const runNameRenderingError =
    props.runNames != null &&
    props.runNames.length > 0 &&
    props.runNames.length != props.estimatesData.length
      ? "Warning: The number of runNames does not match the number of estimates. Ignoring provided runNames."
      : "";

  const runNames =
    props.runNames != null &&
    props.runNames.length == props.estimatesData.length
      ? props.runNames
      : createRunNames(props.estimatesData);

  props.estimatesData.forEach((item, idx) => {
    // Start indexing with 0 to match with the original object indexing.
    item.jobParams.runName = runNames[idx];
  });

  function onPointSelected(seriesIndex: number, pointIndex: number): void {
    if (seriesIndex < 0) {
      // Point was deselected
      onRowSelected("");
      return;
    }

    const data = props.estimatesData[seriesIndex];
    props.setEstimate(CreateSingleEstimateResult(data, pointIndex));
    const rowId = props.estimatesData[seriesIndex].jobParams.runName;
    setSelectedRow(rowId);
    setSelectedPoint([seriesIndex, pointIndex]);
  }

  function onRowSelected(rowId: string) {
    setSelectedRow(rowId);
    if (!rowId) {
      props.setEstimate(null);
      setSelectedPoint(undefined);
    } else {
      const index = props.estimatesData.findIndex(
        (data) => data.jobParams.runName === rowId,
      );

      if (index == -1) {
        props.setEstimate(null);
        setSelectedPoint(undefined);
      } else {
        const estimateFound = props.estimatesData[index];
        setSelectedPoint([index, 0]);
        props.setEstimate(CreateSingleEstimateResult(estimateFound, 0));
      }
    }
  }

  const colorRenderingError =
    props.colors != null &&
    props.colors.length > 0 &&
    props.colors.length != props.estimatesData.length
      ? "Warning: The number of colors does not match the number of estimates. Ignoring provided colors."
      : "";

  const colormap =
    props.colors != null && props.colors.length == props.estimatesData.length
      ? props.colors
      : ColorMap(props.estimatesData.length);

  function getResultTable() {
    return (
      <ResultsTable
        columnNames={columnNames}
        rows={props.estimatesData.map((dataItem, index) =>
          reDataToRow(dataItem, colormap[index]),
        )}
        initialColumns={initialColumns}
        selectedRow={selectedRow}
        onRowSelected={onRowSelected}
        onRowDeleted={props.onRowDeleted}
      />
    );
  }

  function getScatterChart() {
    return (
      <ScatterChart
        xAxis={xAxis}
        yAxis={yAxis}
        data={props.estimatesData.map((dataItem, index) =>
          reDataToRowScatter(dataItem, colormap[index]),
        )}
        onPointSelected={onPointSelected}
        selectedPoint={selectedPoint}
      />
    );
  }

  return (
    <div className="qs-estimatesOverview">
      {runNameRenderingError != "" && (
        <div class="qs-estimatesOverview-error">{runNameRenderingError}</div>
      )}
      {colorRenderingError != "" && (
        <div class="qs-estimatesOverview-error">{colorRenderingError}</div>
      )}
      {!props.isSimplifiedView ? (
        <>
          <details open>
            <summary style="font-size: 1.5em; font-weight: bold; margin: 24px 8px;">
              Results
            </summary>
            {getResultTable()}
          </details>
          <details open>
            <summary style="font-size: 1.5em; font-weight: bold; margin: 24px 8px;">
              Space-time diagram
            </summary>
            {getScatterChart()}
          </details>
        </>
      ) : (
        <>
          {getResultTable()}
          {getScatterChart()}
        </>
      )}
    </div>
  );
}
