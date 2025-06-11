// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { IDocFile } from "qsharp-lang";
import { Markdown } from "qsharp-lang/ux";

export function getNamespaces(
  documentation: Map<string, string> | undefined,
): string[] {
  if (documentation) {
    return Array.from(documentation.keys());
  }
  return new Array<string>();
}

// Takes array of documents (containing data for each item in the standard library)
// and creates a documentation map, which maps from a namespace
// to the combined HTML-formatted documentation for all items in that namespace.
export function processDocumentFiles(
  docFiles: IDocFile[],
): Map<string, string> {
  const contentByNamespace = new Map<string, string>();
  const regex = new RegExp(
    "^qsharp\\.namespace: (Microsoft\\.Quantum|Std)\\.(.+)$",
    "m",
  );

  for (const doc of docFiles) {
    const match = regex.exec(doc.metadata); // Parse namespace out of metadata
    if (match == null) {
      continue; // Skip items with non-parsable metadata
    }
    // The next line contains "Zero-width space" unicode character
    // to allow line breaks before the period.
    const newNamespace = "… " + match[2].replace(".", "​.");

    if (contentByNamespace.has(newNamespace)) {
      const existingContent = contentByNamespace.get(newNamespace)!;
      contentByNamespace.set(
        newNamespace,
        existingContent + "\n<br>\n<br>\n\n" + doc.contents,
      );
    } else {
      contentByNamespace.set(newNamespace, doc.contents);
    }
  }
  return contentByNamespace;
}

export function DocumentationDisplay(props: {
  currentNamespace: string;
  documentation: Map<string, string> | undefined;
}) {
  const docsMd = props.documentation?.get(props.currentNamespace) ?? "";

  return <Markdown markdown={docsMd} />;
}
