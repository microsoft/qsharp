// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// TODO: useEffect to create, populate, and wire-up the editor on creation.

export function Editor(props: any) {
    return (
<div class="editor-column">
  <div class="file-name">main.qs</div>
  <div id="editor"></div>
  <div id="button-row">
    <div> </div>
    <span>Start</span>
    <input id="expr" value="" />
    <span>Shots</span>
    <input id="shot" type="number" value="100" max="1000" min="1" />
    <button id="run">Run</button>
  </div>
  <div id="error-list"></div>
</div>);
}
