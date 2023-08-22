
**Input:** 2 qubits in an unknown state $|\psi\rangle = \sum_{k = 0}^{3} x_k |k\rangle$. The amplitudes $x_k$ will be real and non-negative.

**Output:** A tuple of two numbers $(x_1', x_2')$ - your estimates of the amplitudes of the state $|1\rangle$ and $|2\rangle$, respectively.
The absolute errors $|x_1 - x_1'|$ and $|x_2 - x_2'|$ should be less than or equal to 0.001.

Please note that the state parameter is guaranteed to be the same
if you run the code several times. Your operation will be called
once for every run.

<details>
  <summary><b>Need a hint?</b></summary>
  On a physical quantum system, there would be no way to obtain these values from a single observation. Since this program runs on a simulator, we can use <code>DumpMachine</code> to inspect the qubits and take a note of their state. Furthermore, the problem statement guarantees, that the state will be the same from invocation to invocation. So we can update the code to return the amplitudes that we've taken note of. Then run the code again.
</details>
