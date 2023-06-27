// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//@ts-check

import assert from "node:assert";
import { test } from "node:test";
import { log } from "../dist/log.js";
import {
  getCompiler,
  getCompilerWorker,
  getLanguageService,
  getLanguageServiceWorker,
} from "../dist/main.js";
import { QscEventTarget } from "../dist/compiler/events.js";
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
    const compiler = useWorker ? getCompilerWorker() : getCompiler();

    compiler
      .run(code, expr, 1, resultsHandler)
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

test("kata success", async () => {
  const evtTarget = new QscEventTarget(true);
  const compiler = getCompiler();
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

  const passed = await compiler.runKata(code, verifyCode, evtTarget);
  const results = evtTarget.getResults();

  assert(results.length === 1);
  assert(results[0].events.length === 4);
  assert(passed);
});

test("kata incorrect", async () => {
  const evtTarget = new QscEventTarget(true);
  const compiler = getCompilerWorker();
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

  const passed = await compiler.runKata(code, verifyCode, evtTarget);
  const results = evtTarget.getResults();
  compiler.terminate();

  assert(results.length === 1);
  assert(results[0].events.length === 6);
  assert(!passed);
});

test("kata syntax error", async () => {
  const evtTarget = new QscEventTarget(true);
  const compiler = getCompiler();
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

  await compiler.runKata(code, verifyCode, evtTarget);
  const results = evtTarget.getResults();

  assert.equal(results.length, 1);
  assert.equal(results[0].events.length, 0);
  assert(!results[0].success);
  assert(typeof results[0].result !== "string");
  assert.equal(results[0].result.message, "Error: syntax error");
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
  const compiler = getCompilerWorker();
  await compiler.run(code, expr, 100, resultsHandler);
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
  const compiler = getCompilerWorker();
  const resultsHandler = new QscEventTarget(true);

  for await (const sample of samples) {
    await compiler.run(sample.code, "", 1, resultsHandler);
  }

  compiler.terminate();
  assert.equal(resultsHandler.resultCount(), samples.length);
  resultsHandler.getResults().forEach((result) => {
    assert(result.success);
  });
});

test("state change", async () => {
  const compiler = getCompilerWorker();
  const resultsHandler = new QscEventTarget(false);
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
  await compiler.run(code, "", 10, resultsHandler);
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
    const compiler = getCompilerWorker();
    const resultsHandler = new QscEventTarget(false);

    // Queue some tasks that will never complete
    compiler.run(code, "", 10, resultsHandler).catch((err) => {
      cancelledArray.push(err);
    });
    compiler.getHir(code).catch((err) => {
      cancelledArray.push(err);
    });

    // Ensure those tasks are running/queued before terminating.
    setTimeout(async () => {
      // Terminate the compiler, which should reject the queued promises
      compiler.terminate();

      // Start a new compiler and ensure that works fine
      const compiler2 = getCompilerWorker();
      const result = await compiler2.getHir(code);
      compiler2.terminate();

      // getHir should have worked
      assert(typeof result === "string" && result.length > 0);

      // Old requests were cancelled
      assert(cancelledArray.length === 2);
      assert(cancelledArray[0] === "terminated");
      assert(cancelledArray[1] === "terminated");
      resolve(null);
    }, 4);
  });
});

test("language service diagnostics", async () => {
  const languageService = getLanguageService();
  let gotDiagnostics = false;
  languageService.addEventListener("diagnostics", (event) => {
    gotDiagnostics = true;
    assert.equal(event.type, "diagnostics");
    assert.equal(event.detail.diagnostics.length, 1);
    assert.equal(
      event.detail.diagnostics[0].message,
      "type error: expected (Double, Qubit), found Qubit"
    );
  });
  await languageService.updateDocument(
    "test.qs",
    1,
    `namespace Sample {
    operation main() : Result[] {
        use q1 = Qubit();
        Ry(q1);
        let m1 = M(q1);
        return [m1];
    }
}`
  );
  assert(gotDiagnostics);
});

test("language service diagnostics - web worker", async () => {
  const languageService = getLanguageServiceWorker();
  let gotDiagnostics = false;
  languageService.addEventListener("diagnostics", (event) => {
    gotDiagnostics = true;
    assert.equal(event.type, "diagnostics");
    assert.equal(event.detail.diagnostics.length, 1);
    assert.equal(
      event.detail.diagnostics[0].message,
      "type error: expected (Double, Qubit), found Qubit"
    );
  });
  await languageService.updateDocument(
    "test.qs",
    1,
    `namespace Sample {
    operation main() : Result[] {
        use q1 = Qubit();
        Ry(q1);
        let m1 = M(q1);
        return [m1];
    }
}`
  );
  languageService.terminate();
  assert(gotDiagnostics);
});
async function testCompilerError(useWorker) {
  const compiler = useWorker ? getCompilerWorker() : getCompiler();
  if (useWorker) {
    // @ts-expect-error onstatechange only exists on the worker
    compiler.onstatechange = (state) => {
      lastState = state;
    };
  }

  const events = new QscEventTarget(true);
  let promiseResult = undefined;
  let lastState = undefined;
  await compiler
    .run("invalid code", "", 1, events)
    .then(() => {
      promiseResult = "success";
    })
    .catch(() => {
      promiseResult = "failure";
    });

  assert.equal(promiseResult, "failure");
  const results = events.getResults();
  assert.equal(results.length, 1);
  assert.equal(results[0].success, false);
  if (useWorker) {
    // Only the worker has state change events
    assert.equal(lastState, "idle");
    // @ts-expect-error terminate() only exists on the worker
    compiler.terminate();
  }
}

test("compiler error on run", () => testCompilerError(false));
test("compiler error on run - worker", () => testCompilerError(true));
