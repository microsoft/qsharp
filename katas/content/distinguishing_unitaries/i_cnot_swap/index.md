**Input:** An operation that implements a two-qubit unitary transformation:
either the identity ($I \otimes I$), the $CNOT$ gate with one of the qubits as control and the other qubit as a target, or the $SWAP$ gate.

**Output:**  

* 0 if the given operation is $I \otimes I$, 
* 1 if the given operation is $CNOT_{12}$,
* 2 if the given operation is $CNOT_{21}$,
* 3 if the given operation is $SWAP$.

The operation will accept an array of qubits as input, but it will fail if the array is empty or has one or more than two qubits.
The operation will have Adjoint and Controlled variants defined.
You are allowed to apply the given operation and its adjoint/controlled variants at most **twice**.