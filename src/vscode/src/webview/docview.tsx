// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { Markdown } from "qsharp-lang/ux";
import { useEffect, useState } from "preact/hooks";

export interface IDocFile {
  filename: string;
  metadata: string;
  contents: string;
}

interface ItemDocs {
  pkg: string;
  namespace: string;
  member: string;
  content: string;
}

interface PageContents {
  [name: string]: {
    onclick: (evt: Event) => void;
    content: string;
    anchor: string;
  };
}

function GetPageContents(
  currPath: string,
  docs: ItemDocs[],
  setPath: any,
): PageContents {
  const contents: PageContents = {};

  if (currPath === "") {
    // Collect the set of all packages
    docs.forEach((doc) => {
      if (!(doc.pkg in contents)) {
        contents[doc.pkg] = {
          onclick: () => setPath(doc.pkg),
          content: "",
          anchor: "",
        };
      }
    });
  } else if (currPath.indexOf("/") === -1) {
    // Render the list of namespaces in the current package
    docs.forEach((doc) => {
      if (doc.pkg === currPath) {
        if (!(doc.namespace in contents)) {
          contents[doc.namespace] = {
            onclick: () => setPath(`${currPath}/${doc.namespace}`),
            content: "",
            anchor: "",
          };
        }
      }
    });
  } else {
    // Render the list of members in the current namespace
    docs.forEach((doc) => {
      if (
        doc.pkg === currPath.split("/")[0] &&
        doc.namespace === currPath.split("/")[1]
      ) {
        contents[doc.member] = {
          onclick: (evt: Event) => {
            evt.preventDefault();
            scrollToElement(doc.member);
          },
          content: doc.content,
          anchor: doc.member,
        };
      }
    });
  }
  return contents;
}

function scrollToElement(id: string) {
  const elem = id ? document.getElementById(id) : null;
  if (!elem) {
    window.scrollTo({
      top: 0,
      behavior: "instant",
    });
  } else {
    const yOffset = -64; // Negative value to offset from the top
    const yPosition =
      elem.getBoundingClientRect().top + window.scrollY + yOffset;

    window.scrollTo({
      top: yPosition,
      behavior: "instant",
    });
  }
}

interface SearchResult {
  rank: number;
  anchor: string;
  header: string;
  summary: string;
}

function getSearchResults(
  searchText: string,
  docs: ItemDocs[],
): SearchResult[] {
  const results: SearchResult[] = [];

  // Search on member name first, then on namespace name, then on package name, then on content
  // Prefer earlier matches (e.g. "X" should match "X" before "CX")

  // RegExp groups
  // 1. is the first header (e.g. 'Foo operation'), group
  // 2. Is any text after the first header that precedes the next header
  // 3. Ignore
  // 4. Is the summary (if present)
  const summaryRe =
    /^(?:# )(.*?\n)\s*(.*?)(?=(##+)|$)(## Summary\r?\n.*?(?=(##+)|$))?/s;

  docs.forEach((doc) => {
    const reMatch = doc.content.match(summaryRe);
    const summary = (reMatch?.[2] ?? "") + (reMatch?.[4] ?? "");
    const header = doc.content.match(summaryRe)?.[1] ?? doc.member;

    const lowerText = searchText.toLowerCase();
    let rank = doc.member.toLowerCase().indexOf(lowerText) + 1;
    if (rank) {
      // Matches on larger parts of the name are more important
      rank += doc.member.length / 100;
    }

    if (!rank) {
      rank = (doc.namespace.toLowerCase().indexOf(lowerText) + 1) * 100;
      if (rank) {
        rank += doc.namespace.length;
      }
    }
    if (!rank) {
      rank = (doc.pkg.toLowerCase().indexOf(lowerText) + 1) * 1000;
    }
    if (!rank) {
      rank = (doc.content.toLowerCase().indexOf(lowerText) + 1) * 10000;
    }

    if (rank) {
      results.push({
        rank,
        anchor: `${doc.pkg}/${doc.namespace}/${doc.member}`,
        header,
        summary,
      });
      return;
    }
  });

  return results.sort((a, b) => a.rank - b.rank);
}

function DocsPage(props: { fragmentsToRender: ItemDocs[] }) {
  // currPath is of the format: "<pkg>/<namespace>/<member>", e.g.
  // "Std/Canon/CCNOT" or "Std/Microsoft.Quantum.Diagnostics/AssertMeasurementEqual" or
  // "Unsigned/Main/GetInt" or "Main/Particle/Particle". When at the top level, currPath is "".

  const [currPath, setPath] = useState("");
  const [searchText, setSearchText] = useState("");

  useEffect(() => {
    // Update the xref links to navigate to the correct member
    document.querySelectorAll('a[href^="xref:"]').forEach((a) => {
      const anchor = a as HTMLAnchorElement;
      const xref = "xref:";
      let link = anchor.href.slice(xref.length);
      link = link.replaceAll(".", "/");
      // Just for Qdk links, we want to strip out the leading "Qdk."
      if (link.startsWith("Qdk/")) {
        link = link.slice(4);
      }
      a.addEventListener("click", (e) => {
        e.preventDefault();
        setPath(link);
      });
    });

    // If the member is navigated to, scroll to it after rendering
    const member = currPath.split("/")[2];
    scrollToElement(member);
  }, [currPath]);

  // Skip processing contents if searching
  const contents =
    searchText === ""
      ? GetPageContents(currPath, props.fragmentsToRender, setPath)
      : {};

  const searchResults =
    searchText === ""
      ? []
      : getSearchResults(searchText, props.fragmentsToRender);

  // Used to bold the text links when hovering
  function overLi(e: MouseEvent) {
    (e.target as HTMLElement).style.fontWeight = "600";
    (e.target as HTMLElement).style.textDecoration = "underline";
    (e.target as HTMLElement).style.cursor = "pointer";
  }

  function outLi(e: MouseEvent) {
    (e.target as HTMLElement).style.fontWeight = "400";
    (e.target as HTMLElement).style.textDecoration = "none";
    (e.target as HTMLElement).style.cursor = "default";
  }

  // Whenever the breadcrumbs are clicked, go up one level
  function onPathClick() {
    setSearchText("");
    if (currPath) {
      const parts = currPath.split("/");
      parts.pop();
      setPath(parts.join("/"));
    }
  }

  // Handle the user focusing or updating the search box
  function onSearchFocus(e: FocusEvent) {
    e.preventDefault();
    if (e.target) {
      const currText = (e.target as HTMLInputElement).value;
      if (currText) setPath("");
      setSearchText(currText);
    }
  }

  function onSearchInput(e: InputEvent) {
    e.preventDefault();
    if (e.target) {
      setPath("");
      setSearchText((e.target as HTMLInputElement).value);
    }
  }

  function searchResultClicked(anchor: string) {
    setSearchText("");
    setPath(anchor);
  }

  return (
    <div
      class="qs-docsPage"
      style="width: 100%; padding-bottom: 1em; position: relative; background-color: var(--main-background); color: var(--main-color)"
    >
      <div
        class="qs-docsHeader"
        style="height: 3em; display: flex; justify-content: space-between; align-items: center; margin-top: 0px; padding-top: 1.5em; padding-bottom: 1em; position: fixed; top: 0; width: 95%; background-color: var(--main-background); z-index: 1;"
      >
        <div
          onClick={() => {
            setSearchText("");
            setPath("");
          }}
          onMouseOver={overLi}
          onMouseOut={outLi}
        >
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
          {currPath
            ? currPath.replaceAll("/", " > ")
            : searchText
              ? "Search results"
              : "Q# API documentation"}
        </div>
        <div>
          <input
            type="text"
            placeholder="Search..."
            onFocus={onSearchFocus}
            onInput={onSearchInput}
          />
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
      {searchText ? (
        <div
          class="qs-searchResults"
          style="margin: 2em; position: relative; top: 2em;"
        >
          {searchResults.map((result) => (
            <div>
              <hr />
              <h1
                onMouseOver={overLi}
                onMouseOut={outLi}
                onClick={() => searchResultClicked(result.anchor)}
              >
                {result.header}
              </h1>
              <Markdown markdown={result.summary} />
            </div>
          ))}
        </div>
      ) : (
        <div
          class="qs-docsContent"
          style="margin: 2em; position: relative; top: 2em;"
        >
          <div
            class="qs-index"
            style="background: var(--vscode-textCodeBlock-background); padding: 0.1em"
          >
            <p style="font-size: 1.1em; font-weight: 600; margin: 0.8em;">
              {currPath === ""
                ? "Packages"
                : currPath.indexOf("/") === -1
                  ? "Namespaces"
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
      )}
    </div>
  );
}

export function DocumentationView(props: {
  fragmentsToRender: IDocFile[];
  projectName: string;
}) {
  const docs: ItemDocs[] = [];

  props.fragmentsToRender.forEach((doc) => {
    if (!doc.metadata) {
      return;
    }
    const pkg = doc.metadata.match(/^(?:qsharp\.package: )(.*)$/m)?.[1];
    const namespace = doc.metadata.match(/^(?:qsharp\.namespace: )(.*)$/m)?.[1];
    const member = doc.metadata.match(/^(?:qsharp\.name: )(.*)$/m)?.[1];

    if (pkg && namespace && member) {
      docs.push({
        pkg:
          pkg === "__Core__" || pkg === "__Std__"
            ? "Std"
            : pkg === "__Main__"
              ? props.projectName
              : pkg,
        // For some reason Std namespaces include the package name. Remove it.
        namespace: namespace.startsWith("Std.")
          ? namespace.slice(4)
          : namespace,
        member,
        content: doc.contents,
      });
    }
  });

  docs.sort((a, b) => {
    if (a.pkg != b.pkg) {
      // Sorted by __Main__, then reference packages, then __Std__ (which includes __Core__)
      if (a.pkg === props.projectName || b.pkg === "Std") {
        return -1;
      } else if (b.pkg === props.projectName || a.pkg === "Std") {
        return 1;
      } else {
        return a.pkg.localeCompare(b.pkg);
      }
    } else if (a.namespace != b.namespace) {
      // Main namespace comes first and "Microsoft.Quantum.*" comes last
      if (
        a.namespace === props.projectName ||
        b.namespace.startsWith("Microsoft.Quantum")
      ) {
        return -1;
      } else if (
        b.namespace === props.projectName ||
        a.namespace.startsWith("Microsoft.Quantum")
      ) {
        return 1;
      } else {
        return a.namespace.localeCompare(b.namespace);
      }
    } else {
      return a.member.localeCompare(b.member);
    }
  });

  const style = document.createElement("style");
  style.textContent = `
body {
  background-color: var(--main-background);
  margin: 0;
  padding: 0;
}

.markdown-body {
  background-color: var(--main-background);
}

.markdown-body code {
  color: var(--main-color);
}
`;
  document.head.appendChild(style);

  return <DocsPage fragmentsToRender={docs} />;
}
