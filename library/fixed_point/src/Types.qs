// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

/// # Summary
/// Represents a register of qubits encoding a fixed-point number. Consists of an integer that is equal to the number of
/// qubits to the left of the binary point, i.e., qubits of weight greater
/// than or equal to 1, and a quantum register.
/// There should always be at least 2 integer bits in the fixed-point number. The first one represents the sign, the second one is the most significant bit of the int.
/// There must be at least one bit left for the fixed point, so a minimum of 3 qubits are required total for this representation.
struct FixedPoint { IntegerBits : Int, Register : Qubit[] }
