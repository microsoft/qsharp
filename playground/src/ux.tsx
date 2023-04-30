// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/// <reference path="../../node_modules/monaco-editor/monaco.d.ts"/>

import { render } from "preact";
import { ICompilerWorker, QscEventTarget, getCompilerWorker, loadWasmModule } from "qsharp";

import { Nav } from "./nav.js";
import { Editor } from "./editor.js";
import { Results } from "./results.js";
import { useState } from "preact/hooks";
import { samples } from "./samples.js";

const basePath = (window as any).qscBasePath || "";
const monacoPath = basePath + "libs/monaco/vs";
const modulePath = basePath + "libs/qsharp/qsc_wasm_bg.wasm";
const workerPath = basePath + "libs/worker.js";

const wasmPromise = loadWasmModule(modulePath); // Start loading but don't wait on it

const initialCode = `namespace Sample {
    open Microsoft.Quantum.Diagnostics;

    @EntryPoint()
    operation Main() : Result[] {
        use q1 = Qubit();
        use q2 = Qubit();
        use q3 = Qubit();

        H(q1);
        CNOT(q1, q2);
        Y(q2);
        H(q3);
        DumpMachine();

        let m1 = M(q1);
        let m2 = M(q2);
        let m3 = M(q3);

        return [m1, m2, m3];
    }
}`;

function App(props: {compiler: ICompilerWorker, evtTarget: QscEventTarget}) {
    const [mainCode, setMainCode] = useState(initialCode);

    function onSampleSelected(name: string) {
        const sampleDict = samples as {[index: string]: string};
        const sample: string = sampleDict[name];
        if (sample) setMainCode(sample);
    }

    return (<>
        <header class="header">Q# playground</header>
        <Nav sampleSelected={onSampleSelected}></Nav>
        <Editor code={mainCode} compiler={props.compiler} evtTarget={props.evtTarget}></Editor>
        <Results evtTarget={props.evtTarget}></Results>
    </>);
}

// Called once Monaco is ready
async function loaded() {
    await wasmPromise; // Block until the wasm module is loaded
    const evtHander = new QscEventTarget(true);
    const compiler = await getCompilerWorker(workerPath);

    render(<App compiler={compiler} evtTarget={evtHander}></App>, document.body);
}

// Monaco provides the 'require' global for loading modules.
declare var require: any;
require.config({ paths: { vs: monacoPath } });
require(['vs/editor/editor.main'], loaded);
