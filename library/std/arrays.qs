// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

namespace Microsoft.Quantum.Arrays {
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Math;

    /// # Summary
    /// Splits an array into multiple parts of equal length.
    ///
    /// # Input
    /// ## chunkSize
    /// The length of each chunk. Must be positive.
    /// ## array
    /// The array to be split in chunks.
    ///
    /// # Output
    /// A array containing each chunk of the original array.
    ///
    /// # Remarks
    /// Note that the last element of the output may be shorter
    /// than `chunkSize` if `Length(array)` is not divisible by `chunkSize`.
    function Chunks<'T>(chunkSize : Int, array : 'T[]) : 'T[][] {
        Fact(chunkSize > 0, "`chunkSize` must be positive");
        mutable output = [];
        mutable remaining = array;
        while (not IsEmpty(remaining)) {
            let chunkSizeToTake = MinI(Length(remaining), chunkSize);
            set output += [remaining[...chunkSizeToTake - 1]];
            set remaining = remaining[chunkSizeToTake...];
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
    /// 2-dimensional matrix in row-wise order.
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
        Fact(IsRectangularArray(matrix), "`matrix` is not a rectangular array");
        let rows = Length(matrix);
        let columns = rows == 0 ? 0 | Length(Head(matrix));
        let rangeLimit = MinI(rows, columns) - 1;
        mutable diagonal = [];
        for index in 0 .. rangeLimit {
            set diagonal += [matrix[index][index]];
        }

        diagonal
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
    /// An array of indices denoting which elements should be excluded.
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
        let arrayLength = Length(array);
        mutable toKeep = Repeated(true, arrayLength);
        for indexToRemove in remove {
            set toKeep w/= indexToRemove <- false;
        }
        mutable output = [];
        for index in 0 .. arrayLength - 1 {
            if toKeep[index] {
                set output += [array[index]];
            }
        }
        output
    }

    /// # Summary
    /// Iterates a function `f` through an array `array`, returning
    /// `f(...f(f(initialState, array[0]), array[1]), ...)`.
    ///
    /// # Type Parameters
    /// ## 'State
    /// The type of states the `folder` function operates on, i.e., accepts as its first
    /// argument and returns.
    /// ## 'T
    /// The type of `array` elements.
    ///
    /// # Input
    /// ## folder
    /// A function to be folded over the array.
    /// ## state
    /// The initial state of the folder.
    /// ## array
    /// An array of values to be folded over.
    ///
    /// # Output
    /// The final state returned by the folder after iterating over
    /// all elements of `array`.
    ///
    /// # Example
    /// ```qsharp
    /// let sum = Fold((x, y) -> x + y, 0, [1, 2, 3, 4, 5]); // `sum` is 15.
    /// ```
    function Fold<'State, 'T> (folder : (('State, 'T) -> 'State), state : 'State, array : 'T[]) : 'State {
        mutable current = state;
        for element in array {
            set current = folder(current, element);
        }
        current
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
            firstLength == secondLength or firstLength == secondLength + 1,
            "Array `first` must either be of same size as `second` or have one more element");

        let interleavedLength = firstLength + secondLength;
        mutable interleaved = [];
        for index in 0..interleavedLength - 1 {
            let originalIndex = index / 2;
            let value =
                if index % 2 == 0 {first[originalIndex]}
                else {second[originalIndex]};
            set interleaved += [value];
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
    /// A 2-dimensional array of elements.
    ///
    /// # Output
    /// `true` if the array is rectangular, `false` otherwise.
    ///
    /// # Example
    /// ```qsharp
    /// IsRectangularArray([[1, 2], [3, 4]]);       // true
    /// IsRectangularArray([[1, 2, 3], [4, 5, 6]]); // true
    /// IsRectangularArray([[1, 2], [3, 4, 5]]);    // false
    /// ```
    ///
    /// # See Also
    /// - Microsoft.Quantum.Arrays.IsSquareArray
    function IsRectangularArray<'T>(array : 'T[][]) : Bool {
        if (Length(array) > 0) {
            let columnCount = Length(Head(array));
            for index in 1 .. Length(array) - 1 {
                if Length(array[index]) != columnCount {
                    return false;
                }
            }
        }

        true
    }

    /// # Summary
    /// Returns whether a 2-dimensional array has a square shape
    ///
    /// # Type Parameters
    /// ## 'T
    /// The type of each element of `array`.
    ///
    /// # Input
    /// ## array
    /// A 2-dimensional array of elements.
    ///
    /// # Example
    /// ```qsharp
    /// IsSquareArray([[1, 2], [3, 4]]);         // true
    /// IsSquareArray([[1, 2, 3], [4, 5, 6]]);   // false
    /// IsSquareArray([[1, 2], [3, 4], [5, 6]]); // false
    /// ```
    ///
    /// # Output
    /// `true` if the array is square, `false` otherwise.
    ///
    /// # See Also
    /// - Microsoft.Quantum.Arrays.IsRectangularArray
    function IsSquareArray<'T>(array : 'T[][]) : Bool {
        if (Length(array) > 0) {
            let columnCount = Length(array);
            for column in array {
                if Length(column) != columnCount {
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
    /// Returns an array padded at with specified values up to a
    /// specified length.
    ///
    /// # Type Parameters
    /// ## 'T
    /// The type of the array elements.
    ///
    /// # Input
    /// ## paddedLength
    /// The length of the padded array. If this is positive, `array`
    /// is padded at the head. If this is negative, `array` is padded
    /// at the tail.
    /// ## defaultElement
    /// Default value to use for padding elements.
    /// ## array
    /// Array to be padded.
    ///
    /// # Output
    /// An array `output` that is the `array` padded at the head or the tail
    /// with `defaultElement`s until `output` has length `paddedLength`
    ///
    /// # Example
    /// ```qsharp
    /// let array = [10, 11, 12];
    /// // The following line returns [10, 12, 15, 2, 2].
    /// let output = Padded(-5, 2, array);
    /// // The following line returns [2, 2, 10, 12, 15].
    /// let output = Padded(5, 2, array);
    /// ```
    function Padded<'T> (paddedLength : Int, defaultElement : 'T, inputArray : 'T[]) : 'T[] {
        let nElementsInitial = Length(inputArray);
        let nAbsElementsTotal = AbsI(paddedLength);
        if nAbsElementsTotal <= nElementsInitial {
            fail "Specified output array length must be longer than `inputArray` length.";
        }
        let nElementsPad = nAbsElementsTotal - nElementsInitial;
        let padArray = Repeated(defaultElement, nElementsPad);
        if (paddedLength >= 0) {
            padArray + inputArray // Padded at head.
        } else {
            inputArray + padArray // Padded at tail.
        }
    }

    /// # Summary
    /// Splits an array into multiple parts.
    ///
    /// # Input
    /// ## partitionSizes
    /// Number of elements in each splitted part of array.
    /// ## array
    /// Input array to be split.
    ///
    /// # Output
    /// Multiple arrays where the first array is the first `partitionSizes[0]` of `array`
    /// and the second array are the next `partitionSizes[1]` of `array` etc. The last array
    /// will contain all remaining elements. If the array is split exactly, the
    /// last array will be the empty array, indicating there are no remaining elements.
    /// In other words, `Tail(Partitioned(...))` will always return the remaining
    /// elements, while `Most(Partitioned(...))` will always return the complete
    /// partitions of the array.
    ///
    /// # Example
    /// ```qsharp
    /// // The following returns [[2, 3], [5], [7]];
    /// let split = Partitioned([2, 1], [2, 3, 5, 7]);
    /// // The following returns [[2, 3], [5, 7], []];
    /// let split = Partitioned([2, 2], [2, 3, 5, 7]);
    /// ```
    function Partitioned<'T>(partitionSizes: Int[], array: 'T[]) : 'T[][] {
        mutable output = Repeated([], Length(partitionSizes) + 1);
        mutable partitionStartIndex = 0;
        for index in IndexRange(partitionSizes) {
            let partitionEndIndex = partitionStartIndex + partitionSizes[index] - 1;
            if partitionEndIndex >= Length(array) {
                fail "Partitioned argument out of bounds.";
            }
            set output w/= index <- array[partitionStartIndex .. partitionEndIndex];
            set partitionStartIndex = partitionEndIndex + 1;
        }
        set output w/= Length(partitionSizes) <- array[partitionStartIndex .. Length(array) - 1];
        output
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
    /// Get an array of integers in a given interval.
    ///
    /// # Input
    /// ## from
    /// An inclusive start index of the interval.
    /// ## to
    /// An inclusive end index of the interval that is not smaller than `from`.
    ///
    /// # Output
    /// An array containing the sequence of numbers `from`, `from + 1`, ...,
    /// `to`.
    ///
    /// # Example
    /// ```qsharp
    /// let arr1 = SequenceI(0, 3); // [0, 1, 2, 3]
    /// let arr2 = SequenceI(23, 29); // [23, 24, 25, 26, 27, 28, 29]
    /// let arr3 = SequenceI(-5, -2); // [-5, -4, -3, -2]
    ///
    /// let numbers = SequenceI(0, _); // function to create sequence from 0 to `to`
    /// let naturals = SequenceI(1, _); // function to create sequence from 1 to `to`
    /// ```
    function SequenceI (from : Int, to : Int) : Int[] {
        Fact(to >= from, $"`to` must be larger than `from`.");
        mutable array = [];
        for index in from .. to {
            set array += [index];
        }
        array
    }

    /// # Summary
    /// Takes an array and a list of locations and
    /// produces a new array formed from the elements of the original
    /// array that match the given locations.
    ///
    /// # Remarks
    /// If `locations` contains repeated elements, the corresponding elements 
    /// of `array` will likewise be repeated.
    ///
    /// # Type Parameters
    /// ## 'T
    /// The type of `array` elements.
    ///
    /// # Input
    /// ## locations
    /// A list of locations in the input array that is used to define the subarray.
    /// ## array
    /// An array from which a subarray will be generated.
    ///
    /// # Output
    /// An array `out` of elements whose locations correspond to the subarray,
    /// such that `out[index] == array[locations[index]]`.
    /// 
    /// # Example
    /// 
    /// ```qsharp
    /// let array = [1, 2, 3, 4];
    /// let permutation = Subarray([3, 0, 2, 1], array); // [4, 1, 3, 2]
    /// let duplicates = Subarray([1, 2, 2], array);     // [2, 3, 3]
    /// ```
    function Subarray<'T> (locations : Int[], array : 'T[]) : 'T[] {
        mutable subarray = [];
        for location in locations {
            set subarray += [array[location]];
        }
        subarray
    }

    /// # Summary
    /// Returns the transpose of a matrix represented as an array
    /// of arrays.
    ///
    /// # Description
    /// Input as an $r \times c$ matrix with $r$ rows and $c$ columns.  The matrix
    /// is row-based, i.e., `matrix[i][j]` accesses the element at row $i$ and column $j$.
    ///
    /// This function returns the $c \times r$ matrix that is the transpose of the
    /// input matrix.
    ///
    /// # Type Parameters
    /// ## 'T
    /// The type of each element of `matrix`.
    ///
    /// # Input
    /// ## matrix
    /// Row-based $r \times c$ matrix.
    ///
    /// # Output
    /// Transposed $c \times r$ matrix.
    ///
    /// # Example
    /// ```qsharp
    /// // same as [[1, 4], [2, 5], [3, 6]]
    /// let transposed = Transposed([[1, 2, 3], [4, 5, 6]]);
    /// ```
    function Transposed<'T>(matrix : 'T[][]) : 'T[][] {
        let rowCount = Length(matrix);
        Fact(rowCount > 0, "Matrix must have at least 1 row");
        let columnCount = Length(Head(matrix));
        Fact(columnCount > 0, "Matrix must have at least 1 column");
        Fact(IsRectangularArray(matrix), "Matrix is not a rectangular array");
        mutable transposed = [];
        for columnIndex in 0 .. columnCount - 1 {
            mutable newRow = [];
            for rowIndex in 0 .. rowCount - 1 {
                set newRow += [matrix[rowIndex][columnIndex]];
            }
            set transposed += [newRow];
        }
        transposed
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
        array[Length(array) - 1]
    }

    /// # Summary
    /// Given an array of 2-tuples, returns a tuple of two arrays, each containing
    /// the elements of the tuples of the input array.
    ///
    /// # Type Parameters
    /// ## 'T
    /// The type of the first element in each tuple.
    /// ## 'U
    /// The type of the second element in each tuple.
    ///
    /// # Input
    /// ## array
    /// An array containing 2-tuples.
    ///
    /// # Output
    /// Two arrays, the first one containing all first elements of the input
    /// tuples, the second one containing all second elements of the input tuples.
    ///
    /// # Example
    /// ```qsharp
    /// // split is same as ([5, 4, 3, 2, 1], [true, false, true, true, false])
    /// let split = Unzipped([(5, true), (4, false), (3, true), (2, true), (1, false)]);
    /// ```
    ///
    /// # See Also
    /// - Microsoft.Quantum.Arrays.Zipped
    function Unzipped<'T, 'U>(array : ('T, 'U)[]) : ('T[], 'U[]) {
        mutable first = [];
        mutable second = [];
        for index in 0 .. Length(array) - 1  {
            let (left, right) = array[index];
            set first += [left];
            set second += [right];
        }
        return (first, second);
    }

    /// # Summary
    /// Returns all consecutive subarrays of length `size`.
    ///
    /// # Description
    /// This function returns all `n - size + 1` subarrays of
    /// length `size` in order, where `n` is the length of `array`.
    /// The first subarrays are `array[0..size - 1], array[1..size], array[2..size + 1]`
    /// until the last subarray `array[n - size..n - 1]`.
    ///
    /// # Type Parameters
    /// ## 'T
    /// The type of `array` elements.
    ///
    /// # Input
    /// ## size
    /// Length of the subarrays.
    ///
    /// ## array
    /// An array of elements.
    ///
    /// # Example
    /// ```qsharp
    /// // same as [[1, 2, 3], [2, 3, 4], [3, 4, 5]]
    /// let windows = Windows(3, [1, 2, 3, 4, 5]);
    /// ```
    ///
    /// # Remarks
    /// The size of the window must be a positive integer no greater than the size of the array
    function Windows<'T>(size : Int, array : 'T[]) : 'T[][] {
        let arrayLength = Length(array);
        Fact(
            size > 0 or size <= arrayLength,
            "The size of the window must be a positive integer no greater than the size of the array");

        mutable windows = [];
        for index in 0 .. arrayLength - size {
            set windows += [array[index .. index + size - 1]];
        }
        windows
    }

    /// # Summary
    /// Given two arrays, returns a new array of pairs such that each pair
    /// contains an element from each original array.
    ///
    /// # Type Parameters
    /// ## 'T
    /// The type of the left array elements.
    /// ## 'U
    /// The type of the right array elements.
    ///
    /// # Input
    /// ## left
    /// An array containing values for the first element of each tuple.
    /// ## right
    /// An array containing values for the second element of each tuple.
    ///
    /// # Output
    /// An array containing pairs of the form `(left[index], right[index])` for
    /// each `index`. If the two arrays are not of equal length, the output will
    /// be as long as the shorter of the inputs.
    ///
    /// # Example
    /// ```qsharp
    /// let left = [1, 3, 71];
    /// let right = [false, true];
    /// let pairs = Zipped(left, right); // [(1, false), (3, true)]
    /// ```
    ///
    /// # See Also
    /// - Microsoft.Quantum.Arrays.Unzipped
    function Zipped<'T, 'U>(left : 'T[], right : 'U[]) : ('T, 'U)[] {
        let arrayLength = MinI(Length(left), Length(right));
        mutable zipped = [];
        for index in 0 .. arrayLength - 1 {
            set zipped += [(left[index], right[index])];
        }
        zipped
    }
}
