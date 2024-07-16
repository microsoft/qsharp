In Q# the `Rx` intrinsic gate takes the angle $\theta$ and the target qubit as inputs. To create a controlled version of this gate, we can use the `Controlled` functor.

A matrix representation of this operation would be:

$$
\begin{bmatrix} 1 & 0 & 0 & 0 \\ 0 & 1 & 0 & 0 \\ 0 & 0 & \cos\frac{\theta}{2} & -i\sin\frac{\theta}{2} \\ 0 & 0 & -i\sin\frac{\theta}{2} &  \cos\frac{\theta}{2} \end{bmatrix}
$$

The parameters of the new gate are changed a bit:

* The first parameter has to be the array of control qubits; the `Rx` gate will be applied to the target only if these are all in the $\ket{1}$ state. Note that this parameter has to be an array, even if there is just one control qubit!
* The second parameter is a tuple with the parameters that you would've passed to the original `Rx` gate. You can create a tuple of values by putting round brackets around them.

> The `Controlled` functor can be used before any single-qubit gate to make it a controlled gate. The first argument will be an array of qubits even if you are using a single control qubit, like in the $CNOT$ gate. The second argument is a tuple `()` with the parameters of the gate. For example, these two gates are equivalent: `CNOT(qs[0], qs[1])` and `Controlled X([qs[0]], (qs[1]));`

@[solution]({
    "id": "multi_qubit_gates__controlled_rotation_solution",
    "codePath": "./Solution.qs"
})
