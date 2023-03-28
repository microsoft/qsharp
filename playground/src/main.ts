// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/// <reference path="../../node_modules/monaco-editor/monaco.d.ts"/>

import {init, getCompletions, checkCode, evaluate, 
    outputAsDump, renderDump, IDiagnostic} from "qsharp/browser";

const sampleCode = `namespace Sample {
    operation main() : Result {
        use q1 = Qubit();
        use q2 = Qubit();

        H(q1);
        CNOT(q1, q2);
        let m1 = M(q1);
        let m2 = M(q2);
        return [m1, m2];
    }
}
`;

// MathJax will already be loaded on the page. Need to call `typeset` when LaTeX content changes.
declare var MathJax: {typeset: () => void;};

// This runs after the Monaco editor is initialized
async function loaded() {
    await init("/libs/qsharp/qsc_wasm_bg.wasm");

    // Assign the various UI controls into variables
    let editorDiv = document.querySelector('#editor') as HTMLDivElement;
    let errorsDiv = document.querySelector('#errors') as HTMLDivElement; 
    let exprInput = document.querySelector('#expr') as HTMLInputElement;
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

        let dumpTables = "";
        let event_cb = (ev: string) => {
            let dump = outputAsDump(ev);
            if (dump) {
                dumpTables += `<table>${renderDump(dump)}</table>`;
            }
        }

        try {
            let result = evaluate(code, expr, event_cb);
            outputDiv.innerHTML = `<h2>Results</h2><p>${result}</p>`;
            if (dumpTables != "") {
                outputDiv.innerHTML += `<h3>DumpMachine states</h3>${dumpTables}`;
            }
        } catch(e: any) {
            outputDiv.innerHTML = `<h2>Error</h2><p>${e.toString()}</p>`;
        }
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

// Monaco provides the 'require' global for loading modules.
declare var require: any;
require.config({ paths: { vs: 'libs/monaco/vs' } });
require(['vs/editor/editor.main'], loaded);
