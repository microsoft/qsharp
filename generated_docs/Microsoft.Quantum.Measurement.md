# operation MeasureAllZ(register : Qubit[]) : Result

## Summary
Jointly measures a register of qubits in the Pauli Z basis.

## Description
Measures a register of qubits in the `Z ⊗ Z ⊗ ••• ⊗ Z`
basis, representing the parity of the entire register.

## Input
### register
The register to be measured.

## Output
The result of measuring `Z ⊗ Z ⊗ ••• ⊗ Z`.

## Remarks
This operation does not reset the measured qubits to the |0⟩ state,
leaving them in the state that corresponds to the measurement result.

&nbsp;

---

&nbsp;

# operation MeasureEachZ(register : Qubit[]) : Result[]

## Summary
Measures each qubit in a given array in the standard basis.
## Input
### targets
An array of qubits to be measured.
## Output
An array of measurement results.

## Remarks
This operation does not reset the measured qubits to the |0⟩ state,
leaving them in the state that corresponds to the measurement results.

&nbsp;

---

&nbsp;

# operation MResetEachZ(register : Qubit[]) : Result[]

## Summary
Measures each qubit in a given array in the Z basis
and resets them to a fixed initial state.
## Input
### targets
An array of qubits to be measured.
## Output
An array of measurement results.

&nbsp;

---

&nbsp;

# operation MResetX(target : Qubit) : Result

## Summary
Measures a single qubit in the X basis,
and resets it to a fixed initial state
following the measurement.

## Description
Performs a single-qubit measurement in the X-basis,
and ensures that the qubit is returned to |0⟩
following the measurement.

## Input
### target
A single qubit to be measured.

## Output
The result of measuring `target` in the Pauli X basis.

&nbsp;

---

&nbsp;

# operation MResetY(target : Qubit) : Result

## Summary
Measures a single qubit in the Y basis,
and resets it to a fixed initial state
following the measurement.

## Description
Performs a single-qubit measurement in the Y-basis,
and ensures that the qubit is returned to |0⟩
following the measurement.

## Input
### target
A single qubit to be measured.

## Output
The result of measuring `target` in the Pauli Y basis.

&nbsp;

---

&nbsp;

# operation MResetZ(target : Qubit) : Result

## Summary
Measures a single qubit in the Z basis,
and resets it to a fixed initial state
following the measurement.

## Description
Performs a single-qubit measurement in the Z-basis,
and ensures that the qubit is returned to |0⟩
following the measurement.

## Input
### target
A single qubit to be measured.

## Output
The result of measuring `target` in the Pauli Z basis.

&nbsp;

---

&nbsp;

# operation MeasureInteger(target : Qubit[]) : Int

## Summary
Measures the content of a quantum register and converts
it to an integer. The measurement is performed with respect
to the standard computational basis, i.e., the eigenbasis of `PauliZ`.

## Input
### target
A quantum register in the little-endian encoding.

## Output
An unsigned integer that contains the measured value of `target`.

## Remarks
This operation resets its input register to the |00...0> state,
suitable for releasing back to a target machine.
