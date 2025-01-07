// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.


import Std.Core.*;
import Std.Intrinsic.*;
import Std.Diagnostics.*;
open QIR.Intrinsic;

/// # Summary
/// Jointly measures a register of qubits in the Pauli Z basis.
///
/// # Description
/// Measures a register of qubits in the `Z ⊗ Z ⊗ ••• ⊗ Z`
/// basis, representing the parity of the entire register.
/// This operation does not reset the measured qubits to the |0⟩ state,
/// leaving them in the state that corresponds to the measurement result.
///
/// # Input
/// ## register
/// The register to be jointly measured.
///
/// # Output
/// The result of measuring in the `Z ⊗ Z ⊗ ••• ⊗ Z` basis.
///
/// # See also
/// - [Std.Measurement.MeasureEachZ](xref:Qdk.Std.Measurement.MeasureEachZ)
operation MeasureAllZ(register : Qubit[]) : Result {
    Measure(Repeated(PauliZ, Length(register)), register)
}

/// # Summary
/// Measures each qubit in a given array in the Pauli Z basis.
///
/// # Description
/// Measures each qubit in a register in the `Z` basis
/// and retuns the result of each measurement.
/// This operation does not reset the measured qubits to the |0⟩ state,
/// leaving them in the state that corresponds to the measurement results.
///
/// # Input
/// ## targets
/// An array of qubits to be measured.
/// # Output
/// An array of measurement results.
///
/// # Remarks
/// Please note the following differences:
/// - Operation `MeasureEachZ` performs one measurement for each qubit and retuns
///   an array of results. The operation does not reset the qubits.
/// - Operation `MResetEachZ` performs one measurement for each qubit and retuns
///   an array of results. The operation resets all qubits to |0⟩ state.
/// - Operation `MeasureAllZ` performs a joint measurement on all qubits
///   and returns one result. The operation does not reset the qubits.
///
/// # See also
/// - [Std.Measurement.MeasureAllZ](xref:Qdk.Std.Measurement.MeasureAllZ)
/// - [Std.Measurement.MResetEachZ](xref:Qdk.Std.Measurement.MResetEachZ)
operation MeasureEachZ(register : Qubit[]) : Result[] {
    mutable results = [];
    for qubit in register {
        set results += [M(qubit)];
    }
    results
}

/// # Summary
/// Measures each qubit in a given array in the Pauli Z basis
/// and resets them to |0⟩ state.
///
/// # Input
/// ## targets
/// An array of qubits to be measured.
///
/// # Output
/// An array of measurement results.
///
/// # See also
/// - [Std.Measurement.MeasureEachZ](xref:Qdk.Std.Measurement.MeasureEachZ)
operation MResetEachZ(register : Qubit[]) : Result[] {
    mutable results = [];
    for qubit in register {
        set results += [MResetZ(qubit)];
    }
    results
}

/// # Summary
/// Performs a single-qubit measurement in the Pauli X basis,
/// and resets `target` to the |0⟩ state after the measurement.
///
/// # Input
/// ## target
/// A single qubit to be measured.
///
/// # Output
/// The result of measuring `target` in the Pauli X basis.
operation MResetX(target : Qubit) : Result {
    // Map the qubit's state from the Z-basis to the X-basis.
    // Then measure and reset the qubit.
    H(target);
    MResetZ(target)
}

/// # Summary
/// Performs a single-qubit measurement in the Pauli Y basis,
/// and resets `target` to the |0⟩ state after the measurement.
///
/// # Input
/// ## target
/// A single qubit to be measured.
///
/// # Output
/// The result of measuring `target` in the Pauli Y basis.
operation MResetY(target : Qubit) : Result {
    // Map the qubit's state from the Z-basis to the Y-basis.
    // Then measure and reset the qubit.
    // Note: this use HSadj instead of HSH since that is sufficient for measurement.
    Adjoint S(target);
    H(target);
    MResetZ(target)
}

/// # Summary
/// Performs a single-qubit measurement in the Pauli Z basis,
/// and resets `target` to the |0⟩ state after the measurement.
///
/// # Input
/// ## target
/// A single qubit to be measured.
///
/// # Output
/// The result of measuring `target` in the Pauli Z basis.
operation MResetZ(target : Qubit) : Result {
    __quantum__qis__mresetz__body(target)
}

/// # Summary
/// Measures the content of a quantum register and converts it to an integer.
/// The measurement is performed with respect to the standard computational basis,
/// i.e., the eigenbasis of `PauliZ`. Input register is reset to the |00...0⟩ state,
/// which is suitable for releasing the register back to a target machine.
///
/// # Input
/// ## target
/// A quantum register in the little-endian encoding.
///
/// # Output
/// An unsigned integer that contains the measured value of `target`.
operation MeasureInteger(target : Qubit[]) : Int {
    let nBits = Length(target);
    Fact(nBits < 64, $"`Length(target)` must be less than 64, but was {nBits}.");

    mutable number = 0;
    for i in 0..nBits - 1 {
        if (MResetZ(target[i]) == One) {
            set number |||= 1 <<< i;
        }
    }

    number
}
export MeasureAllZ, MeasureEachZ, MResetEachZ, MResetX, MResetY, MResetZ, MeasureInteger;

