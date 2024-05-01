// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { ExplainedSolutionItem, Kata, getAllKatas } from "qsharp-lang";

(window as any).MathJax = {
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
      return (window as any).MathJax.startup.defaultPageReady();
    },
  },
};

async function onload() {
  const katas = await getAllKatas();

  const indexDiv = document.createElement("div");
  document.body.appendChild(indexDiv);
  const indexKatas = document.createElement("ul");
  indexDiv.appendChild(indexKatas);

  katas.forEach((kata) => {
    const kataLi = document.createElement("li");
    kataLi.innerHTML = `<a href="#kata-${kata.id}">${kata.title}</a>`;
    indexKatas.appendChild(kataLi);

    document.body.appendChild(getKataDiv(kata, indexKatas));
  });
  document.querySelectorAll("details").forEach((item) => (item.open = true));
}

function getKataDiv(kata: Kata, index: HTMLElement) {
  const kataDiv = document.createElement("div");
  const kataHeader = document.createElement("h1");
  kataHeader.innerText = kata.title;
  kataHeader.id = `kata-${kata.id}`;
  kataDiv.appendChild(kataHeader);

  const sectionIndex = document.createElement("ul");
  index.appendChild(sectionIndex);

  kata.sections.forEach((section) => {
    const sectionDiv = document.createElement("div");
    const sectionHeader = document.createElement("h2");
    sectionHeader.innerText = section.title;
    const sectionId = `section-${section.id}`;
    sectionHeader.id = sectionId;
    sectionDiv.appendChild(sectionHeader);

    const sectionLi = document.createElement("li");
    sectionLi.innerHTML = `<a href="#${sectionId}">${section.title}</a>`;
    sectionIndex.appendChild(sectionLi);

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
