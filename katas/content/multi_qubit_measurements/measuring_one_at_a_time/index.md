## Measuring each qubit in a system one after another
As described in the previous sections, in theory it is possible to measure all the qubits in an $n$-qubit system simultaneously in an orthogonal basis. The post-measurement state of the qubits is then exactly one of the $2^n$ possible basis states.

In practice, this is implemented by measuring all the qubits one after another. For example, if one wants to measure a two-qubit system in the computational basis, one can implement this by first measuring the first qubit in the computational basis to obtain $0$ or $1$, and then measuring the second qubit in the computational basis. This can result in one of the four possible outcomes: $00, 01, 10, 11$.

This can be generalized to measurements in other bases, such as the 2-qubit Pauli X basis $\ket{++}, \ket{+-}, \ket{-+}, \ket{--}$, and the bases for larger numbers of qubits.

> Note that measuring all qubits one after another can only be used to measure in orthogonal bases $\{ \ket{b_i}\}$ such that each $\ket{b_i}$ is a 'tensor product state'. That is, each $\ket{b_i}$ must be of the form $\ket{v_0} \otimes \ket{v_1} \dotsc \otimes \ket{v_{n-1}}$, with each $\ket{v_j}$ being a single-qubit basis state. 
For example, for the 2-qubit Pauli X basis $\ket{++}, \ket{+-}, \ket{-+}, \ket{--}$ each basis state is a tensor product of states $\ket{+}$ and $\ket{-}$, which form a single-qubit basis state.
> 
> Measuring in orthogonal bases which contain states which are not tensor product states, such as the Bell basis, are trickier to implement, and require appropriate unitary rotations in addition to measuring all qubits one after another. 
> We will not discuss such measurements in this tutorial.
> You can find examples of such measurements and their implementations in the [Measurements kata](../../Measurements/Measurements.ipynb).
>
> If we restrict ourselves to measurements in tensor product states, the distinction between measuring all the qubits simultaneously versus one after another is not important for an ideal quantum computer: in terms of the outcomes and measurement probabilities, both are identical. Furthermore, as long as all the qubits are measured, the sequence in which they are measured is also inconsequential. These factors can be  important in the case of real quantum computers with imperfect qubits, but we restrict the discussion to ideal systems in this tutorial.


### <span style="color:blue">Demo: Measurement statistics for  qubit-by-qubit full measurement</span>
This demo illustrates the equivalence of the measurement probabilities for simultaneous measurement on all qubits, and measurements on each of the qubits executed one after another. Using the wave function from exercise 1 above as an example, we show that the measurement probabilities obtained using the `M` operation in Q# are the same as those expected theoretically for exercise 1. 

The simulated probabilities will be different for each run of `DemoBasisMeasurement`. The simulated and theoretical probabilities are not expected to be identical, since measurements are probabilistic. However, we expect the values to be similar, and the simulated probabilities to approach the theoretical probabilities as the parameter `numRuns` is increased.
