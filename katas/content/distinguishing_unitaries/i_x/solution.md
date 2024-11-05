The only way to extract information out of a quantum system is measurement. 
Measurements give us information about the states of a system, so to get information about the gate, we need to find a way to convert it into information about a state.
If we want to distinguish two gates, we need to figure out to prepare a state and perform a measurement on it that will give us a result that we can interpret.
To do this, we'll need to find a qubit state that, by applying to it $I$ gate or $X$ gate, will be transformed into states that can be distinguished using measurement, i.e., orthogonal states. 
Let's find such state.

> As a reminder, here are the matrices that correspond to the given gates:
> $$I = \begin{bmatrix} 1 & 0 \\ 0 & 1 \end{bmatrix}, X = \begin{bmatrix} 0 & 1 \\ 1 & 0 \end{bmatrix}$$

Consider the effects of these gates on the basis state $\ket{0}$.

$$I\ket{0} = \ket{0}$$
$$X\ket{0} = \ket{1}$$

We see that the $I$ gate leaves the $\ket{0}$ state unchanged, and the $X$ gate transforms it into the $\ket{1}$ state. 
So the easiest thing to do is to prepare a $\ket{0}$ state, apply the given unitary to it, and measure the resulting state in the computational basis:
* If the measurement result is `Zero`, the measured state was $\ket{0}$, and we know the unitary applied to it was the $I$ gate.
* If the measurement result is `One`, the measured state was $\ket{1}$, and we know the unitary applied to it was the $X$ gate.

> In Q#, the freshly allocated qubits start in the $\ket{0}$ state, so you don't need to do anything to prepare the necessary state before applying the unitary to it.
> You also have to return the qubits you allocated to the $\ket{0}$ state before releasing them. 
> You can do that by measuring the qubit using the `M` operation and applying the $X$ gate if it was measured in the $\ket{1}$ state, or you can use [`MResetZ`](https://learn.microsoft.com/qsharp/api/qsharp-lang/microsoft.quantum.measurement/mresetz) operation that wraps this measurement and qubit reset into one operation.

@[solution]({
    "id": "distinguishing_unitaries__i_x_solution",
    "codePath": "Solution.qs"
})
