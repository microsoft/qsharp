// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.
function Main() : Unit {
    Std.Diagnostics.Fact(
        Functions.TestCases([
            ("Should return 42", TestCaseOne, 42),
            ("Should add one", () -> AddOne(5), 6)
        ]),
        "Test harness failed to return true for all passing tests."
    );
    Std.Diagnostics.Fact(
        Length(Functions.TestCasesSilent([
            ("Should return 42", TestCaseOne, 43),
            ("Should add one", () -> AddOne(5), 42)
        ])) == 2,
        "Test harness failed to return messages for failing tests."
    );
}

function TestCaseOne() : Int {
    42
}

function AddOne(x : Int) : Int {
    x + 1
}