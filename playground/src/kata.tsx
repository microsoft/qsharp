// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { useEffect, useRef } from "preact/hooks";
import {
  CompilerState,
  ExplainedSolution,
  ICompilerWorker,
  ILanguageServiceWorker,
  Kata,
  Lesson,
  QscEventTarget,
  Question,
} from "qsharp";
import { Editor } from "./editor.js";
import { OutputTabs } from "./tabs.js";

function ExplainedSolutionAsHtml(solution: ExplainedSolution): string {
  let html = "";
  for (const item of solution.items) {
    if (item.type === "example" || item.type === "solution") {
      html += `<pre><code>${item.code}</code></pre>`;
    } else if (item.type === "text-content") {
      html += "<div>";
      html += item.asHtml;
      html += "</div>";
    }
  }
  return html;
}

function LessonAsHtml(lesson: Lesson): string {
  let html = "";
  for (const item of lesson.items) {
    if (item.type === "example") {
      html += `<pre><code>${item.code}</code></pre>`;
    } else if (item.type === "text-content") {
      html += "<div>";
      html += item.asHtml;
      html += "</div>";
    }
  }
  return html;
}

function QuestionAsHtml(question: Question): string {
  let html = "";
  html += "<div>";
  html += question.description.asHtml;
  html += "</div>";
  html += "<div>";
  html += question.answer.asHtml;
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
      () => new QscEventTarget(true)
    );
  }
  const itemEvtHandlers = handlerMap.current || [];

  useEffect(() => {
    // MathJax rendering inside of React components seems to mess them up a bit,
    // so we'll take control of it here and ensure the contents are replaced.
    if (!kataContent.current) return;

    props.kata.sections.forEach((section, idx) => {
      const parentDiv = itemContent.current[idx];
      let titlePrefix = "\u{1F41B}";
      if (section.type === "exercise") {
        titlePrefix = "\u{2328} Exercise: ";
        const descriptionDiv = parentDiv?.querySelector(
          ".exercise-description"
        );
        if (!descriptionDiv) return;
        descriptionDiv.innerHTML = section.description.asHtml;
        const solutionDiv = parentDiv?.querySelector(".exercise-solution");
        if (!solutionDiv) return;
        solutionDiv.innerHTML = ExplainedSolutionAsHtml(
          section.explainedSolution
        );
      } else if (section.type === "lesson") {
        titlePrefix = "\u{1F4D6} Lesson: ";
        const contentDiv = parentDiv?.querySelector(".kata-item-content");
        if (!contentDiv) return;
        contentDiv.innerHTML = LessonAsHtml(section);
      } else if (section.type === "question") {
        titlePrefix = "\u{2753} Question: ";
        const contentDiv = parentDiv?.querySelector(".kata-item-content");
        if (!contentDiv) return;
        contentDiv.innerHTML = QuestionAsHtml(section);
      }

      const titleDiv = parentDiv?.querySelector(".section-title");
      if (!titleDiv) return;
      titleDiv.innerHTML = titlePrefix + " <u>" + section.title + "</u>";
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
        if (section.type === "lesson" || section.type === "question") {
          return (
            <div ref={(elem) => (itemContent.current[idx] = elem)}>
              <div class="section-title"></div>
              <div class="kata-item-content"></div>
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
