We have seen in the Single-Qubit Gates kata that the Pauli X gate flips $|0\rangle$ to $|1\rangle$ and vice versa, and as we seem to need some flipping of states, perhaps this gate may be of use. (Bearing in mind, of course, that the X gate operates on a single qubit).

Let's compare the starting state $\frac{1}{\sqrt{2}} \big(|0_A0_B\rangle + |1_A1_B\rangle\big)$ with the goal state $\frac{1}{\sqrt{2}} \big(1_A0_B\rangle + |0_A1_B\rangle\big)$ term by term and see how we need to transform it to reach the goal.

Using our nomenclature from "Bell state change  1", we can now see by comparing terms that $|0_{A}\rangle$ has flipped to $|1_A\rangle$ to get the first term, and $|1_{A}\rangle$ has flipped to $|0_A\rangle$ to get the second term. This allows us to say that the correct gate to use is Pauli X, applied to `qs[0]`.

@[solution]({
"id": "multi_qubit_systems__bell_state_change_2_solution",
"codePath": "Solution.qs"
})
