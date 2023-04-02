// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//@ts-check

import assert from "node:assert";
import {test} from "node:test";
import {checkCode, getCompletions, evaluate, run_shot} from "../dist/node.js"

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
    let result = run_shot(code, expr);
    assert(result.success);
    assert.equal(result.result, "42");
});

test('dump machine output', t => {
    let code = `namespace Test {
        function Answer() : Int {
            Microsoft.Quantum.Diagnostics.DumpMachine();
            return 42;
        }
    }`;
    let expr = `Test.Answer()`;
    let result = run_shot(code, expr);
    assert(result.success);
    assert(result.events.length == 1);
    assert(result.events[0].type == "DumpMachine");
    assert(result.events[0].state["|0⟩"].length == 2);
});

test('runtime error position', t => {
    // TODO: The below should be a compile-time check and the test will fail when fixed.
    let code = `namespace Sample {
        operation main() : Result[] {
            use q1 = Qubit();
            Ry(q1);
            let m1 = M(q1);
            return [m1];
        }
    }`;
    let expr = 'Sample.main()';
    let shot_result = run_shot(code, expr);
    // TODO: Error positions should be returned
    assert(!shot_result.success);
    if (typeof shot_result.result != "object") {
        assert.fail("Wrong result type");
    } else {
        assert(shot_result.result.start_pos == 101);
        assert(shot_result.result.end_pos == 105);
        assert(shot_result.result.message == "mismatched types");
    }
});

test('message output', t => {
    let code = `namespace Sample {
        operation main() : Unit {
            Message("hello qsharp");
            return ();
        }
    }`;
    let expr = 'Sample.main()';
    let result = run_shot(code, expr);
    assert(result.success);
    assert(result.events.length == 1);
    assert(result.events[0].type == "Message");
    assert(result.events[0].message === "hello qsharp");
});
