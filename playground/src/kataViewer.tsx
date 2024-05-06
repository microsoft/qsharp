// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/* eslint-disable @typescript-eslint/no-unused-vars */
import "preact/debug"; // TODO: Remove this line from production builds

import { render } from "preact";
import { useEffect } from "preact/hooks";

import {
  Exercise,
  ExplainedSolutionItem,
  Kata,
  Lesson,
  getAllKatas,
} from "qsharp-lang";

declare global {
  // The below are added by the MathJax and Highlight.js scripts
  interface Window {
    MathJax: any;
    hljs: any;
  }
}

window.MathJax = {
  loader: { load: ["[tex]/physics", "[tex]/color"] },
  tex: {
    packages: { "[+]": ["physics", "color"] },
    inlineMath: [
      ["$", "$"],
      ["\\(", "\\)"],
    ],
    formatError: (jax: any, err: any) => {
      console.log("LaTeX processing error occurred. ", err, jax);
      const errorNode = document.createElement("div");
      errorNode.innerText = `LaTeX processing error: ${err.message}.\nLaTeX: ${jax.latex}\n\n`;
      errorNode.style.fontSize = "20px";
      errorNode.style.color = "red";
      document.querySelector("#errors")?.appendChild(errorNode);
      window.scroll(0, 0);
      jax.formatError(err);
    },
  },
  startup: {
    pageReady: async () => {
      await onload();
      return window.MathJax.startup.defaultPageReady();
    },
  },
};

function Nav(props: {
  katas: Kata[];
  onnav: (index: number) => void;
  selected: number;
}) {
  return (
    <div class="nav">
      {props.katas.map((kata, idx) => (
        <>
          <div
            className={
              idx === props.selected ? "nav-item nav-selected" : "nav-item"
            }
            onClick={() => props.onnav(idx)}
          >
            {kata.title}
          </div>
        </>
      ))}
    </div>
  );
}

function KataEl(props: { kata: Kata }) {
  useEffect(() => {
    window.hljs.highlightAll();
    window.MathJax.typeset();
  });
  window.scrollTo(0, 0);
  return (
    <div class="content">
      <h1>{props.kata.title}</h1>
      {props.kata.sections.map((section) =>
        section.type === "lesson" ? (
          <LessonEl lesson={section} />
        ) : (
          <ExerciseEl exercise={section} />
        ),
      )}
    </div>
  );
}

function LessonEl(props: { lesson: Lesson }) {
  const item = props.lesson;
  return (
    <>
      <h2>{item.title}</h2>
      {item.items.map((item) => {
        switch (item.type) {
          case "text-content":
            return (
              <div dangerouslySetInnerHTML={{ __html: item.asHtml }}></div>
            );
          case "question":
            return (
              <>
                <h3>Question</h3>
                <div
                  dangerouslySetInnerHTML={{ __html: item.description.asHtml }}
                />
                <h3>Answer</h3>
                {item.answer.items.map((answer) => (
                  <ExplainedSolution item={answer} />
                ))}
              </>
            );
          case "example":
            return (
              <pre>
                <code>{item.code}</code>
              </pre>
            );
        }
      })}
    </>
  );
}

function ExerciseEl(props: { exercise: Exercise }) {
  const item = props.exercise;
  return (
    <>
      <h2>{"Exercise: " + item.title}</h2>
      <div dangerouslySetInnerHTML={{ __html: item.description.asHtml }} />
      <pre>
        <code>{item.placeholderCode}</code>
      </pre>
      <h4>Solution</h4>
      {item.explainedSolution.items.map((item) => (
        <ExplainedSolution item={item} />
      ))}
    </>
  );
}

function ExplainedSolution(props: { item: ExplainedSolutionItem }) {
  const item = props.item;
  return (
    <div>
      {item.type === "text-content" ? (
        <div dangerouslySetInnerHTML={{ __html: item.asHtml }}></div>
      ) : (
        <pre>
          <code>{item.code}</code>
        </pre>
      )}
    </div>
  );
}

async function onload() {
  const katas = await getAllKatas();
  const app = document.querySelector("#app") as HTMLDivElement;

  function onNav(index: number) {
    render(
      <>
        <Nav katas={katas} onnav={onNav} selected={index} />
        <KataEl kata={katas[index]} />
      </>,
      app,
    );
  }

  onNav(0);
}

// async function _old_onload() {
//   const katas = await getAllKatas();

//   const indexDiv = document.createElement("div");
//   document.body.appendChild(indexDiv);
//   const indexKatas = document.createElement("ul");
//   indexDiv.appendChild(indexKatas);

//   katas.forEach((kata) => {
//     const kataLi = document.createElement("li");
//     kataLi.innerHTML = `<a href="#kata-${kata.id}">${kata.title}</a>`;
//     indexKatas.appendChild(kataLi);

//     document.body.appendChild(getKataDiv(kata, indexKatas));
//   });
//   document.querySelectorAll("details").forEach((item) => (item.open = true));
//   document
//     .querySelectorAll("code")
//     .forEach((item) => (item.className = "qsharp"));
//   window.hljs.highlightAll();
// }

// function getKataDiv(kata: Kata, index: HTMLElement) {
//   const kataDiv = document.createElement("div");
//   const kataHeader = document.createElement("h1");
//   kataHeader.innerText = kata.title;
//   kataHeader.id = `kata-${kata.id}`;
//   kataDiv.appendChild(kataHeader);

//   const sectionIndex = document.createElement("ul");
//   index.appendChild(sectionIndex);

//   kata.sections.forEach((section) => {
//     const sectionDiv = document.createElement("div");
//     const sectionHeader = document.createElement("h2");
//     sectionHeader.innerText = section.title;
//     const sectionId = `section-${section.id}`;
//     sectionHeader.id = sectionId;
//     sectionDiv.appendChild(sectionHeader);

//     const sectionLi = document.createElement("li");
//     sectionLi.innerHTML = `<a href="#${sectionId}">${section.title}</a>`;
//     sectionIndex.appendChild(sectionLi);

//     if (section.type === "lesson") {
//       section.items.forEach((item) => {
//         switch (item.type) {
//           case "text-content": {
//             const content = document.createElement("div");
//             content.innerHTML = item.asHtml;
//             sectionDiv.appendChild(content);
//             break;
//           }
//           case "question": {
//             const questionHeader = document.createElement("h3");
//             questionHeader.innerHTML = `Question`;
//             sectionDiv.appendChild(questionHeader);
//             const questionBody = document.createElement("div");
//             questionBody.innerHTML = item.description.asHtml;
//             sectionDiv.appendChild(questionBody);
//             const answerHeader = document.createElement("h3");
//             answerHeader.innerHTML = `Answer`;
//             sectionDiv.appendChild(answerHeader);
//             item.answer.items.forEach((item) => addContent(item, sectionDiv));
//             break;
//           }
//           case "example": {
//             const code = document.createElement("pre");
//             code.innerHTML = `<code>${item.code}</code>`;
//             sectionDiv.appendChild(code);
//             break;
//           }
//         }
//       });
//     } else {
//       // Exercise
//       const exerciseHeader = document.createElement("h3");
//       exerciseHeader.innerHTML = `Exercise: <u>${section.title}</u>`;
//       sectionDiv.appendChild(exerciseHeader);
//       const exerciseDesc = document.createElement("div");
//       exerciseDesc.innerHTML = section.description.asHtml;
//       sectionDiv.appendChild(exerciseDesc);

//       const codeDiv = document.createElement("pre");
//       codeDiv.innerHTML = `<code>${section.placeholderCode}</code>`;
//       sectionDiv.appendChild(codeDiv);

//       const solutionHeader = document.createElement("h4");
//       solutionHeader.innerHTML = `Solution`;
//       sectionDiv.appendChild(solutionHeader);

//       section.explainedSolution.items.forEach((item) =>
//         addContent(item, sectionDiv),
//       );
//     }
//     kataDiv.appendChild(sectionDiv);
//   });
//   return kataDiv;
// }

// function addContent(item: ExplainedSolutionItem, sectionDiv: HTMLDivElement) {
//   if (item.type === "text-content") {
//     const contentDiv = document.createElement("div");
//     contentDiv.innerHTML = item.asHtml;
//     sectionDiv.appendChild(contentDiv);
//   } else {
//     // example or solution
//     const codeDiv = document.createElement("pre");
//     codeDiv.innerHTML = `<code>${item.code}</code>`;
//     sectionDiv.appendChild(codeDiv);
//   }
// }
