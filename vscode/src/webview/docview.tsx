// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { Markdown } from "qsharp-lang/ux";

/* TODO
- Flesh out the page structure with Home, breadcrum, search text and icon, index, and content.
- When at the top level, show the list of packages, and default select the __Main__ package.
- When at the package level, show the list of modules in that package, and render the __Main__ module by default.
- When at the module level, show the list of submodules, show the list of members in that module
- Add a search bar and search results
*/

export interface IDocFile {
  filename: string;
  metadata: string;
  contents: string;
}

interface ItemDocs {
  pkg: string;
  module: string;
  member: string;
  content: string;
}

function DocsPage(props: { fragmentsToRender: ItemDocs[] }) {
  const filtered = props.fragmentsToRender.filter((x) =>
    x.module.startsWith("Std.Canon"),
  );

  return (
    <div
      class="qs-docsPage"
      style="width: 100%; position: relative; padding-top: 0.1px;"
    >
      <div
        class="qs-docsHeader"
        style="height: 3em; display: flex; justify-content: space-between; align-items: center;position: fixed; width: 95%; background-color: #0d1117;"
      >
        <div>
          <svg
            style="height: 2.25em; width: 2.25em; margin: 0.25em"
            viewBox="0 0 100 100"
          >
            <g transform="translate(10,10)" fill="#888">
              <path d="M10,50 l40,-40 l40,40 Z"></path>
              <path d="M20,49 l0,30 l20,0 l0,-30Z"></path>
              <path d="M60,49 l0,30 l20,0 l0,-30Z"></path>
              <path d="M39,49 l0,8 l24,0 l0,-30Z"></path>
              <path d="M77,40 l0,-20 l-10,0 l0,20"></path>
            </g>
          </svg>
        </div>
        <div style="flex-grow: 1; font-size: 1.4em; margin-left: 0.5em;">
          Std &gt; Canon
        </div>
        <div>
          <input type="text" placeholder="Search..." />
        </div>
        <div>
          <svg
            style="height: 2.25em; width: 2.25em; margin: 0.25em"
            viewBox="0 0 100 100"
          >
            <g stroke="#888" fill="none" transform="translate(0,10)">
              <circle stroke-width="6" cx="40" cy="35" r="25"></circle>
              <path stroke-width="8" d="M52,57 l20,30"></path>
            </g>
          </svg>
        </div>
      </div>
      <div class="qs-docsContent" style="margin: 2em; margin-top: 3.25em">
        <div class="qs-index" style="background: #161b22; padding: 0.1em">
          <p style="font-size: 1.1em; font-weight: 600; margin: 0.8em;">
            Contents
          </p>
          <ul>
            {filtered.map((doc) => (
              <li>{doc.member}</li>
            ))}
          </ul>
        </div>

        {filtered.map((doc) => (
          <Markdown markdown={doc.content} />
        ))}
      </div>
    </div>
  );
}

export function DocumentationView(props: { fragmentsToRender: IDocFile[] }) {
  const docs: ItemDocs[] = [];

  let currentPkg = "";
  let currentModule = "";

  props.fragmentsToRender.forEach((doc) => {
    if (!doc.metadata) {
      return;
    }
    const pkg = doc.metadata.match(/^(?:qsharp\.package: )(.*)$/m)?.[1];
    const module = doc.metadata.match(/^(?:qsharp\.namespace: )(.*)$/m)?.[1];
    const member = doc.metadata.match(/^(?:qsharp\.name: )(.*)$/m)?.[1];

    if (pkg && module && member) {
      docs.push({
        pkg: pkg === "__Core__" ? "__Std__" : pkg, // Treat Core like Std
        module,
        member,
        content: doc.contents,
      });
    }
  });
  console.log(docs);

  docs.sort((a, b) => {
    if (a.pkg != b.pkg) {
      // Sorted by __Main__, then reference packages, then __Std__ (which includes __Core__)
      if (a.pkg === "__Main__" || b.pkg === "__Std__") {
        return -1;
      } else if (b.pkg === "__Main__" || a.pkg === "__Std__") {
        return 1;
      } else {
        return a.pkg.localeCompare(b.pkg);
      }
    } else if (a.module != b.module) {
      // Main module comes first and "Microsoft.Quantum.*" comes last
      if (a.module === "__Main__" || b.module.startsWith("Microsoft.Quantum")) {
        return -1;
      } else if (
        b.module === "__Main__" ||
        a.module.startsWith("Microsoft.Quantum")
      ) {
        return 1;
      } else {
        return a.module.localeCompare(b.module);
      }
    } else {
      return a.member.localeCompare(b.member);
    }
  });

  //return <Markdown markdown={contentToRender} />;
  return <DocsPage fragmentsToRender={docs} />;
}
