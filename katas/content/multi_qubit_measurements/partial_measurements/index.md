## Partial Measurements

For a system with $n>1$ qubits, it is possible to measure $m<n$ qubits one after another. The number of measurement outcomes is then $2^m$ instead of $2^n$. The probabilities of each of the outcomes and the post-measurement states of the qubits can be found using the projection formalism for measurements.

First, we recall the concept of [projection operators](../SingleQubitSystemMeasurements/SingleQubitSystemMeasurements.ipynb#Measurements-as-projection-operations) introduced in the single-qubit systems measurements tutorial. Measurements are modeled by orthogonal projection operators - matrices that satisfy
$$
P^2 = P^\dagger = P.
$$
Consider an $n$-qubit system in a state $|\psi\rangle$, for which the first $m<n$ qubits are measured in an orthogonal basis $\{ |b_0\rangle , |b_1\rangle, \dotsc, |b_{2^m-1}\rangle\}$ corresponding to the $m$ qubits being measured. Then we define $2^m$ projectors corresponding to each of the $|b_i\rangle$ states as
$$
P_i = |b_i\rangle \langle b_i| \otimes \mathbb{1}_{n-m},
$$
where $\mathbb{1}_{n-m}$ is the identity operator over the remaining $(n-m)$ qubits. 
> The symbol $\otimes$ represents the tensor product or the Kronecker product of two matrices. It is different from the usual matrix multiplication (see the [Linear Algebra tutorial](../LinearAlgebra/LinearAlgebra.ipynb#Tensor-Product) for a refresher). In the current context, $|b_i\rangle \langle b_i| \otimes \mathbb{1}_{n-m}$ simply means that the operator $|b_i\rangle \langle b_i|$ acts only on the $m$ qubits being measured, while the effect of $P_i$ on the remaining qubits is $\mathbb{1}_{n-m}$, i.e., the identity operator. 

Analogous to the case for measurements for single-qubit systems, the rules for partial measurement probabilities and outcomes can be summarized as follows:
- When a measurement is done, one of these projectors is chosen randomly. The probability of choosing projector $P_i$ is $\big|P_i|\psi\rangle\big|^2$.
- If the projector $P_i$ is chosen, the measurement outcome is $b_i$, and the state of the system after the measurement is given by
$$
\frac{P_i |\psi\rangle}{\big|P_i |\psi\rangle\big|}.
$$

For example, consider a two-qubit system in the state $\ket \psi = \frac{1}{\sqrt{2}}\ket{01} - \frac{1}{\sqrt 2}\ket{10}$. Consider a measurement of the first qubit in the computational basis, i.e., in the $\{\ket 0 , \ket 1 \}$ basis. Then, we have two projectors that represent this measurement:
\begin{align*}
P_0 &= \ket 0\bra 0 \otimes \mathbb{1},\\
P_1 &= \ket 1 \bra 1 \otimes \mathbb{1}.
\end{align*}

The action of $P_0$ on $\ket \psi$ is 
\begin{align*}
P_0 \ket \psi &= \left(\ket 0\bra 0 \otimes \mathbb{1}\right) \frac{1}{\sqrt 2}\big(\ket{01} - \ket{10}\big) = \\
              &= \frac{1}{\sqrt 2} \big( \ket 0\bra 0 0\rangle \otimes \mathbb{1} \ket{1} - \ket 0 \bra 0 1\rangle \otimes \mathbb{1} \ket 0 \big) = \\
              &= \frac{1}{\sqrt 2} \ket{01}.
\end{align*}

Similarly, we obtain 
$$
P_1 \ket\psi = -\frac{1}{\sqrt 2} \ket{10}.
$$

Clearly, we have $\big|P_0 \ket \psi\big| = \big|P_1 \ket \psi\big| = \frac{1}{2}$ in this case. Thus, the probabilities of measuring $0$ and $1$ are both $0.5$, with the post-measurement states of system being $\ket{01}$ and $\ket{10}$, respectively.

> Similar to the case of single-qubit system measurements, the applicability of the formalism above requires the state of the multi-qubit system, $\ket \psi$, to be normalized. This is required to ensure that all the probabilities of individual outcomes add up to 1.

### <span style="color:blue">Exercise 4</span>: Partial measurement probabilities for the Hardy state

Consider a 2-qubit system in the state $\ket \psi = \frac{1}{\sqrt{12}} \big(3|00\rangle + |01\rangle + |10\rangle + |11\rangle\big)$.

If only the first qubit is measured in the computational basis, what are the probabilities of the outcomes, and the post-measurement states of the system?

### <span style="color:blue">Demo: Measurement statistics for  partial measurement</span>
Using the `M` operation in Q#, we demonstrate that the simulated outcome probabilities and post-measurement outcomes match the theoretical values obtained using the projection operators as described above. We use the Hardy state from Exercise 4 with a computational basis measurement on the first qubit for this purpose.

The simulated and theoretical measurement probabilities are not expected to match exactly, but should be close to each other, since measurement is probabilistic. However, the post-measurement states from the simulation should match the expected states for Exercise 4 precisely, since partial state collapse is not a probabilistic process.
