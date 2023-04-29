// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { useEffect, useRef, useState } from "preact/hooks";
import { ICompilerWorker, QscEventTarget } from "qsharp";

export function Editor(props: {code: string, compiler: ICompilerWorker, evtTarget: QscEventTarget}) {
    const editorRef = useRef(null);
    const [editor, setEditor] = useState<monaco.editor.IStandaloneCodeEditor | null>(null);

    useEffect(() => {
        // Create the monaco editor
        const editorDiv: HTMLDivElement = editorRef.current as any;
        const editor = monaco.editor.create(editorDiv, {minimap: {enabled: false}, lineNumbersMinChars:3});
        const srcModel = monaco.editor.createModel(props.code, 'qsharp');
        editor.setModel(srcModel);
        setEditor(editor);
    
        // If the browser window resizes, tell the editor to update it's layout
        window.addEventListener('resize', _ => editor.layout());

        return () => editor.dispose();
    }, []);

    async function onRun() {
        const code = editor?.getModel()?.getValue();
        if (!code) return;
        props.evtTarget.clearResults();
        await props.compiler.run(code, "", 10, props.evtTarget);
    }

    return (
<div class="editor-column">
  <div style="display: flex; justify-content: space-between; align-items: center;">
    <div class="file-name">main.qs</div>
    <button class='main-button' style="margin-bottom: 2px">Share</button>
  </div>
  <div id="editor" ref={editorRef}></div>
  <div id="button-row">
    <span>Start</span>
    <input id="expr" value="" />
    <span>Shots</span>
    <input id="shot" type="number" value="100" max="1000" min="1" />
    <button id="run" class='main-button' onClick={onRun}>Run</button>
  </div>
  <div class="error-list">
    <div class="error-row"><span>main.qs@(10,12)</span>: Syntax error. Expected identifier.</div>
    <div class="error-row"><span>main.qs@(15,14)</span>: Identifier 'foo' is unknown</div>
  </div>
</div>);
}
