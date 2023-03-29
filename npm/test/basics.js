// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//@ts-check

import assert from "node:assert";
import {test} from "node:test";
import {checkCode, getCompletions, evaluate, it_will_fail} from "../dist/node.js"

test('no syntax errors', t => {
    let result = checkCode('namespace Foo { operation Main() : Unit {} }')
    assert.equal(result.length, 0);
});

test('one syntax error', t => {
    let result = checkCode('namespace Foo []');
    assert.equal(result.length, 1);
});

test('completions include CNOT', t => {
    let results = getCompletions();
    let cnot = results.items.find(x => x.label === 'CNOT');
    assert.ok(cnot);
});

test('basic evaluation', t => {
    let code = `namespace Test {
        function Answer() : Int {
            return 42;
        }
    }`;
    let expr = `Test.Answer()`;
    let result = evaluate(code, expr);
    assert.equal(result, "42");
});

test('dump machine output', t => {
    let code = `namespace Test {
        function Answer() : Int {
            Microsoft.Quantum.Diagnostics.DumpMachine();
            return 42;
        }
    }`;
    let expr = `Test.Answer()`;
    let dumpText = ``;
    let callback = (ev) => dumpText += ev;
    let result = evaluate(code, expr, callback);
    let dump = JSON.parse(dumpText);
    assert(dump.type == "DumpMachine");
    assert(dump.state["|0⟩"].length == 2);
});

test('error types', t => {
    try {
        let ok_result = it_will_fail(5);
    }
    catch(e)
    {
        assert(false, "Should have worked!");
    }
    try {
        let err_result = it_will_fail(-1);
    } catch(e) {
        assert(e.start_pos != undefined);
        return;
    }
    assert.fail("Didn't throw an exception as expected");
});

test('runtime error position', t => {
    let code = `namespace Sample {
        operation main() : Result {
            use q1 = Qubit();
            Ry(q1);
            let m1 = M(q1);
            return [m1];
        }
    }`;
    let expr = 'Sample.main()';
    try {
        let result = evaluate(code, expr);
    } catch(e) {
        assert(e.start_pos);
        return;
    }
    assert.fail('Runtime error should have a position');
});