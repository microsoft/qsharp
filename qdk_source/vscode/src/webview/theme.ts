// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

const themeAttribute = "data-vscode-theme-kind";

function updateGitHubTheme() {
  let isDark = true;

  const themeType = document.body.getAttribute(themeAttribute);

  switch (themeType) {
    case "vscode-light":
    case "vscode-high-contrast-light":
      isDark = false;
      break;
    default:
      isDark = true;
  }

  // Update the stylesheet href
  document.head.querySelectorAll("link").forEach((el) => {
    const ref = el.getAttribute("href");
    if (ref && ref.includes("github-markdown")) {
      const newVal = ref.replace(
        /(dark\.css)|(light\.css)/,
        isDark ? "dark.css" : "light.css",
      );
      el.setAttribute("href", newVal);
    }
  });
}

export function setThemeStylesheet() {
  // We need to add the right Markdown style-sheet for the theme.

  // For VS Code, there will be an attribute on the body called
  // "data-vscode-theme-kind" that is "vscode-light" or "vscode-high-contrast-light"
  // for light themes, else assume dark (will be "vscode-dark" or "vscode-high-contrast").

  // Use a [MutationObserver](https://developer.mozilla.org/en-US/docs/Web/API/MutationObserver)
  // to detect changes to the theme attribute.
  const callback = (mutations: MutationRecord[]) => {
    for (const mutation of mutations) {
      if (mutation.attributeName === themeAttribute) {
        updateGitHubTheme();
      }
    }
  };
  const observer = new MutationObserver(callback);
  observer.observe(document.body, { attributeFilter: [themeAttribute] });

  // Run it once for initial value
  updateGitHubTheme();
}
