// # Sample
// Test Attribute
//
// # Description
// A Q# function or operation (callable) can be designated as a test case via the @Test() attribute.
// In VS Code, these tests will show up in the "test explorer" in the Activity Bar.
// If the test crashes, it is a failure. If it runs to completion, it is a success.

// Tests must take zero parameters, and contain no generic types (type parameters).
@Test()
function TestPass() : Unit {
    Std.Diagnostics.Fact(true, "This test should pass.");
}

// Because this function asserts `false`, it will crash and the test will fail.
@Test()
function TestFail() : Unit {
    Std.Diagnostics.Fact(false, "This test should fail.");
}

function Main() : Unit {}