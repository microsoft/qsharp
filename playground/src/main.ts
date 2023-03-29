// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/// <reference path="../../node_modules/monaco-editor/monaco.d.ts"/>

import {init, getCompletions, checkCode, evaluate, 
    outputAsDump, renderDump, IDiagnostic, Dump } from "qsharp/browser";

import {generateHistogramData, generateHistogramSvg, sampleData} from "./histogram.js";

const sampleCode = `namespace Sample {
    open Microsoft.Quantum.Diagnostics;

    operation main() : Result {
        use q1 = Qubit();
        use q2 = Qubit();

        H(q1);
        CNOT(q1, q2);
        DumpMachine();

        let m1 = M(q1);
        let m2 = M(q2);

        return [m1, m2];
    }
}
`;

// MathJax will already be loaded on the page. Need to call `typeset` when LaTeX content changes.
declare var MathJax: {typeset: () => void;};

type ShotResult = {
    result: string;
    dumps: Dump[];
}
let runResults: ShotResult[] = [];
let currentFilter = "";

// This runs after the Monaco editor is initialized
async function loaded() {
    await init("/libs/qsharp/qsc_wasm_bg.wasm");

    // Assign the various UI controls into variables
    let editorDiv = document.querySelector('#editor') as HTMLDivElement;
    let errorsDiv = document.querySelector('#errors') as HTMLDivElement; 
    let exprInput = document.querySelector('#expr') as HTMLInputElement;
    let shotCount = document.querySelector('#shot') as HTMLInputElement;
    let runButton = document.querySelector('#run') as HTMLButtonElement;

    // Create the monaco editor and set some initial code
    let editor = monaco.editor.create(editorDiv);
    let srcModel = monaco.editor.createModel(sampleCode, 'qsharp');
    editor.setModel(srcModel);

    // Helpers to turn errors into editor squiggles
    let currentsquiggles: string[] = [];
    function squiggleDiagnostics(errors: IDiagnostic[]) {
        let newDecorations = errors.map(err => {
            let startPos = srcModel.getPositionAt(err.start_pos);
            let endPos = srcModel.getPositionAt(err.end_pos);
            let range = monaco.Range.fromPositions(startPos, endPos);
            let decoration: monaco.editor.IModelDeltaDecoration = {
                range,
                options: {className: 'err-span', hoverMessage: {value: err.message}}
            }
            return decoration;
        });
        currentsquiggles = srcModel.deltaDecorations(currentsquiggles, newDecorations);
    }

    // As code is edited check it for errors and update the error list
    function check() {
        diagnosticsFrame = 0;
        let code = srcModel.getValue();
        let errs = checkCode(code);
        errorsDiv.innerText = JSON.stringify(errs, null, 2);

        squiggleDiagnostics(errs);
    }

    // While the code is changing, update the diagnostics as fast as the browser will render frames
    let diagnosticsFrame = requestAnimationFrame(check);

    srcModel.onDidChangeContent(ev => {
        if (!diagnosticsFrame) {
            diagnosticsFrame = requestAnimationFrame(check);
        }
    });

    // If the browser window resizes, tell the editor to update it's layout
    window.addEventListener('resize', _ => editor.layout());

    // Try to evaluate the code when the run button is clicked
    runButton.addEventListener('click', _ => {
        let code = srcModel.getValue();
        let expr = exprInput.value;
        let shots = parseInt(shotCount.value);

        let currentShotResult: ShotResult = {
            "result": "null",
            "dumps": []
        };
        runResults = [];

        let event_cb = (ev: string) => {
            let dump = outputAsDump(ev);
            if (dump) {
                currentShotResult.dumps.push(dump);
            }
        }

        for(let i = 0; i < shots; ++i) {
            try {
                let result = evaluate(code, expr, event_cb);
                currentShotResult.result = result;
            } catch(e: any) {
                currentShotResult.result = "ERROR";
            }
            runResults.push(currentShotResult);
            currentShotResult = {result: "null", dumps: []};
        }
        runComplete();
    });

    // Example of getting results from a call into the WASM module
    monaco.languages.registerCompletionItemProvider("qsharp", { 
        provideCompletionItems(model, position, context, token) {
            // @ts-ignore : This is required in the defintion, but not needed.
            var range: monaco.IRange = undefined;

            let result = getCompletions();
            
            let mapped: monaco.languages.CompletionList = {
                suggestions: result.items.map(item => ({
                    label: item.label,
                    kind: item.kind, // TODO: Monaco seems to use different values than VS Code.
                    insertText: item.label,
                    range
                }))
            };
            return mapped;
        }
    });
}

const reKetResult = /^\[(?:(Zero|One), *)*(Zero|One)\]$/
function resultToKet(result: string): string {
    if (reKetResult.test(result)) {
        // The result is a simple array of Zero and One
        // The below will return an array of "Zero" or "One" in the order found
        let matches = result.match(/(One|Zero)/g);
        matches?.reverse();
        let ket = "|";
        matches?.forEach(digit => ket += (digit == "One" ? "1" : "0"));
        ket += "⟩";
        return ket;
    } else {
        return result;
    }
}

function renderOutputs(container: HTMLDivElement) {
    container.innerHTML = "";
    let mappedResults = runResults.map(result => ({
        result: resultToKet(result.result),
        dumps: result.dumps
    }));

    let filteredResults = currentFilter == "" ? mappedResults :
            mappedResults.filter(entry => entry.result === currentFilter);
    
    if (filteredResults.length === 0) return;

    // Show the current result and navigation.
    let header = document.createElement("div");
    let prev = document.createElement("button");
    prev.textContent = "Prev";
    let next = document.createElement("button");
    next.textContent = "Next";
    let title = document.createElement("h3");
    let dumpTables = document.createElement("div");

    header.appendChild(prev);
    header.appendChild(next);
    header.appendChild(title);

    container.appendChild(header);
    container.appendChild(dumpTables);

    let currentIndex = 0;
    function showDump(move: number) {
        currentIndex += move;
        if (currentIndex < 0) currentIndex = 0;
        if (currentIndex >= filteredResults.length) currentIndex = filteredResults.length - 1;

        let current = filteredResults[currentIndex];
        title.innerText = `Result: ${filteredResults[currentIndex].result} - #${currentIndex + 1} of ${filteredResults.length}`;
        dumpTables.innerHTML = "";

        filteredResults[currentIndex].dumps.forEach(dump => {
            let table = document.createElement("table");
            table.innerHTML = renderDump(dump);
            dumpTables.appendChild(table);
        });
    }

    prev.addEventListener('click', _ => showDump(-1));
    next.addEventListener('click', _ => showDump(1));
    showDump(0);
}

function runComplete() {
    let outputDiv = document.querySelector('#output') as HTMLDivElement;
    outputDiv.innerHTML = "";
    currentFilter = "";
    if (!runResults.length) return;

    // Get an array of results, preferably in ket form
    let histogramData = runResults.map(result => resultToKet(result.result));
    let bucketData = generateHistogramData(histogramData);
    let histogram = generateHistogramSvg(bucketData, (label) => {
        currentFilter = label;
        renderOutputs(outputContainer);
    });
    outputDiv.appendChild(histogram);

    let outputContainer = document.createElement("div");
    outputDiv.appendChild(outputContainer);
    renderOutputs(outputContainer);
}

// Monaco provides the 'require' global for loading modules.
declare var require: any;
require.config({ paths: { vs: 'libs/monaco/vs' } });
require(['vs/editor/editor.main'], loaded);
