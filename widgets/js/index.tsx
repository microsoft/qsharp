// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { render as prender } from "preact";
import {
  ColorMap,
  ReTable,
  SpaceChart,
  Histogram,
  ScatterChart,
  xAxis,
  yAxis,
  ResultsTable,
  ColumnNames,
  ReDataToRowScatter,
  ReDataToRow,
  InitialColumns,
  ReData,
} from "qsharp-lang/ux";
import markdownIt from "markdown-it";

// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-ignore - there are no types for this
import mk from "@vscode/markdown-it-katex";

const md = markdownIt();
md.use(mk);

export function mdRenderer(input: string) {
  // Note: Need to ensure this 'fix' is still needed with the latest data JSON.
  // In early testing backslashes were being double-escaped in the results.
  return md.render(input.replace(/\\\\/g, "\\"));
}

// Param types for AnyWidget render functions
import type { AnyModel } from "@anywidget/types";

type RenderArgs = {
  model: AnyModel;
  el: HTMLElement;
};

export function render({ model, el }: RenderArgs) {
  const componentType = model.get("comp");

  switch (componentType) {
    case "SpaceChart":
      renderChart({ model, el });
      break;
    case "ScatterChart":
      renderScatterChart({ model, el });
      break;
    case "EstimateDetails":
      renderTable({ model, el });
      break;
    case "Histogram":
      renderHistogram({ model, el });
      break;
    default:
      throw new Error(`Unknown component type ${componentType}`);
  }
}

function renderTable({ model, el }: RenderArgs) {
  const onChange = () => {
    const estimates = model.get("estimates");
    prender(
      <ReTable estimatesData={estimates} mdRenderer={mdRenderer}></ReTable>,
      el,
    );
  };

  onChange();
  model.on("change:estimates", onChange);
}

function renderChart({ model, el }: RenderArgs) {
  const onChange = () => {
    const estimates = model.get("estimates");
    prender(<SpaceChart estimatesData={estimates}></SpaceChart>, el);
  };

  onChange();
  model.on("change:estimates", onChange);
}

function renderScatterChart({ model, el }: RenderArgs) {
  const onChange = () => {
    const results = model.get("estimates");

    const estimates = [];
    if (results[0] == null) {
      estimates.push(results);
    } else {
      for (const estimate of Object.values(results)) {
        estimates.push(estimate);
      }
    }

    (estimates as Array<any>).forEach(
      (item, idx) =>
        (item.jobParams.runName = `(${String.fromCharCode(0x61 + idx)})`),
    );

    const colormap = ColorMap(estimates.length);

    const rows = estimates.map((estimate: ReData, index: number) =>
      ReDataToRow(estimate, colormap[index]),
    );
    const scatterData = estimates.map((estimate: ReData, index: number) =>
      ReDataToRowScatter(estimate, colormap[index]),
    );

    prender(
      <>
        <ResultsTable
          columnNames={ColumnNames}
          initialColumns={InitialColumns}
          rows={rows}
          ensureSelected={true}
          onRowDeleted={() => undefined}
          onRowSelected={() => undefined}
        ></ResultsTable>
        <ScatterChart
          data={scatterData}
          xAxis={xAxis}
          yAxis={yAxis}
          onPointSelected={() => undefined}
        ></ScatterChart>
      </>,
      el,
    );
  };

  onChange();
  model.on("change:estimates", onChange);
}

function renderHistogram({ model, el }: RenderArgs) {
  const onChange = () => {
    const buckets = model.get("buckets") as { [key: string]: number };
    const bucketMap = new Map(Object.entries(buckets));
    const shot_count = model.get("shot_count") as number;

    prender(
      <Histogram
        data={bucketMap}
        shotCount={shot_count}
        filter={""}
        onFilter={() => undefined}
        shotsHeader={true}
      ></Histogram>,
      el,
    );
  };

  onChange();
  model.on("change:buckets", onChange);
  model.on("change:shot_count", onChange);
}
