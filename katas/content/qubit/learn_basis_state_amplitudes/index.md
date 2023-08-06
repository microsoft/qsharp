
**Input:** 2 qubits in an unknown state $|\psi\rangle = \sum_{k = 0}^{3} x_k |k\rangle$. The amplitudes $x_k$ will be real and non-negative.

**Output:** A tuple of two numbers $(x_1', x_2')$ - your estimates of the amplitudes of the state $|1\rangle$ and $|2\rangle$, respectively.
The absolute errors $|x_1 - x_1'|$ and $|x_2 - x_2'|$ should be less than or equal to 0.001.

The test will call your code exactly once, with the fixed state parameter that will not change if you run the cell several times.
