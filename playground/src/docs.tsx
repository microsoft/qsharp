// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { useEffect, useRef } from "preact/hooks";
import { DocFile } from "../../npm/qsharp/dist/compiler/compiler.js";
import markdownit from "markdown-it";

export function getDocumentNames(documentation: Map<string, string> | undefined): string[] {
  if (documentation) {
    return Array.from(documentation.keys());
  }
  return new Array<string>();
}

export function processDocumentFiles(docFiles: DocFile[]): Map<string, string> {
  const md = markdownit();
  const contentByNamespace = new Map<string, string>();
  const regex = new RegExp("^qsharp.namespace: (.+)$", "m");

  for (const doc of docFiles) {
    const match = regex.exec(doc.metadata);
    if (match == null) {
      continue;
    }
    const newNamespace = match[1];
    const newContent = md.render(doc.contents);

    if (contentByNamespace.has(newNamespace)) {
      const existingContent = contentByNamespace.get(newNamespace)!;
      contentByNamespace.set(newNamespace,
        existingContent + "\n<br>\n<br>\n" + newContent);
    } else {
      contentByNamespace.set(newNamespace, newContent);
    }
  }
  return contentByNamespace;
}

export function DocumentationDisplay(props: {
  currentDocument: string;
  documentation: Map<string, string> | undefined }) {

  const docsDiv = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!docsDiv.current) return;
    docsDiv.current.innerHTML = props.documentation!.get(props.currentDocument)!;
  }, [props.currentDocument]);

  return <div ref={docsDiv}></div>;
}
