// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

/// # Summary
/// Runs a series of named test functions with expected results.
///
/// # Description
/// Given an array of test names, test functions, and expected results (of the same type), returns `true`
/// if all tests passed, and `false` if any one test failed.
/// Prints the expected and received result for any failed tests.
///
/// # Input
/// ## test_cases
/// An array of three-arity tuples of the form `(test_name, callable_to_test, expected_result)`.
/// `callable_to_test` will be called and its result will be compared to `expected_result`.
///
/// # Example
/// ```qsharp
/// function Main() : Unit {
///     TestCases([
///         ("Should return 42", TestCaseOne, 42),
///         ("Should add one", () -> AddOne(5), 6)
///     ]);
/// }
///
/// function TestCaseOne() : Int {
///     42
/// }
///
/// function AddOne(x: Int) : Int {
///     x + 1
/// }
/// ```
function TestCases<'Result : Eq + Show > (test_cases : (String, () -> 'Result, 'Result)[]) : Bool {
    let failed_test_buf = TestCasesSilent(test_cases);

    if Length(failed_test_buf) == 0 {
        Message($"{Length(test_cases)} test(s) passed.");
        true
    } else {
        Message($"{Length(failed_test_buf)} tests failed.");
        for item in failed_test_buf {
            Message($"{item}")
        }
        false
    }
}

/// # Summary
/// Similar to `Qtest.Functions.TestCases`, but returns test failure info in an array of strings instead of directly messaging to
/// output. Useful if you want to handle your own messaging when writing tests, for CI or similar.

/// See `Qtest.Functions.TestCases` for more details.
///
/// # Description
/// Given an array of test names, test functions, and expected results (of the same type), returns an array
/// of strings representing all failed test cases (if any). Strings are of the form "test_name: expected {}, got {}"
///
/// # Input
/// ## test_cases
/// An array of three-arity tuples of the form `(test_name, callable_to_test, expected_result)`.
/// `callable_to_test` will be called and its result will be compared to `expected_result`.
///
/// # Example
/// ```qsharp
/// function Main() : Unit {
///     let failure_messages = TestCasesSilent([
///         ("Should return 42", TestCaseOne, 42),
///         ("Should add one", () -> AddOne(5), 6)
///     ]);
///     Std.Diagnostics.Fact(Length(failure_messages) == 0, "No tests should fail.")
/// }
///
/// function TestCaseOne() : Int {
///     42
/// }
///
/// function AddOne(x: Int) : Int {
///     x + 1
/// }
/// ```
function TestCasesSilent<'Result : Eq + Show > (test_cases : (String, () -> 'Result, 'Result)[]) : String[] {
    let num_tests = Length(test_cases);
    mutable failed_test_buf = [];

    for (name, case, result) in test_cases {
        let (did_pass, message) = TestCase(case, result)!;
        if not did_pass {
            set failed_test_buf = failed_test_buf + [$"{name}: {message}"];
        }
    }

    failed_test_buf
}

struct TestCaseResult {
    did_pass : Bool,
    message : String,
}

function TestCase<'Result : Eq + Show > (test_case : () -> 'Result, expected : 'Result) : TestCaseResult {
    let result = test_case();
    if result == expected {
        new TestCaseResult { did_pass = true, message = "" }
    } else {
        new TestCaseResult { did_pass = false, message = $"expected: {expected}, got: {result}" }
    }
}



export TestCases; 