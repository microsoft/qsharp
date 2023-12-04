// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

namespace Microsoft.Quantum.Arithmetic {
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Measurement;

    /// # Summary
    /// Applies a bitwise-XOR operation between a classical integer and an
    /// integer represented by a register of qubits.
    ///
    /// # Description
    /// Applies `X` operations to qubits in a little-endian register based on
    /// 1 bits in an integer.
    ///
    /// Let us denote `value` by a and let y be an unsigned integer encoded in `target`,
    /// then `ApplyXorInPlace` performs an operation given by the following map:
    /// |y⟩ ↦ |y ⊕ a⟩, where ⊕ is the bitwise exclusive OR operator.
    operation ApplyXorInPlace(value : Int, target : Qubit[]) : Unit is Adj + Ctl {
        body (...) {
            Fact(value >= 0, "`value` must be non-negative.");
            mutable runningValue = value;
            for q in target {
                if (runningValue &&& 1) != 0 {
                    X(q);
                }
                set runningValue >>>= 1;
            }
            Fact(runningValue == 0, "value is too large");
        }
        adjoint self;
    }

    /// # Summary
    /// Applies a bitwise-XOR operation between a classical integer and an
    /// integer represented by a register of qubits.
    ///
    /// # Description
    /// Applies `X` operations to qubits in a little-endian register based on
    /// 1 bits in an integer.
    ///
    /// Let us denote `value` by a and let y be an unsigned integer encoded in `target`,
    /// then `ApplyXorInPlace` performs an operation given by the following map:
    /// |y⟩ ↦ |y ⊕ a⟩, where ⊕ is the bitwise exclusive OR operator.
    operation ApplyXorInPlaceL(value : BigInt, target : Qubit[]) : Unit is Adj + Ctl {
        body (...) {
            Fact(value >= 0L, "`value` must be non-negative.");
            mutable runningValue = value;
            for q in target {
                if (runningValue &&& 1L) != 0L {
                    X(q);
                }
                set runningValue >>>= 1;
            }
            Fact(runningValue == 0L, "`value` is too large.");
        }
        adjoint self;
    }

    /// # Summary
    /// Measures the content of a quantum register and converts
    /// it to an integer. The measurement is performed with respect
    /// to the standard computational basis, i.e., the eigenbasis of `PauliZ`.
    ///
    /// # Input
    /// ## target
    /// A quantum register in the little-endian encoding.
    ///
    /// # Output
    /// An unsigned integer that contains the measured value of `target`.
    ///
    /// # Remarks
    /// This operation resets its input register to the |00...0> state,
    /// suitable for releasing back to a target machine.
    @Config(Full)
    operation MeasureInteger(target : Qubit[]) : Int {
        let nBits = Length(target);
        Fact(nBits < 64, $"`Length(target)` must be less than 64, but was {nBits}.");

        mutable number = 0;
        for i in 0..nBits-1 {
            if (MResetZ(target[i]) == One) {
                set number |||= 1 <<< i;
            }
        }

        number
    }

}
