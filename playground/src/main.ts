/// <reference path="../../node_modules/monaco-editor/monaco.d.ts"/>

import {init, getCompletions} from "qsharp/browser";

// MathJax will already be loaded on the page. Need to call `typeset` when LaTeX content changes.
declare var MathJax: {typeset: () => void;};

// This runs after the Monaco editor is initialized
async function loaded() {
    await init("/libs/qsharp/qsc_wasm_bg.wasm");

    let editorDiv = document.querySelector('#editor') as HTMLDivElement;

    let editor = monaco.editor.create(editorDiv);
    let srcModel = monaco.editor.createModel(`// TODO\n`, 'qsharp');
    editor.setModel(srcModel);
    window.addEventListener('resize', _ => editor.layout());

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
