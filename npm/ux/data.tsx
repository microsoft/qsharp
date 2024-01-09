// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.
// Data structures and functions for the scatter chart and the results table.

import colormap from "colormap";
import { FrontierEntry, ReData } from "./reTable.js";
import { Row } from "./resultsTable.js";
import { Axis, PlotItem, ScatterSeries } from "./scatterChart.js";

export const ColumnNames = [
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

export const InitialColumns = [0, 1, 2, 3, 4, 10, 11, 12];

export const xAxis: Axis = {
  isTime: true,
  label: "Runtime",
};

export const yAxis: Axis = {
  isTime: false,
  label: "Physical Qubits",
};

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

export function GetColor(index: number, totalNumber: number) {
  if (totalNumber != colors.length) {
    if (totalNumber > predefinedColors.length) {
      colors = colormap({
        colormap: "jet",
        nshades: Math.max(6, totalNumber), // 6 is the minimum number of colors in the colormap 'jet'
        format: "hex",
        alpha: 1,
      });
    } else {
      colors = predefinedColors;
    }
  }
  return colors[index];
}

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

export function ReDataToRow(input: ReData, color: string): Row {
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

export function ReDataToRowScatter(data: ReData, color: string): ScatterSeries {
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
