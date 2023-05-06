// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

namespace Microsoft.Quantum.Arrays {
    open Microsoft.Quantum.Diagnostics;

    /// # Summary
    /// Create an array that contains the same elements as an input array but in Reversed
    /// order.
    function Reversed<'T>(array : 'T[]) : 'T[] {
        let nElements = Length(array);
        array[nElements-1..-1..0]
    }

    /// # Summary
    /// Returns the first element of the array.
    function Head<'A> (array : 'A[]) : 'A {
        Fact(Length(array) > 0, "Array must be of the length at least 1");
        array[0]
    }

    /// # Summary
    /// Creates an array that is equal to an input array except that the first array
    /// element is dropped: `array[1..Length(array) - 1]`.
    function Rest<'T> (array : 'T[]) : 'T[] {
        array[1 ...]
    }

    /// # Summary
    /// Returns the last element of the array.
    function Tail<'A> (array : 'A[]) : 'A {
        Fact(Length(array) > 0, "Array must be of the length at least 1");
        array[Length(array) - 1]
    }

    /// # Summary
    /// Creates an array that is equal to an input array except that the last array
    /// element is dropped: `array[0..Length(array) - 2]`.
    function Most<'T> (array : 'T[]) : 'T[] {
        array[... Length(array) - 2]
    }

    /// Given an array, returns a range over the indices of that array, suitable
    /// for use in a for loop.
    function IndexRange<'TElement>(array : 'TElement[]) : Range {
       0 .. array::Length - 1
    }
}
