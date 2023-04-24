// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//@ts-check

import assert from "node:assert";
import { test } from "node:test";
import { log } from "../dist/log.js";
import { getCompiler, getCompilerWorker } from "../dist/node.js"
import { getResultsHandler } from "../dist/common.js";
import { getKata } from "../dist/katas.js";

log.setLogLevel('warn');

/**
 * 
 * @param {string} code 
 * @param {string} expr
 * @param {boolean} useWorker
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

test('EntryPoint only', async t => {
    const code = `
namespace Test {
    @EntryPoint()
    operation MyEntry() : Result {
        use q1 = Qubit();
        return M(q1);
    }
}`;
    const result = await runSingleShot(code, "", false);
    assert(result.success === true);
    assert(result.result === "Zero");
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

test('kata success', async t => {
    const resultsHandler = getResultsHandler();
    const compiler = getCompilerWorker(resultsHandler);
    const code = `
namespace Kata {
  operation ApplyY(q : Qubit) : Unit is Adj + Ctl {
    Y(q);
  }
}`;
    const theKata = await getKata("single_qubit_gates");
    const firstExercise = theKata.items[0];
    const verifyCode = firstExercise.verificationImplementation;

    await compiler.runKata(code, verifyCode);
    const results = resultsHandler.getResults();
    compiler.terminate();

    assert(results.length === 1);
    assert(results[0].events.length === 2);
    assert(results[0].success);
});

test('kata incorrect', async t => {
    const resultsHandler = getResultsHandler();
    const compiler = getCompilerWorker(resultsHandler);
    const code = `
namespace Kata {
  operation ApplyY(q : Qubit) : Unit is Adj + Ctl {
    Z(q);
  }
}`;
    const theKata = await getKata("single_qubit_gates");
    const firstExercise = theKata.items[0];
    const verifyCode = firstExercise.verificationImplementation;

    await compiler.runKata(code, verifyCode);
    const results = resultsHandler.getResults();
    compiler.terminate();

    assert(results.length === 1);
    assert(results[0].events.length === 4);
    assert(!results[0].success); 
});

test('kata syntax error', async t => {
    const resultsHandler = getResultsHandler();
    const compiler = getCompilerWorker(resultsHandler);
    const code = `
namespace Kata {
  operaion ApplyY(q : Qubit) : Unt is Adj + Ctl {
    Z(q);
  }
}`;
    const theKata = await getKata("single_qubit_gates");
    const firstExercise = theKata.items[0];
    const verifyCode = firstExercise.verificationImplementation;

    await compiler.runKata(code, verifyCode);
    const results = resultsHandler.getResults();
    compiler.terminate();

    assert(results.length === 1);
    assert(results[0].events.length === 0);
    assert(!results[0].success); 
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
