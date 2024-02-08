# MeasureAllZ operation

`operation MeasureAllZ(register : Qubit[]) : Result`

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
