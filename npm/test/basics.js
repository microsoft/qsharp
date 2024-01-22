// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//@ts-check

import assert from "node:assert/strict";
import { test } from "node:test";
import { log } from "../dist/log.js";
import {
  getCompiler,
  getCompilerWorker,
  getLanguageService,
  getLanguageServiceWorker,
  getDebugServiceWorker,
} from "../dist/main.js";
import { QscEventTarget } from "../dist/compiler/events.js";
import { getAllKatas, getExerciseSources, getKata } from "../dist/katas.js";
import samples from "../dist/samples.generated.js";
import { CreateIntegerTicks, CreateTimeTicks } from "../dist/ux/ticks.js";

/** @type {import("../dist/log.js").TelemetryEvent[]} */
const telemetryEvents = [];
log.setLogLevel("warn");
log.setTelemetryCollector((event) => telemetryEvents.push(event));

/**
 *
 * @param {string} code
 * @param {string} expr
 * @param {boolean} useWorker
 * @returns {Promise<import("../dist/compiler/common.js").ShotResult>}
 */
export function runSingleShot(code, expr, useWorker) {
  return new Promise((resolve, reject) => {
    const resultsHandler = new QscEventTarget(true);
    const compiler = useWorker ? getCompilerWorker() : getCompiler();

    compiler
      .run([["test.qs", code]], expr, 1, resultsHandler)
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
  const compiler = getCompiler();

  const diags = await compiler.checkCode("namespace Foo []");
  assert.equal(diags.length, 1);
  assert.deepEqual(diags[0].range.start, { line: 0, character: 14 });
  assert.deepEqual(diags[0].range.end, { line: 0, character: 15 });
});

test("error with newlines", async () => {
  const compiler = getCompiler();

  const diags = await compiler.checkCode(
    "namespace input { operation Foo(a) : Unit {} }",
  );
  assert.equal(diags.length, 2);
  assert.deepEqual(diags[0].range.start, { line: 0, character: 32 });
  assert.deepEqual(diags[0].range.end, { line: 0, character: 33 });
  assert.deepEqual(diags[1].range.start, { line: 0, character: 32 });
  assert.deepEqual(diags[1].range.end, { line: 0, character: 33 });
  assert.equal(
    diags[1].message,
    "type error: insufficient type information to infer type\n\nhelp: provide a type annotation",
  );
  assert.equal(
    diags[0].message,
    "type error: missing type in item signature\n\nhelp: types cannot be inferred for global declarations",
  );
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

async function runExerciseSolutionCheck(exercise, solution) {
  const evtTarget = new QscEventTarget(true);
  const compiler = getCompiler();
  const sources = await getExerciseSources(exercise);
  const success = await compiler.checkExerciseSolution(
    solution,
    sources,
    evtTarget,
  );

  const unsuccessful_events = evtTarget
    .getResults()
    .filter((evt) => !evt.success);
  let errorMsg = "";
  for (const event of unsuccessful_events) {
    const error = event.result;
    if (typeof error === "string") {
      errorMsg += "Result = " + error + "\n";
    } else {
      errorMsg += "Message = " + error.message + "\n";
    }
  }

  return {
    success: success,
    errorCount: unsuccessful_events.length,
    errorMsg: errorMsg,
  };
}

async function getAllKataExamples(kata) {
  let examples = [];

  // Get all the examples conatined in solution explanations.
  const exerciseExamples = kata.sections
    .filter((section) => section.type === "exercise")
    .map((exercise) =>
      exercise.explainedSolution.items.filter(
        (item) => item.type === "example",
      ),
    )
    .flat();
  examples = examples.concat(exerciseExamples);

  // Get all the examples in lessons.
  const lessonExamples = kata.sections
    .filter((section) => section.type === "lesson")
    .map((lesson) => lesson.items.filter((item) => item.type === "example"))
    .flat();
  examples = examples.concat(lessonExamples);

  return examples;
}

async function validateExercise(
  exercise,
  validatePlaceholder,
  validateSolutions,
) {
  // Validate the correctness of the placeholder code.
  if (validatePlaceholder) {
    const placeholderResult = await runExerciseSolutionCheck(
      exercise,
      exercise.placeholderCode,
    );

    // Check that there are no compilation or runtime errors.
    assert(
      placeholderResult.errorCount === 0,
      `Exercise "${exercise.id}" has compilation or runtime errors when using the placeholder as solution. ` +
        `Compilation and runtime errors:\n${placeholderResult.errorMsg}`,
    );

    // Check that the placeholder is an incorrect solution.
    assert(
      !placeholderResult.success,
      `Placeholder for exercise "${exercise.id}" is a correct solution but it is expected to be an incorrect solution`,
    );
  }

  // Validate the correctness of the solutions.
  if (validateSolutions) {
    const solutions = exercise.explainedSolution.items.filter(
      (item) => item.type === "solution",
    );

    // Check that the exercise has at least one solution.
    assert(
      solutions.length > 0,
      `Exercise "${exercise.id}" does not have solutions`,
    );

    // Check that the solutions are correct.
    for (const solution of solutions) {
      const solutionResult = await runExerciseSolutionCheck(
        exercise,
        solution.code,
      );

      // Check that there are no compilation or runtime errors.
      assert(
        solutionResult.errorCount === 0,
        `Solution "${solution.id}" for exercise "${exercise.id}" has compilation or runtime errors` +
          `Compilation and runtime errors:\n${solutionResult.errorMsg}`,
      );

      // Check that the solution is correct.
      assert(
        solutionResult.success,
        `Solution "${solution.id}" for exercise "${exercise.id}" is incorrect`,
      );
    }
  }
}

async function validateKata(
  kata,
  validateExamples,
  validateExercisePlaceholder,
  validateExerciseSolutions,
) {
  // Validate the correctness of Q# code related to exercises.
  const exercises = kata.sections.filter(
    (section) => section.type === "exercise",
  );
  for (const exercise of exercises) {
    await validateExercise(
      exercise,
      validateExercisePlaceholder,
      validateExerciseSolutions,
    );
  }

  if (validateExamples) {
    const examples = await getAllKataExamples(kata);
    for (const example of examples) {
      try {
        const result = await runSingleShot(example.code, "", false);
        assert(
          result.success,
          `Example "${example.id}" in "${kata.id}" kata failed to run.`,
        );
      } catch (error) {
        assert(
          false,
          `Example "${example.id}" in "${kata.id}" kata failed to build:\n${error}`,
        );
      }
    }
  }
}

test("all katas work", async () => {
  const katas = await getAllKatas();
  // N.B. If you update the expected katas count, make sure to add a validation test for your newly added kata.
  const expectedKatasCount = 9;
  assert.equal(
    katas.length,
    expectedKatasCount,
    `Expected ${expectedKatasCount} katas, but found ${katas.length} katas`,
  );
});

test("getting_started kata is valid", async () => {
  const kata = await getKata("getting_started");
  await validateKata(kata, true, true, true);
});

test("qubit kata is valid", async () => {
  const kata = await getKata("qubit");
  await validateKata(kata, true, true, true);
});

test("single_qubit_gates kata is valid", async () => {
  const kata = await getKata("single_qubit_gates");
  await validateKata(kata, true, true, true);
});

test("multi_qubit_systems kata is valid", async () => {
  const kata = await getKata("multi_qubit_systems");
  await validateKata(kata, true, true, true);
});

test("multi_qubit_gates kata is valid", async () => {
  const kata = await getKata("multi_qubit_gates");
  await validateKata(kata, true, true, true);
});

test("single_qubit_measurements is valid", async () => {
  const kata = await getKata("single_qubit_measurements");
  await validateKata(kata, true, true, true);
});

test("multi_qubit_measurements is valid", async () => {
  const kata = await getKata("multi_qubit_measurements");
  await validateKata(kata, true, true, true);
});

test("random_numbers kata is valid", async () => {
  const kata = await getKata("random_numbers");
  await validateKata(kata, true, true, true);
});

test("oracles kata is valid", async () => {
  const kata = await getKata("oracles");
  await validateKata(kata, true, true, true);
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
  await compiler.run([["test.qs", code]], expr, 100, resultsHandler);
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
  const testCases = samples.filter((x) => !x.omitFromTests);

  for await (const sample of testCases) {
    await compiler.run([[sample.title, sample.code]], "", 1, resultsHandler);
  }

  compiler.terminate();
  assert.equal(resultsHandler.resultCount(), testCases.length);
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
  await compiler.run([["test.qs", code]], "", 10, resultsHandler);
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
    compiler.run([["test.qs", code]], "", 10, resultsHandler).catch((err) => {
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
      resolve(undefined);
    }, 4);
  });
});

test("check code", async () => {
  const compiler = getCompiler();

  const diags = await compiler.checkCode("namespace Foo []");
  assert.equal(diags.length, 1);
  assert.deepEqual(diags[0].range.start, { line: 0, character: 14 });
  assert.deepEqual(diags[0].range.end, { line: 0, character: 15 });
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
      "type error: expected (Double, Qubit), found Qubit",
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
}`,
  );

  // dispose() will complete when the language service has processed all the updates.
  await languageService.dispose();
  assert(gotDiagnostics);
});

test("diagnostics with related spans", async () => {
  const languageService = getLanguageService();
  let gotDiagnostics = false;
  languageService.addEventListener("diagnostics", (event) => {
    gotDiagnostics = true;
    assert.equal(event.type, "diagnostics");
    assert.deepEqual(
      {
        code: "Qsc.Resolve.Ambiguous",
        message:
          "name error: `DumpMachine` could refer to the item in `Microsoft.Quantum.Diagnostics` or `Other`",
        related: [
          {
            message: "ambiguous name",
            range: {
              start: {
                character: 8,
                line: 6,
              },
              end: {
                character: 19,
                line: 6,
              },
            },
          },
          {
            message: "found in this namespace",
            range: {
              start: {
                character: 11,
                line: 2,
              },
              end: {
                character: 40,
                line: 2,
              },
            },
          },
          {
            message: "and also in this namespace",
            range: {
              start: {
                character: 11,
                line: 3,
              },
              end: {
                character: 16,
                line: 3,
              },
            },
          },
        ],
      },
      {
        code: event.detail.diagnostics[0].code,
        message: event.detail.diagnostics[0].message,
        related: event.detail.diagnostics[0].related?.map((r) => ({
          range: r.location.span,
          message: r.message,
        })),
      },
    );
  });

  await languageService.updateDocument(
    "test.qs",
    1,
    `namespace Other { operation DumpMachine() : Unit { } }
    namespace Test {
      open Microsoft.Quantum.Diagnostics;
      open Other;
      @EntryPoint()
      operation Main() : Unit {
        DumpMachine();
      }
    }`,
  );

  // dispose() will complete when the language service has processed all the updates.
  await languageService.dispose();
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
      "type error: expected (Double, Qubit), found Qubit",
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
}`,
  );

  // dispose() will complete when the language service has processed all the updates.
  await languageService.dispose();
  languageService.terminate();
  assert(gotDiagnostics);
});

test("language service configuration update", async () => {
  const languageService = getLanguageServiceWorker();
  let actualMessages = [];
  languageService.addEventListener("diagnostics", (event) => {
    actualMessages.push({
      messages: event.detail.diagnostics.map((d) => d.message),
    });
  });
  await languageService.updateDocument(
    "test.qs",
    1,
    `namespace Sample {
    operation main() : Unit {
    }
}`,
  );

  // Above document should have generated a missing entrypoint error.

  // Now update the configuration.
  await languageService.updateConfiguration({ packageType: "lib" });

  await languageService.dispose();
  languageService.terminate();

  // Updating the config should cause another diagnostics event clearing the errors.

  // All together, two events received: one with the error, one to clear it.
  assert.deepStrictEqual(
    [
      {
        messages: [
          "entry point not found\n" +
            "\n" +
            "help: a single callable with the `@EntryPoint()` attribute must be present if no entry expression is provided",
        ],
      },
      {
        messages: [],
      },
    ],
    actualMessages,
  );
});

test("language service in notebook", async () => {
  const languageService = getLanguageServiceWorker();
  let actualMessages = [];
  languageService.addEventListener("diagnostics", (event) => {
    actualMessages.push({
      messages: event.detail.diagnostics.map((d) => d.message),
    });
  });

  await languageService.updateNotebookDocument("notebook.ipynb", 1, {}, [
    { uri: "cell1", version: 1, code: "operation Main() : Unit {}" },
    { uri: "cell2", version: 1, code: "Foo()" },
  ]);

  // Above document should have generated a resolve error.

  await languageService.updateNotebookDocument("notebook.ipynb", 2, {}, [
    { uri: "cell1", version: 2, code: "operation Main() : Unit {}" },
    { uri: "cell2", version: 2, code: "Main()" },
  ]);

  // dispose() will complete when the language service has processed all the updates.
  await languageService.dispose();
  languageService.terminate();

  // Updating the notebook should cause another diagnostics event clearing the errors.

  // All together, two events received: one with the error, one to clear it.
  assert.deepStrictEqual(
    [
      {
        messages: [
          "name error: `Foo` not found",
          "type error: insufficient type information to infer type\n" +
            "\n" +
            "help: provide a type annotation",
        ],
      },
      {
        messages: [],
      },
    ],
    actualMessages,
  );
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
    .run([["test.qs", "invalid code"]], "", 1, events)
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

test("debug service loading source without entry point attr fails - web worker", async () => {
  const debugService = getDebugServiceWorker();
  try {
    const result = await debugService.loadSource(
      [
        [
          "test.qs",
          `namespace Sample {
    operation main() : Result[] {
        use q1 = Qubit();
        Y(q1);
        let m1 = M(q1);
        return [m1];
    }
}`,
        ],
      ],
      "base",
      undefined,
    );
    assert.ok(typeof result === "string" && result.trim().length > 0);
  } finally {
    debugService.terminate();
  }
});

test("debug service loading source with syntax error fails - web worker", async () => {
  const debugService = getDebugServiceWorker();
  try {
    const result = await debugService.loadSource(
      [
        [
          "test.qs",
          `namespace Sample {
    operation main() : Result[]
    }
}`,
        ],
      ],
      "base",
      undefined,
    );
    assert.ok(typeof result === "string" && result.trim().length > 0);
  } finally {
    debugService.terminate();
  }
});

test("debug service loading source with bad entry expr fails - web worker", async () => {
  const debugService = getDebugServiceWorker();
  try {
    const result = await debugService.loadSource(
      [["test.qs", `namespace Sample { operation main() : Unit { } }`]],
      "base",
      "SomeBadExpr()",
    );
    assert.ok(typeof result === "string" && result.trim().length > 0);
  } finally {
    debugService.terminate();
  }
});

test("debug service loading source with good entry expr succeeds - web worker", async () => {
  const debugService = getDebugServiceWorker();
  try {
    const result = await debugService.loadSource(
      [["test.qs", `namespace Sample { operation Main() : Unit { } }`]],
      "unrestricted",
      "Sample.Main()",
    );
    assert.ok(typeof result === "string");
    assert.equal(result.trim(), "");
  } finally {
    debugService.terminate();
  }
});

test("debug service loading source with entry point attr succeeds - web worker", async () => {
  const debugService = getDebugServiceWorker();
  try {
    const result = await debugService.loadSource(
      [
        [
          "test.qs",
          `namespace Sample {
    @EntryPoint()
    operation main() : Result[] {
        use q1 = Qubit();
        Y(q1);
        let m1 = M(q1);
        return [m1];
    }
}`,
        ],
      ],
      "base",
      undefined,
    );
    assert.ok(typeof result === "string");
    assert.equal(result.trim(), "");
  } finally {
    debugService.terminate();
  }
});

test("debug service getting breakpoints after loaded source succeeds when file names match - web worker", async () => {
  const debugService = getDebugServiceWorker();
  try {
    const result = await debugService.loadSource(
      [
        [
          "test.qs",
          `namespace Sample {
    @EntryPoint()
    operation main() : Result[] {
        use q1 = Qubit();
        Y(q1);
        let m1 = M(q1);
        return [m1];
    }
}`,
        ],
      ],
      "base",
      undefined,
    );
    assert.ok(typeof result === "string" && result.trim().length == 0);
    const bps = await debugService.getBreakpoints("test.qs");
    assert.equal(bps.length, 4);
  } finally {
    debugService.terminate();
  }
});

test("debug service compiling multiple sources - web worker", async () => {
  const debugService = getDebugServiceWorker();
  try {
    const result = await debugService.loadSource(
      [
        [
          "Foo.qs",
          `namespace Foo {
    open Bar;
    @EntryPoint()
    operation Main() : Int {
        Message("Hello");
        Message("Hello");
        return HelloFromBar();
    }
}`,
        ],
        [
          "Bar.qs",
          `namespace Bar {
    operation HelloFromBar() : Int {
          return 5;
    }
}`,
        ],
      ],
      "unrestricted",
      undefined,
    );
    assert.equal(result.trim(), "");
    const fooBps = await debugService.getBreakpoints("Foo.qs");
    assert.equal(fooBps.length, 3);

    const barBps = await debugService.getBreakpoints("Bar.qs");
    assert.equal(barBps.length, 1);
  } finally {
    debugService.terminate();
  }
});

test("CreateIntegerTicks: invalid inputs", () => {
  runAndAssertIntegerTicks(2, 1, []);
  runAndAssertIntegerTicks(0, 2, []);
  runAndAssertIntegerTicks(-5, 100, []);
});

test("CreateIntegerTicks: below 100", () => {
  runAndAssertIntegerTicks(1, 1, [1]);
  runAndAssertIntegerTicks(3, 3, [3]);
  runAndAssertIntegerTicks(4, 6, [4, 5, 6]);
  runAndAssertIntegerTicks(1, 100, [1, 10, 100]);
  runAndAssertIntegerTicks(1, 10, [1, 10]);
  runAndAssertIntegerTicks(1, 9, [1]);
  runAndAssertIntegerTicks(2, 10, [10]);
  runAndAssertIntegerTicks(2, 9, [2, 3, 4, 5, 6, 7, 8, 9]);
});

test("CreateIntegerTicks: more than 100", () => {
  runAndAssertIntegerTicks(20, 59, [20, 30, 40, 50]);
  runAndAssertIntegerTicks(231, 365, [300]);
  runAndAssertIntegerTicks(331, 365, [340, 350, 360]);
  runAndAssertIntegerTicks(567, 569, [567, 568, 569]);
});

test("CreateIntegerTicks: expected qubit numbers", () => {
  runAndAssertIntegerTicks(400, 8000000, [1000, 10000, 100000, 1000000]);
  runAndAssertIntegerTicks(12345, 67890, [20000, 30000, 40000, 50000, 60000]);
  runAndAssertIntegerTicks(23456, 27890, [24000, 25000, 26000, 27000]);
});

test("CreateTimeTicks: invalid inputs", () => {
  runAndAssertTimeTicks(2, 1, []);
  runAndAssertTimeTicks(0, 2, []);
  runAndAssertTimeTicks(-5, 100, []);
});

const second = 1e9;
const minute = 60 * second;
const hour = 60 * minute;
const day = 24 * hour;
const week = 7 * day;
const month = 30 * day;
const year = 365 * day;
const decade = 10 * year;
const century = 10 * decade;

test("CreateTimeTicks: nanoseconds below 100", () => {
  runAndAssertTimeTicks(1, 1, ["1 nanosecond"]);
  runAndAssertTimeTicks(3, 3, ["3 nanoseconds"]);
  runAndAssertTimeTicks(4, 6, [
    "4 nanoseconds",
    "5 nanoseconds",
    "6 nanoseconds",
  ]);
  runAndAssertTimeTicks(1, 100, ["1 nanosecond"]);
  runAndAssertTimeTicks(1, 10, ["1 nanosecond"]);
  runAndAssertTimeTicks(1, 9, ["1 nanosecond"]);
  runAndAssertTimeTicks(2, 10, ["10 nanoseconds"]);
  runAndAssertTimeTicks(2, 9, [
    "2 nanoseconds",
    "3 nanoseconds",
    "4 nanoseconds",
    "5 nanoseconds",
    "6 nanoseconds",
    "7 nanoseconds",
    "8 nanoseconds",
    "9 nanoseconds",
  ]);
});

test("CreateTimeTicks: microseconds", () => {
  runAndAssertTimeTicks(800, 1000, ["1 microsecond"]);
  runAndAssertTimeTicks(800, 2000, ["1 microsecond"]);
  runAndAssertTimeTicks(800, 11000, ["1 microsecond"]);
  runAndAssertTimeTicks(800, 21000, ["1 microsecond"]);
  runAndAssertTimeTicks(800, 111000, ["1 microsecond"]);
  runAndAssertTimeTicks(1001, 21000, ["10 microseconds"]);
  runAndAssertTimeTicks(10001, 21000, ["20 microseconds"]);
  runAndAssertTimeTicks(10001, 30000, ["20 microseconds", "30 microseconds"]);
});

test("CreateTimeTicks: milliseconds", () => {
  runAndAssertTimeTicks(800, 999999, ["1 microsecond"]);
  runAndAssertTimeTicks(800, 1000000, ["1 microsecond", "1 millisecond"]);
  runAndAssertTimeTicks(800000, 2000000, ["1 millisecond"]);
  runAndAssertTimeTicks(800000, 11000000, ["1 millisecond"]);
  runAndAssertTimeTicks(800000, 21000000, ["1 millisecond"]);
  runAndAssertTimeTicks(800000, 111000000, ["1 millisecond"]);
  runAndAssertTimeTicks(1000001, 111000000, ["100 milliseconds"]);
});

test("CreateTimeTicks: seconds", () => {
  runAndAssertTimeTicks(800000, second - 1, ["1 millisecond"]);
  runAndAssertTimeTicks(800000, second, ["1 millisecond", "1 second"]);
  runAndAssertTimeTicks(800000000, 2 * second, ["1 second"]);
  runAndAssertTimeTicks(800000000, 11 * second, ["1 second"]);
  runAndAssertTimeTicks(800000000, 21 * second, ["1 second"]);
  runAndAssertTimeTicks(800000000, 111 * second, ["1 second", "1 minute"]);
  runAndAssertTimeTicks(second + 1, 111 * second, ["1 minute"]);
});

test("CreateTimeTicks: minutes", () => {
  runAndAssertTimeTicks(second - 1, minute, ["1 second", "1 minute"]);
  runAndAssertTimeTicks(minute - second, 2 * minute, ["1 minute"]);
  runAndAssertTimeTicks(minute, 11 * minute, ["1 minute"]);
  runAndAssertTimeTicks(minute + 1, 21 * minute, ["10 minutes"]);
  runAndAssertTimeTicks(second, 111 * minute, [
    "1 second",
    "1 minute",
    "1 hour",
  ]);
  runAndAssertTimeTicks(minute + 1, 111 * minute, ["1 hour"]);
});

test("CreateTimeTicks: hours", () => {
  runAndAssertTimeTicks(minute - 1, hour, ["1 minute", "1 hour"]);
  runAndAssertTimeTicks(hour - minute, 2 * hour, ["1 hour"]);
  runAndAssertTimeTicks(hour, 11 * hour, ["1 hour"]);
  runAndAssertTimeTicks(hour + 1, 21 * hour, ["10 hours"]);
  runAndAssertTimeTicks(minute, 111 * hour, ["1 minute", "1 hour", "1 day"]);
  runAndAssertTimeTicks(hour + 1, 111 * hour, ["1 day"]);
});

test("CreateTimeTicks: days", () => {
  runAndAssertTimeTicks(hour - 1, day, ["1 hour", "1 day"]);
  runAndAssertTimeTicks(day - hour, 2 * day, ["1 day"]);
  runAndAssertTimeTicks(day, 11 * day, ["1 day", "1 week"]);
  runAndAssertTimeTicks(day + 1, 21 * day, ["1 week"]);
  runAndAssertTimeTicks(hour, 111 * day, [
    "1 hour",
    "1 day",
    "1 week",
    "1 month",
  ]);
  runAndAssertTimeTicks(day + 1, 111 * day, ["1 week", "1 month"]);
});

test("CreateTimeTicks: weeks", () => {
  runAndAssertTimeTicks(day, week, ["1 day", "1 week"]);
  runAndAssertTimeTicks(day + 1, week, ["1 week"]);
  runAndAssertTimeTicks(day * 8, day * 27, ["2 weeks", "3 weeks"]);
  runAndAssertTimeTicks(week - day, 2 * week, ["1 week"]);
  runAndAssertTimeTicks(week, 11 * week, ["1 week", "1 month"]);
  runAndAssertTimeTicks(week + 1, 35 * week, ["1 month"]);
  runAndAssertTimeTicks(day, 111 * week, [
    "1 day",
    "1 week",
    "1 month",
    "1 year",
  ]);
  runAndAssertTimeTicks(week + 1, 111 * week, ["1 month", "1 year"]);
});

test("CreateTimeTicks: months", () => {
  runAndAssertTimeTicks(week - 1, month, ["1 week", "1 month"]);
  runAndAssertTimeTicks(month - 1, 2 * month, ["1 month"]);
  runAndAssertTimeTicks(month, 11 * month, ["1 month"]);
  runAndAssertTimeTicks(month, 12 * month, ["1 month"]);
  runAndAssertTimeTicks(month, 12 * month + 5 * day, ["1 month", "1 year"]);
  runAndAssertTimeTicks(month + 1, 12 * month, ["10 months"]);
  // due to precision issues month + 1 == month
  runAndAssertTimeTicks(month + hour, 10 * month - hour, [
    "2 months",
    "3 months",
    "4 months",
    "5 months",
    "6 months",
    "7 months",
    "8 months",
    "9 months",
  ]);
  runAndAssertTimeTicks(week, 111 * month, ["1 week", "1 month", "1 year"]);
  runAndAssertTimeTicks(month + 1, 111 * month, ["1 year"]);
});

test("CreateTimeTicks: years", () => {
  runAndAssertTimeTicks(month - 1, year, ["1 month", "1 year"]);
  runAndAssertTimeTicks(year - month, 2 * year, ["1 year"]);
  // due to precision issues year + 1 == year and decade - 1 == decade
  runAndAssertTimeTicks(year + day, decade - day, [
    "2 years",
    "3 years",
    "4 years",
    "5 years",
    "6 years",
    "7 years",
    "8 years",
    "9 years",
  ]);

  runAndAssertTimeTicks(month, 111 * year, [
    "1 month",
    "1 year",
    "1 decade",
    "1 century",
  ]);
});

test("CreateTimeTicks: decades", () => {
  // due to precision issues year + 1 == year
  runAndAssertTimeTicks(year + day, 21 * year, ["1 decade"]);
  runAndAssertTimeTicks(year, decade, ["1 year", "1 decade"]);
  runAndAssertTimeTicks(decade - year, 2 * decade, ["1 decade"]);
  runAndAssertTimeTicks(year, 111 * decade, [
    "1 year",
    "1 decade",
    "1 century",
  ]);
  // due to precision issues decade + 1 == decade
  runAndAssertTimeTicks(decade + month, 111 * decade, ["1 century"]);
});

test("CreateTimeTicks: centuries", () => {
  runAndAssertTimeTicks(decade - 1, century, ["1 decade", "1 century"]);
  runAndAssertTimeTicks(century - decade, 2 * century, ["1 century"]);
  runAndAssertTimeTicks(century, 11 * century, ["1 century"]);
  runAndAssertTimeTicks(century + 1, 21 * century, ["1 century"]);
  runAndAssertTimeTicks(decade, 111 * century, ["1 decade", "1 century"]);
  runAndAssertTimeTicks(century + 1, 111 * century, ["1 century"]);
});

test("CreateTimeTicks: above centuries", () => {
  runAndAssertTimeTicks(century + 30 * year, 3 * century, [
    "2 centuries",
    "3 centuries",
  ]);
  runAndAssertTimeTicks(century + 30 * year, century + 55 * year, [
    "13 decades",
    "14 decades",
    "15 decades",
  ]);
  runAndAssertTimeTicks(2 * century + 32 * year, 2 * century + 36 * year, [
    "232 years",
    "233 years",
    "234 years",
    "235 years",
    "236 years",
  ]);
});

function getValues(ticks) {
  return ticks.map((tick) => tick.value);
}

function getLabels(ticks) {
  return ticks.map((tick) => tick.label);
}

function runAndAssertIntegerTicks(min, max, expected) {
  const message = `min: ${min}, max: ${max}`;
  assert.deepStrictEqual(
    getValues(CreateIntegerTicks(min, max)),
    expected,
    message,
  );
}

function runAndAssertTimeTicks(min, max, expected) {
  const message = `min: ${min}, max: ${max}`;
  assert.deepStrictEqual(
    getLabels(CreateTimeTicks(min, max)),
    expected,
    message,
  );
}
