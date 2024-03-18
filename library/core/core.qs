// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

namespace Microsoft.Quantum.Core {
    /// # Summary
    /// Returns the number of elements in an array.
    ///
    /// # Input
    /// ## a
    /// Input array.
    ///
    /// # Output
    /// The total count of elements in an array.
    function Length<'T>(a : 'T[]) : Int {
        body intrinsic;
    }

    /// # Summary
    /// Creates an array of given length with all elements equal to given value.
    ///
    /// # Input
    /// ## value
    /// The value of each element of the new array.
    /// ## length
    /// Length of the new array.
    ///
    /// # Output
    /// A new array of length `length`, such that every element is `value`.
    ///
    /// # Example
    /// The following code creates an array of 3 Boolean values, each equal to `true`:
    /// ```qsharp
    /// let array = Repeated(true, 3);
    /// ```
    function Repeated<'T>(value : 'T, length : Int) : 'T[] {
        if length < 0 {
            fail "Length must be a non-negative integer";
        }

        mutable output = [];
        for _ in 1 .. length {
            set output += [value];
        }

        output
    }
}
