//@ts-check

import assert from "node:assert";
import {test} from "node:test";
import {checkCode, getCompletions} from "../dist/node.js"

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
