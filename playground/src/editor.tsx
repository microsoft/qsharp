// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/// <reference types="../../node_modules/monaco-editor/monaco.d.ts"/>

import { useEffect, useRef, useState } from "preact/hooks";
import { ICompilerWorker, QscEventTarget, VSDiagnostic, log } from "qsharp";
import { codeToBase64 } from "./utils.js";

export function Editor(props: {
            code: string,
            compiler: ICompilerWorker,
            evtTarget: QscEventTarget,
            showExpr: boolean,
            defaultShots: number,
            showShots: boolean,
            kataVerify?: string,
            shotError?: VSDiagnostic,
        }) {
    const editorRef = useRef<HTMLDivElement>(null);
    const shotsRef = useRef<HTMLInputElement>(null);

    const [editor, setEditor] = useState<monaco.editor.IStandaloneCodeEditor | null>(null);
    const [errors, setErrors] = useState<{location: string, msg: string}[]>([]);
    const [initialCode, setInitialCode] = useState(props.code);

    // Check if the initial code changed (i.e. sample selected) since first created
    // If so, need to load it into the editor and save as the new initial code.
    if (initialCode !== props.code) {
        editor?.getModel()?.setValue(props.code || "");
        editor?.revealLineNearTop(1);
        setInitialCode(props.code);
    }

    // On reset, reload the initial code
    function onReset() {
        editor?.getModel()?.setValue(initialCode || "");
    }

    function onGetLink() {
        const code = editor?.getModel()?.getValue();
        if (!code) return;

        const encodedCode = codeToBase64(code);
        const escapedCode = encodeURIComponent(encodedCode);

        // Get current URL without query parameters to use as the base URL
        const newUrl = `${window.location.href.split('?')[0]}?code=${escapedCode}`;
        // Copy link to clipboard and update url without reloading the page
        navigator.clipboard.writeText(newUrl);
        window.history.pushState({}, '', newUrl);
        // TODO: Alert user somehow link is on the clipboard
    }

    useEffect(() => {
        // Create the monaco editor
        log.info("Creating a monaco editor");
        const editorDiv = editorRef.current;
        if (!editorDiv) return;
        const editor = monaco.editor.create(editorDiv, {minimap: {enabled: false}, lineNumbersMinChars:3});
        const srcModel = monaco.editor.createModel(props.code, 'qsharp');
        editor.setModel(srcModel);
        setEditor(editor);
    
        // If the browser window resizes, tell the editor to update it's layout
        window.addEventListener('resize', () => editor.layout());

        // As code is edited check it for errors and update the error list
        async function check() {
            // TODO: As this is async, code may be being edited while earlier check calls are still running.
            // Need to ensure that if this occurs, wait and try again on the next animation frame.
            // i.e. Don't queue a bunch of checks if some are still outstanding
            diagnosticsFrame = 0;
            const code = srcModel.getValue();
            const errs = await props.compiler.checkCode(code);

            // Note that as this is async, the code may have changed since checkCode was called.
            // TODO: Account for this scenario (e.g. delta positions with old source version)
            squiggleDiagnostics(errs);
            // TODO: Disable run button on errors: errs.length ?
            //    runButton.setAttribute("disabled", "true") : runButton.removeAttribute("disabled");

        }

        // Helpers to turn errors into editor squiggles
        function squiggleDiagnostics(errors: VSDiagnostic[]) {
            const errList: {location: string, msg: string}[] = [];
            const newMarkers = errors?.map(err => {
                const startPos = srcModel.getPositionAt(err.start_pos);
                const endPos = srcModel.getPositionAt(err.end_pos);
                const marker: monaco.editor.IMarkerData = {
                  severity: monaco.MarkerSeverity.Error,
                  message: err.message,
                  startLineNumber: startPos.lineNumber,
                  startColumn: startPos.column,
                  endLineNumber: endPos.lineNumber,
                  endColumn: endPos.column,
                }
                errList.push({
                    location: `main.qs@(${startPos.lineNumber},${startPos.column})`,
                    msg: err.message // TODO: Handle line breaks and 'help' notes
                });
                return marker;
            });
            monaco.editor.setModelMarkers(srcModel, "qsharp", newMarkers);
            setErrors(errList);
        }

        // While the code is changing, update the diagnostics as fast as the browser will render frames
        let diagnosticsFrame = requestAnimationFrame(check);

        srcModel.onDidChangeContent( () => {
            if (!diagnosticsFrame) {
                diagnosticsFrame = requestAnimationFrame(check);
            }
        });

        return () => {
            log.info("Disposing a monaco editor");
            editor.dispose();
        }
    }, []);

    useEffect( () => {
      // This code highlights the error in the editor if you move to a shot result that has an error
      const srcModel = editor?.getModel();
      if (!srcModel) return;

      if (props.shotError) {
        const err = props.shotError;
        const startPos = srcModel.getPositionAt(err.start_pos);
        const endPos = srcModel.getPositionAt(err.end_pos);

        const marker: monaco.editor.IMarkerData = {
          severity: monaco.MarkerSeverity.Error,
          message: err.message,
          startLineNumber: startPos.lineNumber,
          startColumn: startPos.column,
          endLineNumber: endPos.lineNumber,
          endColumn: endPos.column,
        }
        monaco.editor.setModelMarkers(srcModel, "qsharp", [marker]);
        setErrors([{
            location: `main.qs@(${startPos.lineNumber},${startPos.column})`,
            msg: err.message // TODO: Handle line breaks and 'help' notes
        }]);
      } else {
        monaco.editor.setModelMarkers(srcModel, "qsharp", []);
        setErrors([]);
      }
    }, [props.shotError])

    async function onRun() {
        const code = editor?.getModel()?.getValue();
        const shotsInput = shotsRef.current;
        const shots = shotsInput ? parseInt(shotsInput.value) || 1 : props.defaultShots;
        if (!code) return;
        props.evtTarget.clearResults();
        if (props.kataVerify) {
            // This is for a kata. Provide the verification code.
            await props.compiler.runKata(code, props.kataVerify, props.evtTarget);
        } else {
            await props.compiler.run(code, "", shots, props.evtTarget);
        }
    }

    return (
<div class="editor-column">
  <div style="display: flex; justify-content: space-between; align-items: center;">
    <div class="file-name">main.qs</div>
    <div class="icon-row">
      <svg onClick={onGetLink} width="24px" height="24px" viewBox="0 0 24 24" fill="none">
        <title>Get a link to this code</title>
        <path d="M14 12C14 14.2091 12.2091 16 10 16H6C3.79086 16 2 14.2091 2 12C2 9.79086 3.79086 8 6 8H8M10 12C10 9.79086 11.7909 8 14 8H18C20.2091 8 22 9.79086 22 12C22 14.2091 20.2091 16 18 16H16" stroke="#000000" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
      </svg>
      <svg onClick={onReset} width="24px" height="24px" viewBox="0 0 24 24">
        <title>Reset code to initial state</title>
        <g id="Page-1" stroke="none" stroke-width="1" fill="none" fill-rule="evenodd">
          <g id="Reload">
            <rect id="Rectangle" fill-rule="nonzero" x="0" y="0" width="24" height="24"> </rect>
            <path d="M4,13 C4,17.4183 7.58172,21 12,21 C16.4183,21 20,17.4183 20,13 C20,8.58172 16.4183,5 12,5 C10.4407,5 8.98566,5.44609 7.75543,6.21762" id="Path" stroke="#0C0310" stroke-width="2" stroke-linecap="round"></path>
            <path d="M9.2384,1.89795 L7.49856,5.83917 C7.27552,6.34441 7.50429,6.9348 8.00954,7.15784 L11.9508,8.89768" id="Path" stroke="#0C0310" stroke-width="2" stroke-linecap="round"></path>
          </g>
        </g>
      </svg>
    </div>
  </div>
  <div id="editor" ref={editorRef}></div>
  <div id="button-row">
    { props.showExpr ? <>
        <span>Start</span>
        <input id="expr" value="" />
      </> : null
    }
    { props.showShots ? <>
        <span>Shots</span>
        <input id="shot" type="number" value={props.defaultShots || 100} max="1000" min="1" ref={shotsRef}/>
      </> : null}
    <button id="run" class='main-button' onClick={onRun} disabled={errors.length > 0}>Run</button>
  </div>
{ errors.length ? 
  (<div class="error-list">
    {errors.map(err => (
        <div class="error-row"><span>{err.location}</span>: {err.msg}</div>
    ))}
  </div>) : null
}
</div>);
}
