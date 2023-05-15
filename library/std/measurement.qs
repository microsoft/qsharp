// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

namespace Microsoft.Quantum.Measurement {
    open Microsoft.Quantum.Core;
    open Microsoft.Quantum.Intrinsic;
    open QIR.Intrinsic;

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
        let len = Length(register);
        mutable results = [Zero, size = len];
        for i in 0..(len - 1) {
            set results w/= i <- M(register[i]);
        }
        results
    }

    /// # Summary
    /// Measures a single qubit in the X basis,
    /// and resets it to a fixed initial state
    /// following the measurement.
    ///
    /// # Description
    /// Performs a single-qubit measurement in the $X$-basis,
    /// and ensures that the qubit is returned to $\ket{0}$
    /// following the measurement.
    ///
    /// # Input
    /// ## target
    /// A single qubit to be measured.
    ///
    /// # Output
    /// The result of measuring `target` in the Pauli $X$ basis.
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
    /// Performs a single-qubit measurement in the $Y$-basis,
    /// and ensures that the qubit is returned to $\ket{0}$
    /// following the measurement.
    ///
    /// # Input
    /// ## target
    /// A single qubit to be measured.
    ///
    /// # Output
    /// The result of measuring `target` in the Pauli $Y$ basis.
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
    /// Performs a single-qubit measurement in the $Z$-basis,
    /// and ensures that the qubit is returned to $\ket{0}$
    /// following the measurement.
    ///
    /// # Input
    /// ## target
    /// A single qubit to be measured.
    ///
    /// # Output
    /// The result of measuring `target` in the Pauli $Z$ basis.
    operation MResetZ (target : Qubit) : Result {
        __quantum__qis__mresetz__body(target)
    }
}