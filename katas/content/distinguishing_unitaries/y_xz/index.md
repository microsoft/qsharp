**Input:** An operation that implements a single-qubit unitary transformation:
either the $Y$ gate or the sequence of Pauli $Z$ and Pauli $X$ gates (equivalent to applying the $Z$ gate followed by the $X$ gate).

**Output:**  0 if the given operation is the $Y$ gate, 1 if the given operation is the $XZ$ gate.

The operation will have Adjoint and Controlled variants defined.
You are allowed to apply the given operation and its adjoint/controlled variants at most **twice**.