// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

namespace Microsoft.Quantum.Arrays {

    /// # Summary
    /// Given an array, returns a range over the indices of that array, suitable
    /// for use in a for loop.
    ///
    /// # Type Parameters
    /// ## 'TElement
    /// The type of elements of the array.
    ///
    /// # Input
    /// ## array
    /// An array for which a range of indices should be returned.
    ///
    /// # Output
    /// A range over all indices of the array.
    ///
    /// # Example
    /// The following `for` loops are equivalent:
    /// ```qsharp
    /// for (idx in IndexRange(array)) { ... }
    /// for (idx in 0 .. Length(array) - 1) { ... }
    /// ```
    function IndexRange<'TElement>(array : 'TElement[]) : Range {
       return 0..(Length(array) - 1);
    }

    /// # Summary
    /// Create an array that contains the same elements as an input array but in Reversed
    /// order.
    ///
    /// # Type Parameters
    /// ## 'T
    /// The type of the array elements.
    ///
    /// # Input
    /// ## array
    /// An array whose elements are to be copied in Reversed order.
    ///
    /// # Output
    /// An array containing the elements `array[Length(array) - 1]` .. `array[0]`.
    function Reversed<'T>(array : 'T[]) : 'T[] {
        let nElements = Length(array);
        return array[nElements-1..-1..0];
    }

}