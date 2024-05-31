**Input:** An operation that implements a single-qubit unitary transformation:
either the [$R_z$ gate](https://learn.microsoft.com/qsharp/api/qsharp-lang/microsoft.quantum.intrinsic/rz) or the [$R1$ gate](https://learn.microsoft.com/qsharp/api/qsharp-lang/microsoft.quantum.intrinsic/r1). 

This operation will take two parameters: the first parameter is the rotation angle, in radians, and the second parameter is the qubit to which the gate should be applied (matching normal `Rz` and `R1` gates in Q#).

> As a reminder, $$R_z(\theta) = \begin{bmatrix} e^{-i\theta/2} & 0 \\ 0 & e^{i\theta/2} \end{bmatrix} \text{, } 
R_1(\theta) = \begin{bmatrix} 1 & 0 \\ 0 & e^{i\theta} \end{bmatrix} = e^{i\theta/2} R_z(\theta)$$

**Output:**  0 if the given operation is the $R_z$ gate, 1 if the given operation is the $R1$ gate.

The operation will have Adjoint and Controlled variants defined.
You are allowed to apply the given operation and its adjoint/controlled variants exactly **once**.