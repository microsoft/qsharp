// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//@ts-check

import assert from "node:assert";
import { test } from "node:test";
import { checkCode, getCompletions, evaluate, run_shot } from "../dist/node.js"

test('no syntax errors', t => {
    let result = checkCode('namespace Foo { @EntryPoint() operation Main() : Unit {} }')
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

test('type error', t => {
    let code = `namespace Sample {
        operation main() : Result[] {
            use q1 = Qubit();
            Ry(q1);
            let m1 = M(q1);
            return [m1];
        }
    }`;
    let result = checkCode(code);
    assert.equal(result.length, 1);
    assert.equal(result[0].start_pos, 99);
    assert.equal(result[0].end_pos, 105);
    assert.equal(result[0].message, "expected (Double, Qubit), found Qubit:\n\nmismatched types");
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
