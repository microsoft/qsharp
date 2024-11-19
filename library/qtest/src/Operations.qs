// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

import Util.TestCaseResult, Util.OutputMessage;
import Std.Arrays.Mapped, Std.Arrays.All;

/// # Summary
/// Runs a number of test cases and returns true if all tests passed, false otherwise.
/// Prints a report of what passed and what failed as output.
///
/// For a more flexible test running function, see `RunAllTestCases` which returns
/// test results instead of printing out to output.
///
/// # Input
/// Takes a list of test cases. A test case is a tuple of `(String, () -> T, 'T)`, where
/// the first String is the name of the test, the function is the test case itself, and the
/// final element of the tuple is the expected return value from the test case.
///
/// # Example
/// ```qsharp
/// CheckAllTestCases([("Should return 42", () -> 42, 42)]);
/// ```
operation CheckAllTestCases<'T : Eq + Show>(test_cases : (String, Int, (Qubit[]) => (), (Qubit[]) => 'T, 'T)[]) : Bool {
    let test_results = RunAllTestCases(test_cases);

    OutputMessage(test_results);

    All(test_case -> test_case.did_pass, test_results)

}

/// # Summary
/// Runs all given test cases and returns a `TestCaseResult` for each test, representing whether or not it passed
/// and what the failure message, if any.
/// This is a good alternative to `CheckAllTestCases` when you want custom output based on the results of your tests,
/// or more control over how test results are rendered.
/// # Input
/// Takes a list of test cases. A test case is a tuple of `(String, () -> T, 'T)`, where
/// the first String is the name of the test, the function is the test case itself, and the
/// final element of the tuple is the expected return value from the test case.
///
/// # Example
/// ```qsharp
/// RunAllTestCases([("Should return 42", () -> 42, 42)]);
/// ```
operation RunAllTestCases<'T : Eq + Show>(test_cases : (String, Int, (Qubit[]) => (), (Qubit[]) => 'T, 'T)[]) : TestCaseResult[] {
    let num_tests = Length(test_cases);

    let num_tests = Length(test_cases);

    MappedOperation((name, num_qubits, prepare_state, case, result) => {
        use qubits = Qubit[num_qubits];
        prepare_state(qubits);
        let res = TestCase(name, qubits, case, result);
        ResetAll(qubits);
        res
    }, test_cases)
}

/// Helper function, copy of `Std.Arrays.Mapped` which works on operations instead
/// of functions.
operation MappedOperation<'T, 'U>(mapper : ('T => 'U), array : 'T[]) : 'U[] {
    mutable mapped = [];
    for element in array {
        set mapped += [mapper(element)];
    }
    mapped
}

/// Internal (non-exported) helper function. Runs a test case and produces a `TestCaseResult`
operation TestCase<'T : Eq + Show>(name : String, qubits : Qubit[], test_case : (Qubit[]) => 'T, expected : 'T) : TestCaseResult {
    let result = test_case(qubits);
    if result == expected {
        new TestCaseResult { did_pass = true, message = "" }
    } else {
        new TestCaseResult { did_pass = false, message = $"{name}: expected: {expected}, got: {result}" }
    }
}

export CheckAllTestCases, RunAllTestCases; 