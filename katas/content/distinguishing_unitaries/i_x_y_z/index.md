**Input:** An operation that implements a single-qubit unitary transformation:
either the identity ($I$ gate) or one of the Pauli gates ($X$, $Y$ or $Z$ gate).

**Output:** 
* 0 if the given operation is the $I$ gate,
* 1 if the given operation is the $X$ gate,
* 2 if the given operation is the $Y$ gate,
* 3 if the given operation is the $Z$ gate.

The operation will have Adjoint and Controlled variants defined.
You are allowed to apply the given operation and its adjoint/controlled variants exactly **once**.