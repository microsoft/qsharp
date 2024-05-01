// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { ExplainedSolutionItem, Kata, getAllKatas } from "qsharp-lang";

async function onload() {
  const katas = await getAllKatas();

  katas.forEach((kata) => document.body.appendChild(getKataDiv(kata)));
  document.querySelectorAll("details").forEach((item) => (item.open = true));
  (window as any).MathJax.typeset();
}

function getKataDiv(kata: Kata) {
  const kataDiv = document.createElement("div");
  const kataHeader = document.createElement("h1");
  kataHeader.innerText = kata.title;
  kataDiv.appendChild(kataHeader);

  kata.sections.forEach((section) => {
    const sectionDiv = document.createElement("div");
    const sectionHeader = document.createElement("h2");
    sectionHeader.innerText = section.title;
    sectionDiv.appendChild(sectionHeader);

    if (section.type === "lesson") {
      section.items.forEach((item) => {
        switch (item.type) {
          case "text-content": {
            const content = document.createElement("div");
            content.innerHTML = item.asHtml;
            sectionDiv.appendChild(content);
            break;
          }
          case "question": {
            const questionHeader = document.createElement("h3");
            questionHeader.innerHTML = `Question`;
            sectionDiv.appendChild(questionHeader);
            const questionBody = document.createElement("div");
            questionBody.innerHTML = item.description.asHtml;
            sectionDiv.appendChild(questionBody);
            const answerHeader = document.createElement("h3");
            answerHeader.innerHTML = `Answer`;
            sectionDiv.appendChild(answerHeader);
            item.answer.items.forEach((item) => addContent(item, sectionDiv));
            break;
          }
          case "example": {
            const code = document.createElement("pre");
            code.innerHTML = `<code>${item.code}</code>`;
            sectionDiv.appendChild(code);
            break;
          }
        }
      });
    } else {
      // Exercise
      const exerciseHeader = document.createElement("h3");
      exerciseHeader.innerHTML = `Exercise: <u>${section.title}</u>`;
      sectionDiv.appendChild(exerciseHeader);
      const exerciseDesc = document.createElement("div");
      exerciseDesc.innerHTML = section.description.asHtml;
      sectionDiv.appendChild(exerciseDesc);

      const codeDiv = document.createElement("pre");
      codeDiv.innerHTML = `<code>${section.placeholderCode}</code>`;
      sectionDiv.appendChild(codeDiv);

      const solutionHeader = document.createElement("h4");
      solutionHeader.innerHTML = `Solution`;
      sectionDiv.appendChild(solutionHeader);

      section.explainedSolution.items.forEach((item) =>
        addContent(item, sectionDiv),
      );
    }
    kataDiv.appendChild(sectionDiv);
  });
  return kataDiv;
}

function addContent(item: ExplainedSolutionItem, sectionDiv: HTMLDivElement) {
  if (item.type === "text-content") {
    const contentDiv = document.createElement("div");
    contentDiv.innerHTML = item.asHtml;
    sectionDiv.appendChild(contentDiv);
  } else {
    // example or solution
    const codeDiv = document.createElement("pre");
    codeDiv.innerHTML = `<code>${item.code}</code>`;
    sectionDiv.appendChild(codeDiv);
  }
}

document.addEventListener("DOMContentLoaded", onload);
