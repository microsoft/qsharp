// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

import Std.Diagnostics.Fact;
import Std.Arrays.All;

operation Main() : Unit {
    FunctionTestMatrixTests();
    OperationTestMatrixTests();
    BasicTests();
}

operation OperationTestMatrixTests() : Unit {
    let test_cases : (Qubit[] => Unit, Int)[] = [
        (qs => { X(qs[0]); X(qs[3]); }, 0b1001),
        (qs => { X(qs[0]); X(qs[1]); }, 0b0011)
    ];

    let res : Util.TestCaseResult[] = Operations.TestMatrix(
        "QubitTestMatrix",
        qs => MeasureInteger(qs),
        4,
        test_cases,
        Operations.RunAllTestCases
    );
}

function FunctionTestMatrixTests() : Unit {
    let all_passed = Functions.TestMatrix("Return 42", TestCaseOne, [((), 42), ((), 42)], Functions.CheckAllTestCases);
    Fact(all_passed, "basic test matrix did not pass");

    let at_least_one_failed = not Functions.TestMatrix("Return 42", TestCaseOne, [((), 42), ((), 43)], Functions.CheckAllTestCases);
    Fact(at_least_one_failed, "basic test matrix did not report failure");

    let results = Functions.TestMatrix("AddOne", AddOne, [(5, 6), (6, 7)], Functions.RunAllTestCases);
    Fact(Length(results) == 2, "test matrix did not return results for all test cases");
    Fact(All(result -> result.did_pass, results), "test matrix did not pass all test cases");
}

function BasicTests() : Unit {
    let sample_tests = [
        ("Should return 42", TestCaseOne, 43),
        ("Should add one", () -> AddOne(5), 6),
        ("Should add one", () -> AddOne(5), 6)
    ];

    Fact(
        not Functions.CheckAllTestCases(sample_tests),
        "Test harness failed to return false for a failing tests."
    );

    Fact(
        Functions.CheckAllTestCases([("always returns true", () -> true, true)]),
        "Test harness failed to return true for a passing test"
    );

    let run_all_result = Functions.RunAllTestCases(sample_tests);

    Fact(
        Length(run_all_result) == 3,
        "Test harness did not return results for all test cases."
    );

    Fact(not run_all_result[0].did_pass, "test one passed when it should have failed");
    Fact(run_all_result[1].did_pass, "test two failed when it should have passed");
    Fact(run_all_result[2].did_pass, "test three failed when it should have passed");

}

function TestCaseOne() : Int {
    42
}

function AddOne(x : Int) : Int {
    x + 1
}