You have seen in the Single-Qubit Gates kata that the Pauli X gate flips $\ket{0}$ to $\ket{1}$ and vice versa, and as you seem to need some flipping of states, perhaps this gate may be of use. (Bearing in mind, of course, that the $X$ gate operates on a single qubit).

Let's compare the starting state $\frac{1}{\sqrt{2}} \big(\ket{0_A0_B} + \ket{1_A1_B}\big)$ with the goal state $\frac{1}{\sqrt{2}} \big(\ket{1_A0_B} + \ket{0_A1_B}\big)$ term by term and see how you need to transform it to reach the goal.

Using the nomenclature from "Bell state change  1", you can now see by comparing terms that $\ket{0_{A}}$ has flipped to $\ket{1_A}$ to get the first term, and $\ket{1_{A}}$ has flipped to $\ket{0_A}$ to get the second term. This allows you to say that the correct gate to use is Pauli X, applied to `qs[0]`.

@[solution]({
"id": "multi_qubit_systems__bell_state_change_2_solution",
"codePath": "Solution.qs"
})
