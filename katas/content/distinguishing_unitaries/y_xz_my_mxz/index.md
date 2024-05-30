**Input:** An operation that implements a single-qubit unitary transformation:
either the $Y$ gate (possibly with an extra global phase of $-1$) or the sequence of Pauli $Z$ and Pauli $X$ gates (possibly with an extra global phase of $-1$).

**Output:** 
* 0 if the given operation is the $Y$ gate,
* 1 if the given operation is the $-XZ$ gate,
* 2 if the given operation is the $-Y$ gate,
* 3 if the given operation is the $XZ$ gate.

The operation will have Adjoint and Controlled variants defined.
You are allowed to apply the given operation and its adjoint/controlled variants at most **three times**.