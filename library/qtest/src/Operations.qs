// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

/// # Summary
/// Runs a series of named test operations with expected results.
///
/// # Description
/// Given an array of test names, test operations, and expected results (of the same type), returns `true`
/// if all tests passed, and `false` if any one test failed.
/// Prints the expected and received result for any failed tests.
///
/// # Input
/// ## test_cases
/// An array of five-arity tuples of the form `(test_name, num_qubits, qubit_prep_callable, callable_to_test, expected_result)`.
/// `num_qubits` will be allocated in a qubit array and passed to `qubit_prep_callable` to prepare the state before executing 
/// `callable_to_test`. Afterwards, `callable_to_test` will be called and its result will be compared to `expected_result`.
///
/// # Example
/// ```qsharp
/// function Main() : Unit {
///     TestCases([
///         ("Should return 42", TestCaseOne, 42),
///         ("Should add one", () => AddOne(5), 6)
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
operation TestCases<'Result : Eq + Show > (test_cases : (String, Int, (Qubit[]) => (), (Qubit[]) => 'Result, 'Result)[]) : Bool {
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
/// Similar to `Qtest.Operations.TestCases`, but returns test failure info in an array of strings instead of directly messaging to
/// output. Useful if you want to handle your own messaging when writing tests, for CI or similar.

/// See `Qtest.Operations.TestCases` for more details.
///
/// # Description
/// Given an array of test names, test functions, and expected results (of the same type), returns an array
/// of strings representing all failed test cases (if any). Strings are of the form "test_name: expected {}, got {}"
///
/// # Input
/// ## test_cases
/// An array of five-arity tuples of the form `(test_name, num_qubits, qubit_prep_callable, callable_to_test, expected_result)`.
/// `num_qubits` will be allocated in a qubit array and passed to `qubit_prep_callable` to prepare the state before executing 
/// `callable_to_test`. Afterwards, `callable_to_test` will be called and its result will be compared to `expected_result`.
///
/// # Example
/// ```qsharp
/// function Main() : Unit {
///     let failure_messages = TestCasesSilent([
///         ("Should return 42", TestCaseOne, 42),
///         ("Should add one", () => AddOne(5), 6)
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
operation TestCasesSilent<'Result : Eq + Show > (test_cases : (String, Int, (Qubit[]) => (), (Qubit[]) => 'Result, 'Result)[]) : String[] {
    let num_tests = Length(test_cases);
    mutable failed_test_buf = [];

    for (name, num_qubits, prepare_state, case, result) in test_cases {
        use qubits = Qubit[num_qubits];
        prepare_state(qubits);
        let (did_pass, message) = TestCase(qubits, case, result)!;
        if not did_pass {
            set failed_test_buf = failed_test_buf + [$"{name}: {message}"];
        }
        ResetAll(qubits);
    }

    failed_test_buf
}
struct TestCaseResult {
    did_pass : Bool,
    message : String,
}

operation TestCase<'Result : Eq + Show > (qubits: Qubit[], test_case : (Qubit[]) => 'Result, expected : 'Result) : TestCaseResult {
    let result = test_case(qubits);
    if result == expected {
        new TestCaseResult { did_pass = true, message = "" }
    } else {
        new TestCaseResult { did_pass = false, message = $"expected: {expected}, got: {result}" }
    }
}



export TestCases; 