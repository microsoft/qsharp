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

// This runs after the Monaco editor is initialized
async function loaded() {
    await init("/libs/qsharp/qsc_wasm_bg.wasm");

    // Assign the various UI controls into variables
    let editorDiv = document.querySelector('#editor') as HTMLDivElement;
    let errorsDiv = document.querySelector('#errors') as HTMLDivElement; 
    let exprInput = document.querySelector('#expr') as HTMLInputElement;
    let shotCount = document.querySelector('#shot') as HTMLInputElement;
    let runButton = document.querySelector('#run') as HTMLButtonElement;
    let outputDiv = document.querySelector('#output') as HTMLDivElement;

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
        let shotResults: ShotResult[] = [];

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
            shotResults.push(currentShotResult);
            currentShotResult = {result: "null", dumps: []};
        }
        runComplete(shotResults);
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

function runComplete(results: ShotResult[]) {
    if (!results.length) return;

    // Get an array of results, preferably in ket form
    let histogramData = results.map(result => resultToKet(result.result));
    let bucketData = generateHistogramData(histogramData);
    let histogram = generateHistogramSvg(bucketData);

    let resultsDiv = document.querySelector('#results')!;
    resultsDiv.innerHTML = "";
    resultsDiv.appendChild(histogram);
    results[0].dumps.forEach(dump => {
        let table = document.createElement("table");
        table.innerHTML = renderDump(dump);
        resultsDiv.appendChild(table);
    });
}

// Monaco provides the 'require' global for loading modules.
declare var require: any;
require.config({ paths: { vs: 'libs/monaco/vs' } });
require(['vs/editor/editor.main'], loaded);
