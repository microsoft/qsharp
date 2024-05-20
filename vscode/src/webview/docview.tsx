// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { Markdown } from "qsharp-lang/ux";

export function DocumentationView(props: { fragmentsToRender: string[] }) {
  // Concatenate all documentation.
  // The following adds an empty line and a horizontal line
  // between documentation for different functions.
  // We may consider filtering of fragments later.
  const contentToRender = props.fragmentsToRender.join("<br/>\n\n---\n\n");

  return <Markdown markdown={contentToRender} />;
}
