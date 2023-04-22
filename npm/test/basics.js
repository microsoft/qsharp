// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//@ts-check

import assert from "node:assert";
import { test } from "node:test";
import { getCompiler, getCompilerWorker } from "../dist/node.js"
import { getResultsHandler } from "../dist/common.js";

/**
 * 
 * @param {string} code 
 * @param {string} expr
 * @returns {Promise<import("../dist/common.js").ShotResult>}
 */
export function runSingleShot(code, expr, useWorker) {
    return new Promise( (resolve, reject) => {
        const resultsHandler = getResultsHandler();
        const compiler = useWorker ? getCompilerWorker(resultsHandler) : getCompiler(resultsHandler);

        compiler.run(code, expr, 1)
          .then(_ => resolve(resultsHandler.getResults()[0]))
          .catch(err => reject(err))
          /** @ts-ignore : terminate is only on workers */
          .finally(_ => useWorker ? compiler.terminate() : null);
    });
}

test('basic eval', async t => {
    let code = `namespace Test {
        function Answer() : Int {
            return 42;
        }
    }`;
    let expr = `Test.Answer()`;

    const result = await runSingleShot(code, expr, false);
    assert(result.success);
    assert.equal(result.result, "42");
});

test('one syntax error', async t => {
    const resultsHandler = getResultsHandler();
    const compiler = getCompiler(resultsHandler);

    const diags = await compiler.checkCode("namespace Foo []");
    assert.equal(diags.length, 1);
    assert.equal(diags[0].start_pos, 14);
    assert.equal(diags[0].end_pos, 15);
});

test('completions include CNOT', async t => {
    const resultsHandler = getResultsHandler();
    const compiler = getCompiler(resultsHandler);

    let results = await compiler.getCompletions();
    let cnot = results.items.find(x => x.label === 'CNOT');
    assert.ok(cnot);
});

test('dump and message output', async t => {
    let code = `namespace Test {
        function Answer() : Int {
            Microsoft.Quantum.Diagnostics.DumpMachine();
            Message("hello, qsharp");
            return 42;
        }
    }`;
    let expr = `Test.Answer()`;

    const result = await runSingleShot(code, expr, false);
    assert(result.success);
    assert(result.events.length == 2);
    assert(result.events[0].type == "DumpMachine");
    assert(result.events[0].state["|0⟩"].length == 2);
    assert(result.events[1].type == "Message");
    assert(result.events[1].message == "hello, qsharp");
});

test('type error', async t => {
    let code = `namespace Sample {
        operation main() : Result[] {
            use q1 = Qubit();
            Ry(q1);
            let m1 = M(q1);
            return [m1];
        }
    }`;
    const resultsHandler = getResultsHandler();
    const compiler = getCompiler(resultsHandler);
    let result = await compiler.checkCode(code);

    assert.equal(result.length, 1);
    assert.equal(result[0].start_pos, 99);
    assert.equal(result[0].end_pos, 105);
    assert.equal(result[0].message, "mismatched types");
});

test('worker check', async t => {
    let code = `namespace Sample {
        operation main() : Result[] {
            use q1 = Qubit();
            Ry(q1);
            let m1 = M(q1);
            return [m1];
        }
    }`;
    const resultsHandler = getResultsHandler();
    const compiler = getCompilerWorker(resultsHandler);
    let result = await compiler.checkCode(code);
    compiler.terminate();

    assert.equal(result.length, 1);
    assert.equal(result[0].start_pos, 99);
    assert.equal(result[0].end_pos, 105);
    assert.equal(result[0].message, "mismatched types"); 
});

test('worker 100 shots', async t => {
    let code = `namespace Test {
        function Answer() : Int {
            Microsoft.Quantum.Diagnostics.DumpMachine();
            Message("hello, qsharp");
            return 42;
        }
    }`;
    let expr = `Test.Answer()`;

    const resultsHandler = getResultsHandler();
    const compiler = getCompilerWorker(resultsHandler);
    await compiler.run(code, expr, 100);
    compiler.terminate();

    const results = resultsHandler.getResults();

    assert.equal(results.length, 100);
    results.forEach(result => {
        assert(result.success);
        assert.equal(result.result, "42");
        assert.equal(result.events.length, 2);
    });
});
