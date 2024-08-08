// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

    /// # Summary
    /// Represents a register of qubits encoding a fixed-point number. Consists of an integer that is equal to the number of
    /// qubits to the left of the binary point, i.e., qubits of weight greater
    /// than or equal to 1, and a quantum register.
    newtype FixedPoint = (IntegerBits : Int, Register : Qubit[]);
