// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

import Std.Diagnostics.Fact;

function Main() : Unit {
    let sample_tests = [
        ("Should return 42", TestCaseOne, 43),
        ("Should add one", () -> AddOne(5), 42),
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

    Fact(run_all_result[0].did_pass, "test one passed when it should have failed");
    Fact(run_all_result[1].did_pass, "test two failed when it should have passed");
    Fact(run_all_result[2].did_pass, "test three passed when it should have failed");
}

function TestCaseOne() : Int {
    42
}

function AddOne(x : Int) : Int {
    x + 1
}