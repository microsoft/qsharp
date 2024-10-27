// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { Markdown } from "qsharp-lang/ux";
import { useState } from "preact/hooks";

/* TODO
- Wire up search functionality
- Test single-page functionality
- How to wire up dir name (for project) or file name for top-level project?
- How to move CSS to a separate file and respond to theme?
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
  // currPath is of the format: "<pkg>/<module>/<member>", e.g.
  // "Std/Canon/CCNOT" or "Std/Microsoft.Quantum.Diagnostics/AssertMeasurementEqual" or
  // "Unsigned/Main/GetInt" or "Main/Particle/Particle". When at the top level, currPath is "".

  const [currPath, setPath] = useState("");

  const contents: {
    [name: string]: {
      onclick: (evt: Event) => void;
      content: string;
      anchor: string;
    };
  } = {};

  // Calculate the current list of content to render based on the current package and module
  if (currPath === "") {
    // Collect the set of all packages
    props.fragmentsToRender.forEach((doc) => {
      if (!(doc.pkg in contents)) {
        contents[doc.pkg] = {
          onclick: () => setPath(doc.pkg),
          content: "",
          anchor: "",
        };
      }
    });
  } else if (currPath.indexOf("/") === -1) {
    // Render the list of modules in the current package
    props.fragmentsToRender.forEach((doc) => {
      if (doc.pkg === currPath) {
        if (!(doc.module in contents)) {
          contents[doc.module] = {
            onclick: () => setPath(`${currPath}/${doc.module}`),
            content: "",
            anchor: "",
          };
        }
      }
    });
  } else {
    // Render the list of members in the current module
    props.fragmentsToRender.forEach((doc) => {
      if (
        doc.pkg === currPath.split("/")[0] &&
        doc.module === currPath.split("/")[1]
      ) {
        contents[doc.member] = {
          onclick: (evt: Event) => {
            evt.preventDefault();
            const elem = document.getElementById(doc.member)!;
            //elem.scrollIntoView({ behavior: "instant", block: "start" });
            const yOffset = -64; // Negative value to offset from the top
            const yPosition =
              elem.getBoundingClientRect().top + window.scrollY + yOffset;

            window.scrollTo({
              top: yPosition,
              behavior: "instant",
            });
          },
          content: doc.content,
          anchor: doc.member,
        };
      }
    });
  }

  function overLi(e: MouseEvent) {
    (e.target as HTMLElement).style.fontWeight = "600";
  }

  function outLi(e: MouseEvent) {
    (e.target as HTMLElement).style.fontWeight = "400";
  }

  function onPathClick() {
    if (currPath) {
      setPath(currPath.split("/")[0]);
    }
  }

  return (
    <div class="qs-docsPage" style="width: 100%; position: relative;">
      <div
        class="qs-docsHeader"
        style="height: 3em; display: flex; justify-content: space-between; align-items: center; margin-top: 0px; padding-top: 1.5em; padding-bottom: 1em; position: fixed; top: 0; width: 95%; background-color: black; z-index: 1;"
      >
        <div onClick={() => setPath("")}>
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
        <div
          style="flex-grow: 1; font-size: 1.4em; margin-left: 0.5em;"
          onClick={onPathClick}
        >
          {currPath ? currPath.replace("/", " > ") : "Q# API documentation"}
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
      <div
        class="qs-docsContent"
        style="margin: 2em; position: relative; top: 2em;"
      >
        <div class="qs-index" style="background: #161b22; padding: 0.1em">
          <p style="font-size: 1.1em; font-weight: 600; margin: 0.8em;">
            {currPath === ""
              ? "Packages"
              : currPath.indexOf("/") === -1
                ? "Modules"
                : "Members"}
          </p>
          <ul>
            {Object.keys(contents).map((key) => (
              <li
                onClick={contents[key].onclick}
                onMouseOver={overLi}
                onMouseOut={outLi}
              >
                {key}
              </li>
            ))}
          </ul>
        </div>

        {Object.keys(contents).map((key) =>
          !contents[key].content ? null : (
            <div id={contents[key].anchor} style="margin-top: 12px;">
              <Markdown markdown={contents[key].content} />
              <hr />
            </div>
          ),
        )}
      </div>
    </div>
  );
}

export function DocumentationView(props: { fragmentsToRender: IDocFile[] }) {
  const docs: ItemDocs[] = [];

  props.fragmentsToRender.forEach((doc) => {
    if (!doc.metadata) {
      return;
    }
    const pkg = doc.metadata.match(/^(?:qsharp\.package: )(.*)$/m)?.[1];
    const module = doc.metadata.match(/^(?:qsharp\.namespace: )(.*)$/m)?.[1];
    const member = doc.metadata.match(/^(?:qsharp\.name: )(.*)$/m)?.[1];

    if (pkg && module && member) {
      docs.push({
        pkg:
          pkg === "__Core__" || pkg === "__Std__"
            ? "Std"
            : pkg === "__Main__"
              ? "Main"
              : pkg,
        module,
        member,
        content: doc.contents,
      });
    }
  });

  docs.sort((a, b) => {
    if (a.pkg != b.pkg) {
      // Sorted by __Main__, then reference packages, then __Std__ (which includes __Core__)
      if (a.pkg === "Main" || b.pkg === "Std") {
        return -1;
      } else if (b.pkg === "Main" || a.pkg === "Std") {
        return 1;
      } else {
        return a.pkg.localeCompare(b.pkg);
      }
    } else if (a.module != b.module) {
      // Main module comes first and "Microsoft.Quantum.*" comes last
      if (a.module === "Main" || b.module.startsWith("Microsoft.Quantum")) {
        return -1;
      } else if (
        b.module === "Main" ||
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

  // document.documentElement.style.height = "100%";
  // document.body.style.height = "100%";
  // document.documentElement.style.overflow = "hidden";
  // document.body.style.overflow = "hidden";

  return <DocsPage fragmentsToRender={docs} />;
}
