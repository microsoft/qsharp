// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

namespace Microsoft.Quantum.Arrays {
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Math;

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
    function Chunks<'T>(nElements : Int, arr : 'T[]) : 'T[][] {
        Fact(nElements > 0, "nElements must be positive");
        mutable output = [];
        mutable remaining = arr;
        while (not IsEmpty(remaining)) {
            let nElementsToTake = MinI(Length(remaining), nElements);
            set output += [remaining[...nElementsToTake - 1]];
            set remaining = remaining[nElementsToTake...];
        }

        output
    }

    /// # Summary
    /// Returns an array of diagonal elements of a 2-dimensional array
    ///
    /// # Description
    /// If the 2-dimensional array has not a square shape, the diagonal over
    /// the minimum over the number of rows and columns will be returned.
    ///
    /// # Type Parameters
    /// ## 'T
    /// The type of each element of `matrix`.
    ///
    /// # Input
    /// ## matrix
    /// 2-dimensional matrix in row-wise order
    ///
    /// # Example
    /// ```qsharp
    /// let matrix = [[1, 2, 3], [4, 5, 6], [7, 8, 9]];
    /// let diagonal = Diagonal(matrix);
    /// // same as: column = [1, 5, 9]
    /// ```
    ///
    /// # See Also
    /// - Microsoft.Quantum.Arrays.Transposed
    function Diagonal<'T>(matrix : 'T[][]) : 'T[] {
        Fact(IsRectangularArray(matrix), "Matrix is not a rectangular array");
        let rows = Length(matrix);
        let columns = rows == 0 ? 0 | Length(Head(matrix));
        let rangeLimit = MinI(rows, columns) - 1;
        mutable diagonal = [];
        for index in 0..rangeLimit {
            set diagonal += [matrix[index][index]];
        }

        diagonal
    }

    /// # Summary
    /// Returns the at the given index of an array.
    ///
    /// # Type Parameters
    /// ## 'T
    /// The type of each element of `array`.
    ///
    /// # Input
    /// ## index
    /// Index of element
    /// ## array
    /// The array being indexed.
    ///
    /// # Remark
    /// This function is more general than `LookupFunction`, since
    /// it can also be used for partial application on a fixed index.
    /// Note that the type parameter must explicitly be provided in
    /// this case as it cannot be deduced automatically.
    ///
    /// # Example
    /// Get the third number in four famous integer sequences. (note
    /// that the 0 index corresponds to the _first_ value of the sequence.)
    /// ```qsharp
    /// let lucas = [2, 1, 3, 4, 7, 11, 18, 29, 47, 76];
    /// let prime = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29];
    /// let fibonacci = [0, 1, 1, 2, 3, 5, 8, 13, 21, 34];
    /// let catalan = [1, 1, 2, 5, 14, 42, 132, 429, 1430, 4862];
    /// let famous2 = Mapped(ElementAt<Int>(2, _), [lucas, prime, fibonacci, catalan]);
    /// // same as: famous2 = [3, 5, 1, 2]
    /// ```
    ///
    /// # See Also
    /// - Microsoft.Quantum.Arrays.LookupFunction
    /// - Microsoft.Quantum.Arrays.ElementsAt
    function ElementAt<'T>(index : Int, array : 'T[]) : 'T {
        Fact(index >= 0 and index < Length(array), "Index is out of bound");
        array[index]
    }

    /// # Summary
    /// Returns the array's elements at a given range
    /// of indices.
    ///
    /// # Type Parameters
    /// ## 'T
    /// The type of each element of `array`.
    ///
    /// # Input
    /// ## range
    /// Range of array indexes
    /// ## array
    /// Array
    ///
    /// # Example
    /// Get the odd indexes in famous integer sequences. (note
    /// that the 0 index corresponds to the _first_ value of the sequence.)
    /// ```qsharp
    /// let lucas = [2, 1, 3, 4, 7, 11, 18, 29, 47, 76];
    /// let prime = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29];
    /// let fibonacci = [0, 1, 1, 2, 3, 5, 8, 13, 21, 34];
    /// let catalan = [1, 1, 2, 5, 14, 42, 132, 429, 1430, 4862];
    /// let famousOdd = Mapped(ElementsAt<Int>(0..2..9, _), [lucas, prime, fibonacci, catalan]);
    /// // same as: famousOdd = [[2, 3, 7, 18, 47], [2, 5, 11, 17, 23], [0, 1, 3, 8, 21], [1, 2, 14, 132, 1430]]
    /// ```
    ///
    /// # See Also
    /// - Microsoft.Quantum.Arrays.ElementAt
    /// - Microsoft.Quantum.Arrays.LookupFunction
    function ElementsAt<'T>(range : Range, array : 'T[]) : 'T[] {
        array[range]
    }

    /// # Summary
    /// Returns an array containing the elements of another array,
    /// excluding elements at a given list of indices.
    ///
    /// # Type Parameters
    /// ## 'T
    /// The type of the array elements.
    ///
    /// # Input
    /// ## remove
    /// An array of indices denoting which elements should be excluded
    /// from the output.
    /// ## array
    /// Array of which the values in the output array are taken.
    ///
    /// # Output
    /// An array `output` such that `output[0]` is the first element
    /// of `array` whose index does not appear in `remove`,
    /// such that `output[1]` is the second such element, and so
    /// forth.
    ///
    /// # Example
    /// ```qsharp
    /// let array = [10, 11, 12, 13, 14, 15];
    /// // The following line returns [10, 12, 15].
    /// let subarray = Excluding([1, 3, 4], array);
    /// ```
    function Excluding<'T>(remove : Int[], array : 'T[]) : 'T[] {
        let nElements = Length(array);
        mutable toKeep = [true, size = nElements];
        for indexToRemove in remove {
            if indexToRemove >= nElements {
                fail "Index is out of bound";
            }
            set toKeep w/= indexToRemove <- false;
        }

        // N.B. This would be better using the `Count` function once it is implemented.
        mutable outputCount = 0;
        for keep in toKeep {
            if keep {
                set outputCount += 1;
            }
        }

        if (outputCount == 0)
        {
            return [];
        }

        mutable output = [array[0], size = outputCount];
        mutable outputIndex = 0;
        for index in 0..nElements-1 {
            if toKeep[index] {
                set output w/= outputIndex <- array[index];
                set outputIndex += 1;
            }
        }

        output
    }

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
    /// Interleaves two arrays of (almost) same size.
    ///
    /// # Description
    /// This function returns the interleaving of two arrays, starting
    /// with the first element from the first array, then the first
    /// element from the second array, and so on.
    ///
    /// The first array must either be
    /// of the same length as the second one, or can have one more element.
    ///
    /// # Type Parameters
    /// ## 'T
    /// The type of each element of `first` and `second`.
    ///
    /// # Input
    /// ## first
    /// The first array to be interleaved.
    ///
    /// ## second
    /// The second array to be interleaved.
    ///
    /// # Output
    /// Interleaved array
    ///
    /// # Example
    /// ```qsharp
    /// // same as interleaved = [1, -1, 2, -2, 3, -3]
    /// let interleaved = Interleaved([1, 2, 3], [-1, -2, -3])
    /// ```
    function Interleaved<'T>(first : 'T[], second : 'T[]) : 'T[] {
        let firstLength = Length(first);
        let secondLength = Length(second);
        Fact(
            firstLength >= secondLength and firstLength - secondLength <= 1,
            "Array `first` must either be of same size as `second` or have one more element");

        if firstLength == 0 {
            return [];
        }

        let interleavedLength = firstLength + secondLength;
        mutable interleaved = [first[0], size = interleavedLength];
        for index in 0..(interleavedLength - 1) {
            let originalIndex = index / 2;
            let value =
                if index % 2 == 0 {first[originalIndex]}
                else {second[originalIndex]};

            set interleaved w/= index <- value;
        }

        interleaved
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
    function IsEmpty<'T>(array : 'T[]) : Bool {
        Length(array) == 0
    }

    /// # Summary
    /// Returns whether a 2-dimensional array has a rectangular shape
    ///
    /// # Type Parameters
    /// ## 'T
    /// The type of each element of `array`.
    ///
    /// # Input
    /// ## array
    /// A 2-dimensional array of elements
    ///
    /// # Example
    /// ```qsharp
    /// RectangularArrayFact([[1, 2], [3, 4]], "Array is not rectangular");       // true
    /// RectangularArrayFact([[1, 2, 3], [4, 5, 6]], "Array is not rectangular"); // true
    /// RectangularArrayFact([[1, 2], [3, 4, 5]], "Array is not rectangular");    // false
    /// ```
    ///
    /// # See Also
    /// - Microsoft.Quantum.Arrays.IsSquareArray
    function IsRectangularArray<'T>(array : 'T[][]) : Bool {
        if (Length(array) > 0) {
            let columns = Length(Head(array));
            for idx in IndexRange(Rest(array)) {
                if Length(array[idx+1]) != columns {
                    return false;
                }
            }
        }

        true
    }

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
