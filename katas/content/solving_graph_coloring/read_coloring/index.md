In this task, you need to read the coloring of an $V$-vertex graph stored in a qubit array.

**Inputs**:

  1. The number of bits in each color of the coloring $nBits$.
  2. An array of $V \cdot nBits$ qubits which are guaranteed to be in a basis state.

**Output**:

An array of $V$ $nBits$-bit integers that represent this basis state.
$i$-th integer of the array is stored in qubits with indices $i \cdot nBits$, $i \cdot nBits + 1$, ..., $(i + 1) \cdot nBits - 1$ in big-endian format. 
The operation should not change the state of the qubits.
