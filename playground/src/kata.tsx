// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { useEffect, useRef, useState } from "preact/hooks";
import {
  CompilerState,
  ExplainedSolution,
  ICompilerWorker,
  ILanguageServiceWorker,
  Kata,
  Lesson,
  QscEventTarget,
  Question,
  VSDiagnostic,
} from "qsharp-lang";
import { Editor } from "./editor.js";
import { OutputTabs } from "./tabs.js";

function ExplainedSolutionAsHtml(solution: ExplainedSolution): string {
  let html = "<details>";
  html += `<summary>💡 Solution</summary>`;
  for (const item of solution.items) {
    switch (item.type) {
      case "example":
      case "solution":
        html += `<pre><code>${item.code}</code></pre>`;
        break;
      case "text-content":
        html += `<div>${item.asHtml}</div>`;
        break;
    }
  }
  html += `</details>`;
  return html;
}

function LessonAsHtml(lesson: Lesson): string {
  let html = "";
  for (const item of lesson.items) {
    switch (item.type) {
      case "example":
        html += `<pre><code>${item.code}</code></pre>`;
        break;
      case "text-content":
        html += `<div>${item.asHtml}</div>`;
        break;
      case "question":
        html += QuestionAsHtml(item);
        break;
    }
  }
  return html;
}

function QuestionAsHtml(question: Question): string {
  let html = "<h2>❓ Question:</h2>";
  html += `<div>${question.description.asHtml}</div>`;
  html += "<div>";
  html += "<details>";
  html += "<summary><strong>💡 Answer</strong></summary>";
  for (const item of question.answer.items) {
    switch (item.type) {
      case "example":
        html += `<pre><code>${item.code}</code></pre>`;
        break;
      case "text-content":
        html += `<div>${item.asHtml}</div>`;
        break;
    }
  }
  html += "</details>";
  html += "</div>";
  return html;
}

export function Kata(props: {
  kata: Kata;
  compiler: ICompilerWorker;
  compilerState: CompilerState;
  onRestartCompiler: () => void;
  languageService: ILanguageServiceWorker;
}) {
  const kataContent = useRef<HTMLDivElement>(null);
  const itemContent = useRef<(HTMLDivElement | null)[]>([]);

  // Need to keep around QscEventTargets around on re-render unless the Kata changes.
  const lastKata = useRef<Kata>();
  const handlerMap = useRef<QscEventTarget[]>();
  if (lastKata.current !== props.kata) {
    lastKata.current = props.kata;

    // This gives an extra EventTarget we don't need for 'reading' types, but that's fine.
    handlerMap.current = props.kata.sections.map(
      () => new QscEventTarget(true),
    );
  }
  const itemEvtHandlers = handlerMap.current || [];

  const [shotError, setShotError] = useState<VSDiagnostic | undefined>(
    undefined,
  );

  useEffect(() => {
    // MathJax rendering inside of React components seems to mess them up a bit,
    // so we'll take control of it here and ensure the contents are replaced.
    if (!kataContent.current) return;

    props.kata.sections.forEach((section, idx) => {
      const parentDiv = itemContent.current[idx];
      let titlePrefix = "🐛";
      if (section.type === "exercise") {
        titlePrefix = "⌨ Exercise: ";
        const descriptionDiv = parentDiv?.querySelector(
          ".exercise-description",
        );
        if (!descriptionDiv)
          throw new Error("exercise-description div not found");
        descriptionDiv.innerHTML = section.description.asHtml;
        const solutionDiv = parentDiv?.querySelector(".exercise-solution");
        if (!solutionDiv) throw new Error("exercise-solution div not found");
        solutionDiv.innerHTML = ExplainedSolutionAsHtml(
          section.explainedSolution,
        );
      } else if (section.type === "lesson") {
        titlePrefix = "📖 Lesson: ";
        const contentDiv = parentDiv?.querySelector(".kata-text-content");
        if (!contentDiv) throw new Error("kata-text-content div not found");
        contentDiv.innerHTML = LessonAsHtml(section);
      } else {
        throw new Error(`Unexpected section`);
      }

      const titleDiv = parentDiv?.querySelector(".section-title");
      if (!titleDiv) throw new Error("section-title div not found");
      titleDiv.innerHTML = `<h1>${titlePrefix} <u>${section.title}</u></h1>`;
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
        if (section.type === "lesson") {
          return (
            <div ref={(elem) => (itemContent.current[idx] = elem)}>
              <div class="section-title"></div>
              <div class="kata-text-content"></div>
            </div>
          );
        } else if (section.type === "exercise") {
          return (
            <div ref={(elem) => (itemContent.current[idx] = elem)}>
              <div class="section-title"></div>
              <div class="exercise-description"></div>
              <div>
                <Editor
                  defaultShots={1}
                  showExpr={false}
                  showShots={false}
                  shotError={shotError}
                  evtTarget={itemEvtHandlers[idx]}
                  compiler={props.compiler}
                  compilerState={props.compilerState}
                  onRestartCompiler={props.onRestartCompiler}
                  code={section.placeholderCode}
                  kataExercise={section}
                  key={section.id}
                  setHir={() => ({})}
                  activeTab="results-tab"
                  languageService={props.languageService}
                ></Editor>
                <OutputTabs
                  key={section.id + "-results"}
                  evtTarget={itemEvtHandlers[idx]}
                  showPanel={false}
                  kataMode={true}
                  onShotError={(diag?: VSDiagnostic) => setShotError(diag)}
                  hir=""
                  activeTab="results-tab"
                  setActiveTab={() => undefined}
                ></OutputTabs>
              </div>
              <div class="exercise-solution"></div>
            </div>
          );
        }
      })}
    </div>
  );
}
