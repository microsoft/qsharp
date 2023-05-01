// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { useEffect, useRef, useState } from "preact/hooks";
import { ICompilerWorker, QscEventTarget, VSDiagnostic, log } from "qsharp";

export function Editor(props: {
            code: string,
            compiler: ICompilerWorker,
            evtTarget: QscEventTarget,
            showExpr: boolean,
            defaultShots: number,
            showShots: boolean,
            kataVerify?: string,
        }) {
    const editorRef = useRef(null);
    const shotsRef = useRef(null);

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

    useEffect(() => {
        // Create the monaco editor
        log.info("Creating a monaco editor");
        const editorDiv: HTMLDivElement = editorRef.current as any;
        const editor = monaco.editor.create(editorDiv, {minimap: {enabled: false}, lineNumbersMinChars:3});
        const srcModel = monaco.editor.createModel(props.code, 'qsharp');
        editor.setModel(srcModel);
        setEditor(editor);
    
        // If the browser window resizes, tell the editor to update it's layout
        window.addEventListener('resize', _ => editor.layout());

        // As code is edited check it for errors and update the error list
        async function check() {
            // TODO: As this is async, code may be being edited while earlier check calls are still running.
            // Need to ensure that if this occurs, wait and try again on the next animation frame.
            // i.e. Don't queue a bunch of checks if some are still outstanding
            diagnosticsFrame = 0;
            let code = srcModel.getValue();
            let errs = await props.compiler.checkCode(code);

            // Note that as this is async, the code may have changed since checkCode was called.
            // TODO: Account for this scenario (e.g. delta positions with old source version)
            squiggleDiagnostics(errs);
            // TODO: Disable run button on errors: errs.length ?
            //    runButton.setAttribute("disabled", "true") : runButton.removeAttribute("disabled");

        }

        // Helpers to turn errors into editor squiggles
        let currentsquiggles: string[] = [];
        function squiggleDiagnostics(errors: VSDiagnostic[]) {
            let errList: {location: string, msg: string}[] = [];
            let newDecorations = errors?.map(err => {
                let startPos = srcModel.getPositionAt(err.start_pos);
                let endPos = srcModel.getPositionAt(err.end_pos);
                let range = monaco.Range.fromPositions(startPos, endPos);
                let decoration: monaco.editor.IModelDeltaDecoration = {
                    range,
                    options: { className: 'err-span', hoverMessage: { value: err.message } }
                }
                errList.push({
                    location: `main.qs@(${startPos.lineNumber},${startPos.column})`,
                    msg: err.message // TODO: Handle line breaks and 'help' notes
                });
                return decoration;
            });
            currentsquiggles = srcModel.deltaDecorations(currentsquiggles, newDecorations || []);
            setErrors(errList);
        }

        // While the code is changing, update the diagnostics as fast as the browser will render frames
        let diagnosticsFrame = requestAnimationFrame(check);

        srcModel.onDidChangeContent(ev => {
            if (!diagnosticsFrame) {
                diagnosticsFrame = requestAnimationFrame(check);
            }
        });

        return () => {
            log.info("Disposing a monaco editor");
            editor.dispose();
        }
    }, []);

    async function onRun() {
        const code = editor?.getModel()?.getValue();
        const shotsInput: HTMLInputElement = shotsRef.current as any;
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
        <span class="icon-button" title="Get link to this code">🔗</span>
        <span class="icon-button" title="Reset code to initial state" onClick={onReset}>🔄</span>
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
    <button id="run" class='main-button' onClick={onRun}>Run</button>
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
