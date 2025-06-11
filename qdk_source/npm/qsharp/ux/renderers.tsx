// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { createElement } from "preact";

// Default renderer to be replaced before components are first used.
// Expectation is that this will convert markdown and/or LaTeX to HTML.
let theRenderer = function (input: string): string {
  const err = "ERROR: Rendered has not been set";
  console.error(err);
  return err + ". " + input;
};

export function setRenderer(renderer: (input: string) => string) {
  theRenderer = renderer;
}

export function Markdown(props: {
  markdown: string;
  className?: string;
  tagName?: string;
}) {
  const tag = props.tagName || "div";
  const nodeProps = {
    className: props.className,
    dangerouslySetInnerHTML: {
      __html: theRenderer(props.markdown),
    },
  };

  return createElement(tag, nodeProps);
}
