// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

namespace Microsoft.Quantum.Convert {
    open Microsoft.Quantum.Diagnostics;

    /// # Summary
    /// Converts a given integer to an equivalent double-precision floating-point number.
    function IntAsDouble(number : Int) : Double {
        body intrinsic;
    }

    /// # Summary
    /// Converts a given integer to an equivalent big integer.
    function IntAsBigInt(number : Int) : BigInt {
        body intrinsic;
    }

    /// # Summary
    /// Converts a `Result` type to a `Bool` type, where `One` is mapped to
    /// `true` and `Zero` is mapped to `false`.
    ///
    /// # Input
    /// ## input
    /// `Result` to be converted.
    ///
    /// # Output
    /// A `Bool` representing the `input`.
    function ResultAsBool(input : Result) : Bool {
        input == One
    }

    /// # Summary
    /// Converts a `Bool` type to a `Result` type, where `true` is mapped to
    /// `One` and `false` is mapped to `Zero`.
    ///
    /// # Input
    /// ## input
    /// `Bool` to be converted.
    ///
    /// # Output
    /// A `Result` representing the `input`.
    function BoolAsResult(input : Bool) : Result {
        if input {One} else {Zero}
    }

    /// # Summary
    /// Produces a non-negative integer from a string of bits in little endian format.
    ///
    /// # Input
    /// ## bits
    /// Bits in binary representation of number.
    function BoolArrayAsInt(bits : Bool[]) : Int {
        let nBits = Length(bits);
        Fact(nBits < 64, $"`Length(bits)` must be less than 64, but was {nBits}.");

        mutable number = 0;
        for i in 0 .. nBits - 1 {
            if (bits[i]) {
                set number |||= 1 <<< i;
            }
        }

        number
    }

    /// # Summary
    /// Produces a binary representation of a non-negative integer, using the
    /// little-endian representation for the returned array.
    ///
    /// # Input
    /// ## number
    /// A non-negative integer to be converted to an array of boolean values.
    /// ## bits
    /// The number of bits in the binary representation of `number`.
    ///
    /// # Output
    /// An array of boolean values representing `number`.
    ///
    /// # Remarks
    /// The input `bits` must be non-negative.
    /// The input `number` must be between 0 and 2^bits - 1.
    function IntAsBoolArray(number : Int, bits : Int) : Bool[] {
        Fact(bits >= 0, "Requested number of bits must be non-negative.");
        Fact(number >= 0, "Number must be non-negative.");
        mutable runningValue = number;
        mutable result = [];
        for _ in 1..bits {
            set result += [ (runningValue &&& 1) != 0 ];
            set runningValue >>>= 1;
        }
        Fact(runningValue == 0, $"`number`={number} is too large to fit into {bits} bits.");

        result
    }

    /// # Summary
    /// Produces a non-negative integer from a string of Results in little-endian format.
    ///
    /// # Input
    /// ## results
    /// Results in binary representation of number.
    ///
    /// # Output
    /// A non-negative integer
    ///
    /// # Example
    /// ```qsharp
    /// // The following returns 1
    /// let int1 = ResultArrayAsInt([One,Zero])
    /// ```
    function ResultArrayAsInt(results : Result[]) : Int {
        let nBits = Length(results);
        Fact(nBits < 64, $"`Length(bits)` must be less than 64, but was {nBits}.");

        mutable number = 0;
        for idxBit in 0 .. nBits - 1 {
            if (results[idxBit] == One) {
                set number |||= 1 <<< idxBit;
            }
        }

        number
    }

    /// # Summary
    /// Converts a `Result[]` type to a `Bool[]` type, where `One`
    /// is mapped to `true` and `Zero` is mapped to `false`.
    ///
    /// # Input
    /// ## input
    /// `Result[]` to be converted.
    ///
    /// # Output
    /// A `Bool[]` representing the `input`.
    function ResultArrayAsBoolArray(input : Result[]) : Bool[] {
        mutable output = [];
        for r in input {
            set output += [r == One];
        }

        output
    }

    /// # Summary
    /// Converts a `Bool[]` type to a `Result[]` type, where `true`
    /// is mapped to `One` and `false` is mapped to `Zero`.
    ///
    /// # Input
    /// ## input
    /// `Bool[]` to be converted.
    ///
    /// # Output
    /// A `Result[]` representing the `input`.
    function BoolArrayAsResultArray(input : Bool[]) : Result[] {
        mutable output = [];
        for b in input {
            set output += [if b {One} else {Zero}];
        }

        output
    }

}
