**Input:** An operation that implements a single-qubit unitary transformation:
either the $Z$ gate or the minus $Z$ gate (i.e., the gate $-\ket{0}\bra{0} + \ket{1}\bra{1}$). 

**Output:**  0 if the given operation is the $Z$ gate, 1 if the given operation is the $-Z$ gate.

The operation will have Adjoint and Controlled variants defined.
You are allowed to apply the given operation and its adjoint/controlled variants exactly **once**.