Here you start with the end state of the previous task $\frac{1}{2} \big(\ket{00} + \ket{01} + \ket{10} + \ket{11}\big)$. Looking at the desired state, the phase of the $\ket{11}$ state is flipped ($+$ changed to a $-$).

A regular phase flip on one qubit can be done using a $Z$ gate:
$$\begin{bmatrix} 1 & 0 \\ 0 & -1 \end{bmatrix}$$
This gate will perform a phase flip only on the $\ket{1}$ state:

$$Z(\alpha\ket{0} + \beta\ket{1}) = \alpha\ket{0} - \beta\ket{1}$$

In this case, you only want to flip the phase of the $\ket{11}$ state and not the $\ket{01}$ state. To accomplish this, you can use a controlled $Z$ gate; this will make sure that the $Z$ gate is only applied if the control bit is in the $\ket{1}$ state, and the $\ket{01}$ state won't change.

> In Q#, you can apply a controlled gate by using the `Controlled` keyword before the gate. The controlled gate will take two parameters; the first parameter is an array of control qubits (you can have multiple qubits used as a control), and the second parameter is a tuple of parameters passed to the original gate (in this case it's just the qubit to which you want to apply the gate if the control bit is $\ket{1}$).

@[solution]({
    "id": "preparing_states__even_sup_two_qubits_phase_flip_solution",
    "codePath": "./Solution.qs"
})
