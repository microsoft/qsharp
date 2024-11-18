// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

import Std.Arrays.Filtered;
import Std.Diagnostics.Fact;

struct TestCaseResult {
    did_pass : Bool,
    message : String,
}


function OutputMessage(test_results : TestCaseResult[]) : Unit {
    let num_tests = Length(test_results);
    let failed_tests = Filtered((item -> not item.did_pass), test_results);
    let num_passed = Std.Arrays.Count((item -> item.did_pass), test_results);
    let num_failed = Length(failed_tests);

    Fact((num_passed + num_failed) == num_tests, "invariant failed in test harness: passed plus failed should equal total");

    let test_word = if num_tests == 1 or num_tests == 0 { "test" } else { "tests" };
    Message($"{num_passed} of {num_tests} {test_word} passed. ({num_failed} failed)");
    for failed_test in failed_tests {
        Message($"{failed_test.message}");
    }
}
