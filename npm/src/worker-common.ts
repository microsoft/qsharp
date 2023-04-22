// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { CompilerEvents, ICompiler, ICompilerWorker } from "./compiler.js";

export function createWorkerProxy(
            callbacks: CompilerEvents,
            postMessage: (msg: any) => void, 
            setMsgHandler: (handler: (e: any) => void) => void,
            terminator: () => void): ICompilerWorker
{
    // Used to resolve the in-flight promise (or null if nothing in-flight)
    let resolver: ((val: any) => void) | null = null;

    // Used to contruct the Promise that represents the WebWorker request
    function invoker(msg: any): Promise<any> {
        if (resolver) throw "Compiler operation in progress";
        return new Promise((resolve) => {
            resolver = resolve;
            postMessage(msg);
        });
    }

    // Construct the proxy for the WebWorker
    const proxy: ICompilerWorker = {
        checkCode(code) {
            return invoker({ "type": "checkCode", code });
        },
        getCompletions() {
            return invoker({ "type": "getCompletions" });
        },
        run(code, expr, shots) {
            return invoker({ "type": "run", code, expr, shots });
        },
        runKata(user_code, verify_code) {
            return invoker({ "type": "runKata", user_code, verify_code });
        },
        // Kill the worker without a chance to shutdown. May be needed if it is not responding.
        terminate: () => terminator()
    }
        
    setMsgHandler( (msg: any) => {
        switch (msg.type) {
            case "checkCode-result":
            case "getCompletions-result":
            case "run-result":
            case "runKata-result":
                resolver!(msg.result);
                resolver = null;
                break;
            case "message-event":
                callbacks.onMessage(msg.event)
                break;
            case "dumpMachine-event":
                callbacks.onDumpMachine(msg.event);
                break;
            case "success-event":
                callbacks.onSuccess(msg.event);
                break;
            case "failure-event":
                callbacks.onFailure(msg.event);
                break;
        };
    });

    return proxy;
}

export function getWorkerEventHandlers(postMessage: (msg: any) => void): CompilerEvents {
    return {
        onMessage(msg) {
            postMessage( {"type": "message-event", "event": msg} );
        },
        onDumpMachine(dump) {
            postMessage( {"type": "dumpMachine-event", "event": dump} );
        },
        onSuccess(result) {
            postMessage( {"type": "success-event", "event": result} );
        },
        onFailure(err) {
            postMessage( {"type": "failure-event", "event": err} );
        },
    }
}

export function handleMessageInWorker(data: any, compiler: ICompiler, postMessage: (msg: any) => void) {
    switch (data.type) {
        case "checkCode":
            compiler.checkCode(data.code)
                .then(result => postMessage({"type": "checkCode-result", result}));
            break;
        case "getCompletions":
            compiler.getCompletions()
                .then(result => postMessage({"type": "getCompletions-result", result}));
            break;
        case "run":
            compiler.run(data.code, data.expr, data.shots)
                .then(result => postMessage({"type": "run-result", result}));
            break;
        case "runKata":
            compiler.runKata(data.user_code, data.verify_code)
                .then(result => postMessage({"type": "runKata-result", result}));
        default:
            console.error(`Unrecognized msg type: ${data}`);
    }
}
