// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { codeToCompressedBase64 } from "./utils.js";

export function Viewer(props: { code: string }) {
  async function onGetLink(ev: MouseEvent) {
    const code = props.code;
    if (!code) return;

    let messageText = "Unable to create the link";
    try {
      const encodedCode = await codeToCompressedBase64(code);
      const escapedCode = encodeURIComponent(encodedCode);
      // Get current URL without query parameters to use as the base URL
      const newUrl = `${
        window.location.href.split("?")[0]
      }?code=${escapedCode}&profile=unrestricted`;

      // Copy link to clipboard and update url without reloading the page
      navigator.clipboard.writeText(newUrl);

      window.history.pushState({}, "", newUrl);
      messageText = "Link was copied to the clipboard";
    } finally {
      const popup = document.getElementById("popup") as HTMLDivElement;
      popup.style.display = "block";
      popup.innerText = messageText;
      popup.style.left = `${ev.clientX - 120}px`;
      popup.style.top = `${ev.clientY - 40}px`;

      setTimeout(() => {
        popup.style.display = "none";
      }, 2000);
    }
  }

  return (
    <div class="viewer-column">
      <div style="display: flex; justify-content: space-between; align-items: center">
        <div></div>
        <div class="icon-row">
          <svg
            onClick={onGetLink}
            width="24px"
            height="24px"
            viewBox="0 0 24 24"
            fill="none"
          >
            <title>Get a link to this code</title>
            <path
              d="M14 12C14 14.2091 12.2091 16 10 16H6C3.79086 16 2 14.2091 2 12C2 9.79086 3.79086 8 6 8H8M10 12C10 9.79086 11.7909 8 14 8H18C20.2091 8 22 9.79086 22 12C22 14.2091 20.2091 16 18 16H16"
              stroke="#000000"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
          </svg>
        </div>
      </div>
      <pre class="example-viewer">
        <code>{props.code}</code>
      </pre>
    </div>
  );
}
