// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { Markdown } from "qsharp-lang/ux";

export function DocumentationView(props: { contentFragments: string[] }) {

  // Concatenate all documentation.
  // The following adds an empty line and a horizontal line
  // between documentation for different functions.
  const contentToRender = props.contentFragments.join("<br/>\n\n---\n\n");

  return <Markdown markdown={contentToRender} />;
}
