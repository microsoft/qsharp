
**Input:** A qubit in an unknown state $|\psi\rangle = \alpha|0\rangle + \beta|1\rangle$. The amplitudes $\alpha$ and $\beta$ will be real and non-negative.

**Output:** A tuple of two numbers $(\alpha', \beta')$ - your estimates of the amplitudes $\alpha$ and $\beta$.
The absolute errors $|\alpha - \alpha'|$ and $|\beta - \beta'|$ should be less than or equal to 0.001.

Please note that the state parameter is guaranteed to be the same
if you run the code several times. Your operation will be called
once for every run.

<details>
  <summary><b>Need a hint?</b></summary>
  On a physical quantum system, there would be no way to obtain the values of $\alpha$ and $\beta$ from a single observation. Since this program runs on a simulator, we can use <code>DumpMachine</code> to inspect the qubit and take a note of its state. Furthermore, the problem statement guarantees, that the state will be the same from invocation to invocation. So we can update the code to return the amplitudes that we've taken note of. Then run the code again.
</details>
