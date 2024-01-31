// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { render as prender } from "preact";
import {
  ReTable,
  SpaceChart,
  Histogram,
  CreateSingleEstimateResult,
  EstimatesOverview,
  EstimatesPanel,
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
    case "EstimatesOverview":
      renderEstimatesOverview({ model, el });
      break;
    case "EstimateDetails":
      renderTable({ model, el });
      break;
    case "Histogram":
      renderHistogram({ model, el });
      break;
    case "EstimatesPanel":
      renderEstimatesPanel({ model, el });
      break;
    default:
      throw new Error(`Unknown component type ${componentType}`);
  }
}

function renderTable({ model, el }: RenderArgs) {
  const onChange = () => {
    const estimates = model.get("estimates");
    const index = model.get("index");
    const singleEstimateResult = CreateSingleEstimateResult(estimates, index);
    prender(
      <ReTable
        estimatesData={singleEstimateResult}
        mdRenderer={mdRenderer}
      ></ReTable>,
      el,
    );
  };

  onChange();
  model.on("change:estimates", onChange);
  model.on("change:index", onChange);
}

function renderChart({ model, el }: RenderArgs) {
  const onChange = () => {
    const estimates = model.get("estimates");
    const index = model.get("index");
    const singleEstimateResult = CreateSingleEstimateResult(estimates, index);
    prender(<SpaceChart estimatesData={singleEstimateResult}></SpaceChart>, el);
  };

  onChange();
  model.on("change:estimates", onChange);
  model.on("change:index", onChange);
}

function renderEstimatesOverview({ model, el }: RenderArgs) {
  const onChange = () => {
    const results = model.get("estimates");
    const colors = model.get("colors");
    const runNames = model.get("runNames");

    let estimates = [];
    if (results[0] == null) {
      estimates.push(results);
    } else {
      for (const estimate of Object.values(results)) {
        estimates.push(estimate);
      }
    }

    const onRowDeleted = createOnRowDeleted(estimates, (newEstimates) => {
      estimates = newEstimates;
      model.set("estimates", estimates);
    });

    prender(
      <EstimatesOverview
        estimatesData={estimates}
        runNames={runNames}
        colors={colors}
        isSimplifiedView={true}
        onRowDeleted={onRowDeleted}
        setEstimate={() => undefined}
      ></EstimatesOverview>,
      el,
    );
  };

  onChange();
  model.on("change:estimates", onChange);
  model.on("change:colors", onChange);
  model.on("change:runNames", onChange);
}

function renderEstimatesPanel({ model, el }: RenderArgs) {
  const onChange = () => {
    const results = model.get("estimates");
    const colors = model.get("colors");
    const runNames = model.get("runNames");

    let estimates: ReData[] = [];
    if (results[0] == null) {
      estimates.push(results);
    } else {
      for (const estimate of Object.values(results)) {
        estimates.push(estimate as ReData);
      }
    }

    const onRowDeleted = createOnRowDeleted(estimates, (newEstimates) => {
      estimates = newEstimates;
      model.set("estimates", estimates);
    });

    prender(
      <EstimatesPanel
        estimatesData={estimates}
        runNames={runNames}
        colors={colors}
        renderer={mdRenderer}
        calculating={false}
        onRowDeleted={onRowDeleted}
      ></EstimatesPanel>,
      el,
    );
  };

  onChange();
  model.on("change:estimates", onChange);
  model.on("change:colors", onChange);
  model.on("change:runNames", onChange);
}

function createOnRowDeleted(
  estimates: ReData[],
  setEstimates: (estimates: ReData[]) => void,
) {
  return (rowId: string) => {
    // Clone estimates into a new object
    const newEstimates = JSON.parse(JSON.stringify(estimates)) as ReData[];

    // Splice out the estimate that was deleted
    const index = newEstimates.findIndex(
      (estimate) => estimate.jobParams.runName === rowId,
    );
    if (index >= 0) {
      newEstimates.splice(index, 1);
    }

    setEstimates(newEstimates);
  };
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
