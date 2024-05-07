// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { Markdown } from "qsharp-lang/ux";

export function DocumentationView(props: { contentToRender: string }) {
  return <Markdown markdown={props.contentToRender} />;
}
