**Input:** An operation that implements a two-qubit unitary transformation:
either the $I \otimes X$ (the $X$ gate applied to the second qubit) or the $CNOT$ gate with the first qubit as control and the second qubit as target.

**Output:**  0 if the given operation is $I \otimes X$, 1 if the given operation is the $CNOT$ gate.

The operation will accept an array of qubits as input, but it will fail if the array is empty or has one or more than two qubits.
The operation will have Adjoint and Controlled variants defined.
You are allowed to apply the given operation and its adjoint/controlled variants exactly **once**.