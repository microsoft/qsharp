// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// Default renderer to be replaced before components are first used.
// Expectation is that this will convert markdown and/or LaTeX to HTML.
let the_renderer = function (input: string): string {
  const err = "ERROR: Rendered has not been set";
  console.error(err);
  return err + ". " + input;
};

export function setRenderer(renderer: (input: string) => string) {
  the_renderer = renderer;
}

export function RenderDiv(props: { input: string; className?: string }) {
  return (
    <div
      className={props.className || ""}
      dangerouslySetInnerHTML={{ __html: the_renderer(props.input) }}
    />
  );
}

export function RenderLi(props: { input: string; className?: string }) {
  return (
    <li
      className={props.className || ""}
      dangerouslySetInnerHTML={{ __html: the_renderer(props.input) }}
    />
  );
}
