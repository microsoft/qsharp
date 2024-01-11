// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

namespace Microsoft.Quantum.Measurement {
    open Microsoft.Quantum.Core;
    open Microsoft.Quantum.Intrinsic;
    open Microsoft.Quantum.Diagnostics;
    open QIR.Intrinsic;

    /// # Summary
    /// Jointly measures a register of qubits in the Pauli Z basis.
    ///
    /// # Description
    /// Measures a register of qubits in the `Z ⊗ Z ⊗ ••• ⊗ Z`
    /// basis, representing the parity of the entire register.
    ///
    /// # Input
    /// ## register
    /// The register to be measured.
    ///
    /// # Output
    /// The result of measuring `Z ⊗ Z ⊗ ••• ⊗ Z`.
    ///
    /// # Remarks
    /// This operation does not reset the measured qubits to the |0⟩ state,
    /// leaving them in the state that corresponds to the measurement result.
    operation MeasureAllZ (register : Qubit[]) : Result {
        Measure(Repeated(PauliZ, Length(register)), register)
    }

    /// # Summary
    /// Measures each qubit in a given array in the standard basis.
    /// # Input
    /// ## targets
    /// An array of qubits to be measured.
    /// # Output
    /// An array of measurement results.
    ///
    /// # Remarks
    /// This operation does not reset the measured qubits to the |0⟩ state,
    /// leaving them in the state that corresponds to the measurement results.
    operation MeasureEachZ (register : Qubit[]) : Result[] {
        mutable results = [];
        for qubit in register {
            set results += [M(qubit)];
        }
        results
    }

    /// # Summary
    /// Measures each qubit in a given array in the Z basis
    /// and resets them to a fixed initial state.
    /// # Input
    /// ## targets
    /// An array of qubits to be measured.
    /// # Output
    /// An array of measurement results.
    operation MResetEachZ (register : Qubit[]) : Result[] {
        mutable results = [];
        for qubit in register {
            set results += [MResetZ(qubit)];
        }
        results
    }

    /// # Summary
    /// Measures a single qubit in the X basis,
    /// and resets it to a fixed initial state
    /// following the measurement.
    ///
    /// # Description
    /// Performs a single-qubit measurement in the X-basis,
    /// and ensures that the qubit is returned to |0⟩
    /// following the measurement.
    ///
    /// # Input
    /// ## target
    /// A single qubit to be measured.
    ///
    /// # Output
    /// The result of measuring `target` in the Pauli X basis.
    operation MResetX (target : Qubit) : Result {
        // Map the qubit's state from the Z-basis to the X-basis.
        // Then measure and reset the qubit.
        H(target);
        MResetZ(target)
    }

    /// # Summary
    /// Measures a single qubit in the Y basis,
    /// and resets it to a fixed initial state
    /// following the measurement.
    ///
    /// # Description
    /// Performs a single-qubit measurement in the Y-basis,
    /// and ensures that the qubit is returned to |0⟩
    /// following the measurement.
    ///
    /// # Input
    /// ## target
    /// A single qubit to be measured.
    ///
    /// # Output
    /// The result of measuring `target` in the Pauli Y basis.
    operation MResetY (target : Qubit) : Result {
        // Map the qubit's state from the Z-basis to the Y-basis.
        // Then measure and reset the qubit.
        // Note: this use HSadj instead fo HSH since that is sufficient for measurement.
        Adjoint S(target);
        H(target);
        MResetZ(target)
    }

    /// # Summary
    /// Measures a single qubit in the Z basis,
    /// and resets it to a fixed initial state
    /// following the measurement.
    ///
    /// # Description
    /// Performs a single-qubit measurement in the Z-basis,
    /// and ensures that the qubit is returned to |0⟩
    /// following the measurement.
    ///
    /// # Input
    /// ## target
    /// A single qubit to be measured.
    ///
    /// # Output
    /// The result of measuring `target` in the Pauli Z basis.
    operation MResetZ (target : Qubit) : Result {
        __quantum__qis__mresetz__body(target)
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
    @Config(Unrestricted)
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
