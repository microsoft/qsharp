// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

namespace Microsoft.Quantum.Arrays {
    open Microsoft.Quantum.Diagnostics;
    //open Microsoft.Quantum.Math;

    /// # Summary
    /// Splits an array into multiple parts of equal length.
    ///
    /// # Input
    /// ## nElements
    /// The length of each chunk. Must be positive.
    /// ## arr
    /// The array to be split.
    ///
    /// # Output
    /// A array containing each chunk of the original array.
    ///
    /// # Remarks
    /// Note that the last element of the output may be shorter
    /// than `nElements` if `Length(arr)` is not divisible by `nElements`.
    //function Chunks<'T>(nElements : Int, arr : 'T[]) : 'T[][] {
    //    Fact(nElements > 0, "nElements must be positive");
    //    mutable output = [];
    //    mutable remaining = arr;
    //    while (not IsEmpty(remaining)) {
    //        let nElementsToTake = MinI(Length(remaining), nElements);
    //        set output += [remaining[...nElementsToTake - 1]];
    //        set remaining = remaining[nElementsToTake...];
    //    }
    //    output
    //}

    /// # Summary
    /// Returns the first element of the array.
    ///
    /// # Type Parameters
    /// ## 'A
    /// The type of the array elements.
    ///
    /// # Input
    /// ## array
    /// Array of which the first element is taken. Array must have at least 1 element.
    ///
    /// # Output
    /// The first element of the array.
    function Head<'A> (array : 'A[]) : 'A {
        Fact(Length(array) > 0, "Array must be of the length at least 1");
        array[0]
    }

    /// # Summary
    /// Returns true if and only if an array is empty.
    ///
    /// # Input
    /// ## array
    /// The array to be checked.
    ///
    /// # Output
    /// `true` if and only if the array is empty (has length 0).
    //function IsEmpty<'T>(array : 'T[]) : Bool {
    //    return Length(array) == 0;
    //}

    /// # Summary
    /// Creates an array that is equal to an input array except that the last array
    /// element is dropped.
    ///
    /// # Type Parameters
    /// ## 'T
    /// The type of the array elements.
    ///
    /// # Input
    /// ## array
    /// An array whose first to second-to-last elements are to form the output array.
    ///
    /// # Output
    /// An array containing the elements `array[0..Length(array) - 2]`.
    function Most<'T> (array : 'T[]) : 'T[] {
        array[... Length(array) - 2]
    }

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
    /// for idx in IndexRange(array) { ... }
    /// for idx in 0 .. Length(array) - 1 { ... }
    /// ```
    function IndexRange<'TElement>(array : 'TElement[]) : Range {
       0 .. Length(array) - 1
    }

    /// # Summary
    /// Creates an array that is equal to an input array except that the first array
    /// element is dropped.
    ///
    /// # Type Parameters
    /// ## 'T
    /// The type of the array elements.
    ///
    /// # Input
    /// ## array
    /// An array whose second to last elements are to form the output array.
    ///
    /// # Output
    /// An array containing the elements `array[1..Length(array) - 1]`.
    function Rest<'T> (array : 'T[]) : 'T[] {
        array[1 ...]
    }

    /// # Summary
    /// Create an array that contains the same elements as an input array but in reversed
    /// order.
    ///
    /// # Type Parameters
    /// ## 'T
    /// The type of the array elements.
    ///
    /// # Input
    /// ## array
    /// An array whose elements are to be copied in reversed order.
    ///
    /// # Output
    /// An array containing the elements `array[Length(array) - 1]` .. `array[0]`.
    function Reversed<'T>(array : 'T[]) : 'T[] {
        array[... -1 ...]
    }

    /// # Summary
    /// Returns the last element of the array.
    ///
    /// # Type Parameters
    /// ## 'A
    /// The type of the array elements.
    ///
    /// # Input
    /// ## array
    /// Array of which the last element is taken. Array must have at least 1 element.
    ///
    /// # Output
    /// The last element of the array.
    function Tail<'A> (array : 'A[]) : 'A {
        Fact(Length(array) > 0, "Array must be of the length at least 1");
        array[Length(array) - 1]
    }
}
