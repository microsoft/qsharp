// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { render as prender } from "preact";
import { ReTable, SpaceChart } from "qsharp-lang/ux";
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
    case "EstimateDetails":
      renderTable({ model, el });
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
