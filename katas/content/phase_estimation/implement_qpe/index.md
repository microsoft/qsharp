**Inputs:**

1. A single-qubit unitary $U$. The unitary will have its controlled variant defined.
2. A single-qubit state $\ket{\psi}$, given as a unitary $P$ that prepares it from the $\ket{0}$ state. In other words, the result of applying the unitary $P$ to the state $\ket{0}$ is the $\ket{\psi}$ state:
   $$P\ket{0} = \ket{\psi}$$
3. A positive integer $n$.

**Output:** Return the eigenphase $\theta$ which corresponds to the eigenstate $\ket{\psi}$, multiplied by $2^n$. You are guaranteed that this eigenvalue has at most $n$ digits of binary precision, so multiplying it by $2^n$ yields an integer.
