// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

namespace Microsoft.Quantum.Arithmetic {
    open Microsoft.Quantum.Arrays;

    /// # Summary
    /// Converts a `LittleEndian` qubit register to a `BigEndian` qubit
    /// register by reversing the qubit ordering.
    ///
    /// # Input
    /// ## input
    /// Qubit register in `LittleEndian` format.
    ///
    /// # Output
    /// Qubit register in `BigEndian` format.
    function LittleEndianAsBigEndian(input: Qubit[]) : Qubit[] {
        // TODO: BigEndian, LittleEndian
        return Reversed(input);
    }

    /// # Summary
    /// Converts a `BigEndian` qubit register to a `LittleEndian` qubit
    /// register by reversing the qubit ordering.
    ///
    /// # Input
    /// ## input
    /// Qubit register in `BigEndian` format.
    ///
    /// # Output
    /// Qubit register in `LittleEndian` format.
    function BigEndianAsLittleEndian(input: Qubit[]) : Qubit[] {
        // TODO: BigEndian, LittleEndian
        return Reversed(input!);
    }

}