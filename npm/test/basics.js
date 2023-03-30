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

test('runtime error position', t => {
    // TODO: The below should be a compile-time check and the test will fail when fixed.
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

test('message output', t => {
    let code = `namespace Sample {
        operation main() : Unit {
            Message("hello qsharp");
            return ();
        }
    }`;
    let expr = 'Sample.main()';
    let output = null;
    let called = 0;
    let result = evaluate(code, expr, (msg) => {
        ++called;
        output = msg;
    });
    assert.equal(called, 1);
    let msg_obj = JSON.parse(output || "");
    assert(msg_obj.type === "Message");
    assert(msg_obj.message == "hello qsharp");
});
