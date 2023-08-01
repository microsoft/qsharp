# High-probability basis states

**Input:** $N$ qubits in an unknown state $|\psi\rangle = \sum_{k = 0}^{2^N-1} x_k |k\rangle$. The amplitudes $x_k$ will be real and non-negative.

**Output:** An array of integers which represent the basis states which have amplitudes bigger than 0.01 (in little endian encoding). The integers can be returned in any order.

The test will call your code exactly once, with the fixed state parameter that will not change if you run the cell several times.

