// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { useEffect, useRef } from "preact/hooks";
import {
  CompilerState,
  ICompilerWorker,
  Kata,
  KataN,
  QscEventTarget,
} from "qsharp";
import { Editor } from "./editor.js";
import { OutputTabs } from "./tabs.js";

export function Kata(props: {
  kata: KataN;
  compiler: ICompilerWorker;
  compilerState: CompilerState;
  onRestartCompiler: () => void;
}) {
  const kataContent = useRef<HTMLDivElement>(null);
  const itemContent = useRef<(HTMLDivElement | null)[]>([]);

  // Need to keep around QscEventTargets around on re-render unless the Kata changes.
  const lastKata = useRef<KataN>();
  const handlerMap = useRef<QscEventTarget[]>();
  if (lastKata.current !== props.kata) {
    lastKata.current = props.kata;

    // This gives an extra EventTarget we don't need for 'reading' types, but that's fine.
    handlerMap.current = props.kata.sections.map(
      () => new QscEventTarget(true)
    );
  }
  const itemEvtHandlers = handlerMap.current || [];

  useEffect(() => {
    // MathJax rendering inside of React components seems to mess them up a bit,
    // so we'll take control of it here and ensure the contents are replaced.
    if (!kataContent.current) return;

    props.kata.sections.forEach((section, idx) => {
      console.log(section.type);
      const parentDiv = itemContent.current[idx];
      const div = parentDiv?.querySelector(".kata-item-content");
      if (!div) return;
      if (section.type === "text") {
        console.log("text");
        div.innerHTML = section.contentAsHtml;
      } else if (section.type === "exercise") {
        console.log(section.id);
        div.innerHTML = section.solutionDescriptionAsHtml;
      } else {
        console.log(section.id);
        div.innerHTML = "";
      }

      console.log(div.innerHTML.length);
    });
    // In case we're now rendering less items than before, be sure to truncate
    itemContent.current.length = props.kata.sections.length;

    MathJax.typeset();
  }, [props.kata]);

  return (
    <div class="markdown-body kata-override">
      <div ref={kataContent}></div>
      <br></br>
      {props.kata.sections.map((section, idx) => {
        if (section.type === "text") {
          return (
            <div ref={(elem) => (itemContent.current[idx] = elem)}>
              <div class="kata-item-content"></div>
            </div>
          );
        } else if (section.type === "example" || section.type === "exercise") {
          return (
            <div>
              <Editor
                defaultShots={1}
                showExpr={false}
                showShots={false}
                evtTarget={itemEvtHandlers[idx]}
                compiler={props.compiler}
                compilerState={props.compilerState}
                onRestartCompiler={props.onRestartCompiler}
                code={
                  section.type === "exercise"
                    ? section.placeholderCode
                    : section.code
                }
                kataExercise={section.type === "exercise" ? section : undefined}
                key={section.id}
                setHir={() => ({})}
                activeTab="results-tab"
              ></Editor>
              <OutputTabs
                key={section.id + "-results"}
                evtTarget={itemEvtHandlers[idx]}
                showPanel={false}
                kataMode={true}
                hir=""
                activeTab="results-tab"
                setActiveTab={() => undefined}
              ></OutputTabs>
              <div ref={(elem) => (itemContent.current[idx] = elem)}>
                <div class="kata-item-content"></div>
              </div>
            </div>
          );
        }
      })}
    </div>
  );
}
