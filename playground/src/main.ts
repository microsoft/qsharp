// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/// <reference path="../../node_modules/monaco-editor/monaco.d.ts"/>

import {
    init, getCompletions, checkCode, evaluate, eventStringToMsg, mapDiagnostics,
    renderDump, IDiagnostic, ShotResult
} from "qsharp/browser";

import { generateHistogramData, generateHistogramSvg, sampleData } from "./histogram.js";
import { PopulateKatasList, RenderKatas } from "./katas.js";
import { base64ToCode, codeToBase64 } from "./utils.js";

const sampleCode = `namespace Sample {
    open Microsoft.Quantum.Diagnostics;

    @EntryPoint()
    operation Main() : Result[] {
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
declare var MathJax: { typeset: () => void; };

let runResults: ShotResult[] = [];
let currentFilter = "";
let editor: monaco.editor.IStandaloneCodeEditor;

// Helpers to turn errors into editor squiggles
let currentsquiggles: string[] = [];
function squiggleDiagnostics(errors: IDiagnostic[]) {
    let srcModel = editor.getModel()!;
    let newDecorations = errors.map(err => {
        let startPos = srcModel.getPositionAt(err.start_pos);
        let endPos = srcModel.getPositionAt(err.end_pos);
        let range = monaco.Range.fromPositions(startPos, endPos);
        let decoration: monaco.editor.IModelDeltaDecoration = {
            range,
            options: { className: 'err-span', hoverMessage: { value: err.message } }
        }
        return decoration;
    });
    currentsquiggles = srcModel.deltaDecorations(currentsquiggles, newDecorations);
}

// This runs after the Monaco editor is initialized
async function loaded() {
    await init(`libs/qsharp/qsc_wasm_bg.wasm`);

    // Assign the various UI controls into variables
    let editorDiv = document.querySelector('#editor') as HTMLDivElement;
    let errorsDiv = document.querySelector('#errors') as HTMLDivElement;
    let exprInput = document.querySelector('#expr') as HTMLInputElement;
    let shotCount = document.querySelector('#shot') as HTMLInputElement;
    let runButton = document.querySelector('#run') as HTMLButtonElement;
    let shareButton = document.querySelector('#share') as HTMLButtonElement;
    let shareConfirmation = document.querySelector('#share-confirmation') as HTMLDivElement;

    // Create the monaco editor
    editor = monaco.editor.create(editorDiv);

    // If URL is a sharing link, populate the editor with the code from the link. 
    // Otherwise, populate with sample code.
    const params = new URLSearchParams(window.location.search);

    let code = sampleCode;
    if (params.get("code")) {
        const base64code = decodeURIComponent(params.get("code")!);
        code = base64ToCode(base64code);
    }

    let srcModel = monaco.editor.createModel(code, 'qsharp');
    editor.setModel(srcModel);

    // As code is edited check it for errors and update the error list
    function check() {
        diagnosticsFrame = 0;
        let code = srcModel.getValue();
        let errs = checkCode(code);
        errorsDiv.innerText = JSON.stringify(errs, null, 2);

        squiggleDiagnostics(errs);
        errs.length ?
            runButton.setAttribute("disabled", "true") : runButton.removeAttribute("disabled");
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

        // State for tracking as shot results are reported
        let currentShotResult: ShotResult = {
            success: false,
            result: "pending",
            events: []
        };
        runResults = [];

        let event_cb = (ev: string) => {
            let result = eventStringToMsg(ev);
            if (!result) {
                console.error("Unrecognized message: " + ev);
                return;
            }
            switch (result.type) {
                case "Result":
                    currentShotResult.success = result.success;

                    // If there was an error, map the diagnostic location
                    let resultObj = result.result;
                    if(typeof resultObj == "object") {
                        resultObj = mapDiagnostics([resultObj], code)[0];
                    }

                    currentShotResult.result = resultObj;
                    // Push this result and prep for the next
                    runResults.push(currentShotResult);
                    currentShotResult = { success: false, result: "pending", events: [] };
                    break;
                case "Message":
                    currentShotResult.events.push(result);
                    break;
                case "DumpMachine":
                    currentShotResult.events.push(result);
                    break;
            }
        }

        try {
            performance.mark("start-shots");
            let result = evaluate(code, expr, event_cb, shots);
        } catch (e: any) {
            // TODO: Should only happen on crash. Telmetry?

        }
        performance.mark("end-shots");
        let measure = performance.measure("shots-duration", "start-shots", "end-shots");
        console.info(`Ran ${shots} shots in ${measure.duration}ms`);
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

    // Render katas.
    PopulateKatasList()
        .then(() => RenderKatas())
        .then(() => {
            let modulesSelect = document.querySelector('#katas-list') as HTMLSelectElement;
            modulesSelect.addEventListener('change', _ => {
                RenderKatas();
            });
        });

    shareButton.addEventListener('click', _ => {
        const code = srcModel.getValue();
        const encodedCode = codeToBase64(code);
        const escapedCode = encodeURIComponent(encodedCode);

        // Get current URL without query parameters to use as the base URL
        const newUrl = `${window.location.href.split('?')[0]}?code=${escapedCode}`;
        // Copy link to clipboard and update url without reloading the page
        navigator.clipboard.writeText(newUrl);
        window.history.pushState({}, '', newUrl);
        shareConfirmation.style.display = "inline";
    });
}

const reKetResult = /^\[(?:(Zero|One), *)*(Zero|One)\]$/
function resultToKet(result: string | IDiagnostic): string {
    if (typeof result !== 'string') return "ERROR";

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
        events: result.events,
        error: (typeof result.result === 'string') ? undefined : result.result
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
    let title = document.createElement("span");
    title.className = "result-header";
    let dumpTables = document.createElement("div");

    header.appendChild(prev);
    header.appendChild(next);
    header.appendChild(title);

    container.appendChild(header);
    container.appendChild(dumpTables);

    let currentIndex = 0;
    function showOutput(move: number) {
        currentIndex += move;
        if (currentIndex < 0) currentIndex = 0;
        if (currentIndex >= filteredResults.length) currentIndex = filteredResults.length - 1;

        let current = filteredResults[currentIndex];
        title.innerText = `Output for shot #${currentIndex + 1} of ${filteredResults.length}`;

        let resultHeader = `<p><b>Result:</b> ${current.result}`;
        if (current.error) {
            let pos = editor.getModel()?.getPositionAt(current.error.start_pos);
            resultHeader += ` - "${current.error.message}" at line ${pos?.lineNumber}, col ${pos?.column}`;
            squiggleDiagnostics([current.error]);
        } else {
            squiggleDiagnostics([]);
        }
        resultHeader += "</p>";
        dumpTables.innerHTML = resultHeader;

        filteredResults[currentIndex].events.forEach(event => {
            switch (event.type) {
                case "Message":
                    // A Message output
                    let div = document.createElement("div");
                    div.className = "message-output";
                    div.innerText = event.message
                    dumpTables.appendChild(div);
                    break;
                case "DumpMachine":
                    // A DumpMachine output
                    let table = document.createElement("table");
                    table.innerHTML = renderDump(event.state);
                    dumpTables.appendChild(table);
            }
        });
    }

    prev.addEventListener('click', _ => showOutput(-1));
    next.addEventListener('click', _ => showOutput(1));
    showOutput(0);
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
