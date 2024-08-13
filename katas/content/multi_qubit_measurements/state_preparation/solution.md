While it's possible to prepare the state $\ket \psi$ directly using unitary rotations, it's simpler to use post-selection for preparing it.

Initially, you prepare an equal superposition of all basis states on the first two qubits by applying the $H$ gate to each of them, and allocate an extra qubit in the $\ket{0}$ state:
$$\frac{1}{2} \big(\ket{00} + \ket{01} + \ket{10} + \ket{11}\big) \otimes \ket 0$$

The state of the first two qubits is a superposition of the state you want to prepare and the $\ket{11}$ state that you want to discard.

Now, you want to separate the first three basis states from the last one and to store this separation in the extra qubit.
For example, you can keep the state of the extra qubit $\ket{0}$ for the basis states that you want to keep, and switch it to $\ket{1}$ for the basis states that you want to discard.
A $CCNOT$ gate can do this, with the first two qubits used as control qubits and the extra qubit as target.
When the gate is applied, the state of the extra qubit will only change to $\ket{1}$ if both control qubits are in the $\ket{11}$ state, which marks exactly the state that you want to discard:

$$CCNOT\frac{1}{2} \big(\ket{00\textbf{0}} + \ket{01\textbf{0}} + \ket{10\textbf{0}} + \ket{11\textbf{0}}\big) =
\frac{1}{2}\big(\ket{00} + \ket{01} + \ket{10} \big) \otimes \ket{\textbf{0}} + \frac{1}{2}\ket{11} \otimes \ket{\textbf{1}} $$

Finally, you measure just the extra qubit; this causes a partial collapse of the system to the state defined by the measurement result:
* If the result is $\ket{0}$, the first two qubits collapse to a state that is a linear combination of basis states which had the extra qubit in state $\ket{0}$, that is, they end up in the target state $\frac{1}{\sqrt3}\big(\ket{00} + \ket{01} + \ket{10}\big)$.
* If the result is $\ket{1}$, the first two qubits collapse to a state $\ket{11}$, so your goal is not achieved. The good thing is, this only happens in 25% of the cases, and you can just reset our qubits to the $\ket{00}$ state and try again.

> Q# has a built-in <a href="https://learn.microsoft.com/azure/quantum/user-guide/language/expressions/conditionalloops#repeat-expression" target="_blank">repeat-until-success (RUS) loop</a>, which comes in handy in this case.
> * You'll describe the main operations (applying $H$ and $CCNOT$ gates and the measurement) in the `repeat` part of the loop, which specifies its body.  
> * `until` section specifies the condition which will break the loop. In this case, the result of the measurement needs to be `Zero` to indicate your success.  
> * Finally, the `fixup` section allows you to clean up the results of the loop body execution before trying again if the success criteria isn't met. In this case, you reset the first two qubits back to the $\ket{00}$ state.

@[solution]({
    "id": "multi_qubit_measurements__state_preparation_solution",
    "codePath": "Solution.qs"
})
