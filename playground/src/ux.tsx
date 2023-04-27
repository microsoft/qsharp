// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/// <reference path="../../node_modules/monaco-editor/monaco.d.ts"/>

import { render } from "preact";
import { getCompilerWorker, loadWasmModule } from "qsharp";

import { Nav } from "./nav.js";
import { Editor } from "./editor.js";
import { Results } from "./results.js";

const basePath = (window as any).qscBasePath || "";
const monacoPath = basePath + "libs/monaco/vs";
const modulePath = basePath + "libs/qsharp/qsc_wasm_bg.wasm";
const workerPath = basePath + "libs/worker.js";

const wasmPromise = loadWasmModule(modulePath); // Start loading but don't wait on it

const code = `namespace Sample {
    @EntryPoint()

    operation AllBasisVectorsWithPhases_TwoQubits() : Unit {
        use q1 = Qubit();
        use q4 = Qubit();

        H(q1);
        R1(0.3, q1);
        H(q4);

        use q5 = Qubit();
        use q6 = Qubit();
        S(q5);

        Rxx(1.0, q5, q6);

        Microsoft.Quantum.Diagnostics.DumpMachine();
    }
}
`;

function App(props: any) {
    return (<>
        <header class="header">Q# playground</header>
        <Nav></Nav>
        <Editor></Editor>
        <Results></Results>
    </>);
}


// Called once Monaco is ready
async function loaded() {
    await wasmPromise;

    render(<App></App>, document.body);

    let editorDiv = document.querySelector('#editor') as HTMLDivElement;
    let editor = monaco.editor.create(editorDiv, {minimap: {enabled: false}});
    let srcModel = monaco.editor.createModel(code, 'qsharp');
    editor.setModel(srcModel);

    // If the browser window resizes, tell the editor to update it's layout
    // TODO: Doesn't seem to resize nicely currently. Preact needs to rerender?
    window.addEventListener('resize', _ => editor.layout());
}

// Monaco provides the 'require' global for loading modules.
declare var require: any;
require.config({ paths: { vs: monacoPath } });
require(['vs/editor/editor.main'], loaded);
