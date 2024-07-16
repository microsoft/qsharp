Let's find a unitary transformation that converts the state $\ket{S_0}$ to the basis state $\ket{000}$. To do this, we first apply a unitary operation that maps the first state to the W state 
$\frac{1}{\sqrt{3}} \big(\ket{100} + \ket{010} + \ket{001}\big)$.

After this, we will use a convenient rotation gate $R_1$ which applies a relative phase to the $\ket{1}$ state and doesn't change the $\ket{0}$ state.
In matrix form: 

$$R_1(\theta) = \begin{bmatrix} 1 & 0 \\ 0 & e^{i\theta} \end{bmatrix} $$

This can be accomplished by a tensor product 
$I \otimes R_1(-\frac{2\pi}{3}) \otimes R_1(-\frac{4\pi}{3})$, where

* $I$ is the identity gate applied to qubit 0,
* $R_1(-\frac{2\pi}{3}) = \begin{bmatrix} 1 & 0 \\ 0 & \omega^{-1} \end{bmatrix}$, applied to qubit 1,
* $R_1(-\frac{4\pi}{3}) = \begin{bmatrix} 1 & 0 \\ 0 & \omega^{-2} \end{bmatrix}$, applied to qubit 2.

> Note that applying this operation to the state $\ket{S_1}$ converts it to $\frac{1}{\sqrt{3}} \big (\ket{100} + \omega \ket{010} + \omega^2 \ket{001} \big)$.

Now we can use adjoint of the state preparation routine for W state (from task "W State on Arbitrary Number of Qubits" of the Preparing Quantum States kata), which will map the W state to the state $\ket{000}$ and the second state to some other state $\ket{S'_1}$.

We don't need to do the math to figure out the exact state $\ket{S'_1}$ in which $\ket{S_1}$ will end up after those two transformations. Remember that our transformations are unitary, i.e., they preserve the inner products of vectors. Since the states $\ket{S_0}$ and $\ket{S_1}$ were orthogonal, their inner product $\braket{S_0|S_1} = 0$ is preserved when applying unitary transformations, and the states after the transformation will remain orthogonal.

The state $\ket{S'_1}$ is guaranteed to be orthogonal to the state $\ket{000}$, i.e., $\ket{S_1}$ gets mapped to a superposition that does not include basis state $\ket{000}$. To distinguish the states $\ket{000}$ and $\ket{S'_1}$, we measure all qubits; if all measurement results were 0, the state was $\ket{000}$ and we return 0, otherwise we return 1.

@[solution]({
    "id": "distinguishing_states__two_orthogonal_three_qubit_solution",
    "codePath": "Solution.qs"
})
