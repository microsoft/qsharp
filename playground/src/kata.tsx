// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { useEffect } from "preact/hooks";
import { ICompilerWorker, Kata, QscEventTarget } from "qsharp";
import { Editor } from "./editor.js";
import { Results } from "./results.js";

export function Kata(props: {kata: Kata, compiler: ICompilerWorker}) {
    useEffect(() => {
        MathJax.typeset();
    }, [props.kata]);

    return (
    <div>
        <div dangerouslySetInnerHTML={{__html: props.kata.contentAsHtml}}></div>
        <br></br>
        {
            props.kata.items.map(item => {
              const evtTarget = new QscEventTarget(true);
              return (
              <div>
                <h2>{item.title}</h2>
                <div dangerouslySetInnerHTML={{__html: item.contentAsHtml}}></div>
                <Editor 
                    defaultShots={1}
                    showExpr={false}
                    showShots={false}
                    evtTarget={evtTarget}
                    compiler={props.compiler} 
                    code={item.placeholderImplementation}
                    kataVerify={item.verificationImplementation}
                    key={item.id}></Editor>
                <Results key={item.id + "-results"} evtTarget={evtTarget} showPanel={false}></Results>
              </div>);
            })
        }
    </div>
);
}
