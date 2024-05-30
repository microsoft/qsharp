**Input:** An operation that implements a single-qubit unitary transformation:
either the [$R_z$ gate](https://learn.microsoft.com/qsharp/api/qsharp-lang/microsoft.quantum.intrinsic/rz) or the [$R1$ gate](https://learn.microsoft.com/qsharp/api/qsharp-lang/microsoft.quantum.intrinsic/r1). 

This operation will take two parameters: the first parameter is the rotation angle, in radians, and the second parameter is the qubit to which the gate should be applied (matching normal `Rz` and `R1` gates in Q#).

**Output:**  0 if the given operation is the $R_z$ gate, 1 if the given operation is the $R1$ gate.

The operation will have Adjoint and Controlled variants defined.
You are allowed to apply the given operation and its adjoint/controlled variants exactly **once**.