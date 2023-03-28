// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//@ts-check

import assert from "node:assert";
import {test} from "node:test";
import {checkCode, getCompletions, evaluate} from "../dist/node.js"

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
            return 42;
        }
    }`;
    let expr = `Test.Answer()`;
    let dumpText = ``;
    let callback = (ev) => dumpText += ev;
    let result = evaluate(code, expr, callback);
    let dump = JSON.parse(dumpText);
    assert(dump.type == "DumpMachine");
    assert(dump.state["|00>"].length == 2);
});
