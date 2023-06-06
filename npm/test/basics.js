// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//@ts-check

import assert from "node:assert";
import { test } from "node:test";
import { log } from "../dist/log.js";
import { getCompiler, getCompilerWorker } from "../dist/main.js";
import { QscEventTarget } from "../dist/events.js";
import { getKata } from "../dist/katas.js";
import samples from "../dist/samples.generated.js";

log.setLogLevel("warn");

/**
 *
 * @param {string} code
 * @param {string} expr
 * @param {boolean} useWorker
 * @returns {Promise<import("../dist/common.js").ShotResult>}
 */
export function runSingleShot(code, expr, useWorker) {
  return new Promise((resolve, reject) => {
    const resultsHandler = new QscEventTarget(true);
    const compiler = useWorker
      ? getCompilerWorker(resultsHandler)
      : getCompiler(resultsHandler);

    compiler
      .run(code, expr, 1)
      .then(() => resolve(resultsHandler.getResults()[0]))
      .catch((err) => reject(err))
      /* @ts-expect-error: ICompiler does not include 'terminate' */
      .finally(() => (useWorker ? compiler.terminate() : null));
  });
}

test("basic eval", async () => {
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

test("EntryPoint only", async () => {
  const code = `
namespace Test {
    @EntryPoint()
    operation MyEntry() : Result {
        use q1 = Qubit();
        return M(q1);
    }
}`;
  const result = await runSingleShot(code, "", true);
  assert(result.success === true);
  assert(result.result === "Zero");
});

test("one syntax error", async () => {
  const evtTarget = new QscEventTarget(true);
  evtTarget.addEventListener("diagnostics", (evt) => {
    diagnostics = true;
    assert.equal(evt.detail.diagnostics.length, 1);
    assert.equal(evt.detail.diagnostics[0].start_pos, 14);
    assert.equal(evt.detail.diagnostics[0].end_pos, 15);
  });
  const compiler = getCompiler(evtTarget);
  let diagnostics = false;
  await compiler.updateDocument("<buffer>", 1, "namespace Foo []");
  assert(diagnostics);
});

test("completions include CNOT", async () => {
  const compiler = getCompiler(new QscEventTarget(false));
  await compiler.updateDocument("<source>", 1, "");
  let results = await compiler.getCompletions("<source>", "", 0);
  let cnot = results.items.find((x) => x.label === "CNOT");
  assert.ok(cnot, `items are ${results.items.map((i) => i.label).join(", ")}`);
});

test("dump and message output", async () => {
  let code = `namespace Test {
        function Answer() : Int {
            Microsoft.Quantum.Diagnostics.DumpMachine();
            Message("hello, qsharp");
            return 42;
        }
    }`;
  let expr = `Test.Answer()`;

  const result = await runSingleShot(code, expr, true);
  assert(result.success);
  assert(result.events.length == 2);
  assert(result.events[0].type == "DumpMachine");
  assert(result.events[0].state["|0⟩"].length == 2);
  assert(result.events[1].type == "Message");
  assert(result.events[1].message == "hello, qsharp");
});

test("type error", async () => {
  let code = `namespace Sample {
        operation main() : Result[] {
            use q1 = Qubit();
            Ry(q1);
            let m1 = M(q1);
            return [m1];
        }
    }`;
  const evtTarget = new QscEventTarget(true);
  evtTarget.addEventListener("diagnostics", (evt) => {
    diagnostics = true;
    const diags = evt.detail;
    assert.equal(diags.diagnostics.length, 1);
    assert.equal(diags.diagnostics[0].start_pos, 99);
    assert.equal(diags.diagnostics[0].end_pos, 105);
    assert.equal(
      diags.diagnostics[0].message,
      "type error: expected (Double, Qubit), found Qubit"
    );
  });
  const compiler = getCompiler(evtTarget);
  let diagnostics = false;
  await compiler.updateDocument("<buffer>", 1, code);
  assert(diagnostics);
});

test("kata success", async () => {
  const evtTarget = new QscEventTarget(true);
  const compiler = getCompiler(evtTarget);
  const code = `
namespace Kata {
  operation ApplyY(q : Qubit) : Unit is Adj + Ctl {
    Y(q);
  }
}`;
  const theKata = await getKata("single_qubit_gates");
  const firstExercise = theKata.items[0];

  assert(firstExercise.type === "exercise");
  const verifyCode = firstExercise.verificationImplementation;

  const passed = await compiler.runKata(code, verifyCode);
  const results = evtTarget.getResults();

  assert(results.length === 1);
  assert(results[0].events.length === 2);
  assert(passed);
});

test("kata incorrect", async () => {
  const evtTarget = new QscEventTarget(true);
  const compiler = getCompilerWorker(evtTarget);
  const code = `
namespace Kata {
  operation ApplyY(q : Qubit) : Unit is Adj + Ctl {
    Z(q);
  }
}`;
  const theKata = await getKata("single_qubit_gates");
  const firstExercise = theKata.items[0];
  assert(firstExercise.type === "exercise");
  const verifyCode = firstExercise.verificationImplementation;

  const passed = await compiler.runKata(code, verifyCode);
  const results = evtTarget.getResults();
  compiler.terminate();

  assert(results.length === 1);
  assert(results[0].events.length === 4);
  assert(!passed);
});

test("kata syntax error", async () => {
  const evtTarget = new QscEventTarget(true);
  const compiler = getCompiler(evtTarget);
  const code = `
namespace Kata {
  operaion ApplyY(q : Qubit) : Unt is Adj + Ctl {
    Z(q);
  }
}`;
  const theKata = await getKata("single_qubit_gates");
  const firstExercise = theKata.items[0];
  assert(firstExercise.type === "exercise");
  const verifyCode = firstExercise.verificationImplementation;

  await compiler.runKata(code, verifyCode);
  const results = evtTarget.getResults();

  assert.equal(results.length, 1);
  assert.equal(results[0].events.length, 0);
  assert(!results[0].success);
  assert(typeof results[0].result !== "string");
  assert.equal(results[0].result.message, "Error: syntax error");
});

test("worker check", async () => {
  let code = `namespace Sample {
        operation main() : Result[] {
            use q1 = Qubit();
            Ry(q1);
            let m1 = M(q1);
            return [m1];
        }
    }`;
  const evtTarget = new QscEventTarget(true);
  evtTarget.addEventListener("diagnostics", (evt) => {
    diagnostics = true;
    const diags = evt.detail;
    assert.equal(diags.diagnostics.length, 1);
    assert.equal(diags.diagnostics[0].start_pos, 99);
    assert.equal(diags.diagnostics[0].end_pos, 105);
    assert.equal(
      diags.diagnostics[0].message,
      "type error: expected (Double, Qubit), found Qubit"
    );
  });
  const compiler = getCompilerWorker(evtTarget);
  let diagnostics = false;
  await compiler.updateDocument("<buffer>", 1, code);
  compiler.terminate();
  assert(diagnostics);
});

test("worker 100 shots", async () => {
  let code = `namespace Test {
        function Answer() : Int {
            Microsoft.Quantum.Diagnostics.DumpMachine();
            Message("hello, qsharp");
            return 42;
        }
    }`;
  let expr = `Test.Answer()`;

  const resultsHandler = new QscEventTarget(true);
  const compiler = getCompilerWorker(resultsHandler);
  await compiler.run(code, expr, 100);
  compiler.terminate();

  const results = resultsHandler.getResults();

  assert.equal(results.length, 100);
  results.forEach((result) => {
    assert(result.success);
    assert.equal(result.result, "42");
    assert.equal(result.events.length, 2);
  });
});

test("Run samples", async () => {
  const resultsHandler = new QscEventTarget(true);
  const compiler = getCompilerWorker(resultsHandler);

  for await (const sample of samples) {
    await compiler.run(sample.code, "", 1);
  }

  compiler.terminate();
  assert.equal(resultsHandler.resultCount(), samples.length);
  resultsHandler.getResults().forEach((result, i) => {
    assert(
      result.success,
      `sample ${i}: ${samples[i].title} failed, result: ${JSON.stringify(
        result.result
      )}`
    );
  });
});

test("state change", async () => {
  const resultsHandler = new QscEventTarget(false);
  const compiler = getCompilerWorker(resultsHandler);
  const stateChanges = [];

  compiler.onstatechange = (state) => {
    stateChanges.push(state);
  };
  const code = `namespace Test {
    @EntryPoint()
    operation MyEntry() : Result {
        use q1 = Qubit();
        return M(q1);
    }
  }`;
  await compiler.run(code, "", 10);
  compiler.terminate();
  // There SHOULDN'T be a race condition here between the 'run' promise completing and the
  // statechange events firing, as the run promise should 'resolve' in the next microtask,
  // whereas the idle event should fire synchronously when the queue is empty.
  // For more details, see https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Using_promises#task_queues_vs._microtasks
  assert(stateChanges.length === 2);
  assert(stateChanges[0] === "busy");
  assert(stateChanges[1] === "idle");
});

test("cancel worker", () => {
  return new Promise((resolve) => {
    const code = `namespace MyQuantumApp {
      open Microsoft.Quantum.Diagnostics;

      @EntryPoint()
      operation Main() : Result[] {
          repeat {} until false;
          return [];
      }
    }`;

    const cancelledArray = [];
    const evtTarget = new QscEventTarget(false);
    const compiler = getCompilerWorker(evtTarget);

    // Queue some tasks that will never complete
    compiler.run(code, "", 10).catch((err) => {
      cancelledArray.push(err);
    });
    compiler.updateDocument("<buffer>", 1, code).catch((err) => {
      cancelledArray.push(err);
    });

    // Ensure those tasks are running/queued before terminating.
    setTimeout(async () => {
      // Terminate the compiler, which should reject the queued promises
      compiler.terminate();

      // Start a new compiler and ensure that works fine
      const compiler2 = getCompilerWorker(evtTarget);
      evtTarget.addEventListener("diagnostics", (evt) => {
        diagnostics = true;
        // New 'check' result is good
        assert(
          Array.isArray(evt.detail.diagnostics) &&
            evt.detail.diagnostics.length === 0
        );
      });
      let diagnostics = false;
      await compiler2.updateDocument("<buffer>", 1, code);
      compiler2.terminate();
      assert(diagnostics);

      // Old requests were cancelled
      assert(cancelledArray.length === 2);
      assert(cancelledArray[0] === "terminated");
      assert(cancelledArray[1] === "terminated");
      resolve(null);
    }, 4);
  });
});
