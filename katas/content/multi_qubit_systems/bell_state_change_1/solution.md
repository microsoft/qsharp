You recognize that the goal is another Bell state. In fact, it is one of the four Bell states.

You have seen in the Single-Qubit Gates kata that the Pauli Z gate will change the state of the $\ket{1}$ basis state of a single qubit, so this gate seems like a good candidate for what you want to achieve. This gate leaves the sign of the $\ket{0}$ basis state of a superposition unchanged, but flips the sign of the $\ket{1}$ basis state of the superposition.

Don't forget that the $Z$ gate acts on only a single qubit, and you have two here.
Let's also remember how the Bell state is made up from its individual qubits.

If the two qubits are A and B, where A is `qs[0]` and B is `qs[1]`, you can write that
$\ket{\Phi^{+}} = \frac{1}{\sqrt{2}} \big(\ket{0_{A}0_{B}} + \ket{1_{A}1_{B}}\big)$.
If you apply the $Z$ gate to the qubit A, it will flip the phase of the basis state $\ket{1_A}$. As this phase is in a sense spread across the entangled state, with $\ket{1_A}$ basis state being part of the second half of the superposition, this application has the effect of flipping the sign of the whole basis state $\ket{1_A1_B}$, as you can see by running the solution below.

The exact same calculations can be done if you apply $Z$ to the qubit B, so that's another possible solution.
@[solution]({
"id": "multi_qubit_systems__bell_state_change_1_solution",
"codePath": "Solution.qs"
})
