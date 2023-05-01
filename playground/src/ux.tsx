// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/// <reference path="../../node_modules/monaco-editor/monaco.d.ts"/>

import { render } from "preact";
import { ICompilerWorker, QscEventTarget, getCompilerWorker, loadWasmModule, getAllKatas, Kata } from "qsharp";

import { Nav } from "./nav.js";
import { Editor } from "./editor.js";
import { Results } from "./results.js";
import { useState } from "preact/hooks";
import { samples } from "./samples.js";
import { Kata as Katas } from "./kata.js";

const basePath = (window as any).qscBasePath || "";
const monacoPath = basePath + "libs/monaco/vs";
const modulePath = basePath + "libs/qsharp/qsc_wasm_bg.wasm";
const workerPath = basePath + "libs/worker.js";

declare global {
    var MathJax: { typeset: () => void; };
}

const wasmPromise = loadWasmModule(modulePath); // Start loading but don't wait on it

function App(props: {compiler: ICompilerWorker, evtTarget: QscEventTarget, katas: Kata[]}) {
    const [currentNavItem, setCurrentNavItem] = useState("Minimal");
    const kataTitles = props.katas.map(elem => elem.title);
    const sampleTitles = Object.keys(samples);

    const sampleCode: string = (samples as any)[currentNavItem];
    const activeKata = kataTitles.includes(currentNavItem) ?
            props.katas.find(kata => kata.title === currentNavItem)
            : undefined;

    function onNavItemSelected(name: string) {
        setCurrentNavItem(name);
    }

    return (<>
        <header class="header">Q# playground</header>
        <Nav selected={currentNavItem} navSelected={onNavItemSelected}
            katas={kataTitles} samples={sampleTitles}></Nav>
{
    sampleCode ? <>
        <Editor 
            code={sampleCode}
            compiler={props.compiler}
            evtTarget={props.evtTarget}
            defaultShots={100}
            showShots={true}
            showExpr={true}></Editor>
        <Results evtTarget={props.evtTarget} showPanel={true}></Results>
      </> :
        <Katas kata={activeKata!} compiler={props.compiler}></Katas>
}
    </>);
}

// Called once Monaco is ready
async function loaded() {
    await wasmPromise; // Block until the wasm module is loaded
    const katas = await getAllKatas();
    const evtHander = new QscEventTarget(true);
    const compiler = await getCompilerWorker(workerPath);

    render(<App compiler={compiler} evtTarget={evtHander} katas={katas}></App>, document.body);
}

// Monaco provides the 'require' global for loading modules.
declare var require: any;
require.config({ paths: { vs: monacoPath } });
require(['vs/editor/editor.main'], loaded);
