We remember from the Single-Qubit Gates kata that the Pauli Z gate leaves the sign of the $|0\rangle$ component of the single qubit superposition unchanged but flips the sign of the $|1\rangle$ component of the superposition. We have also just seen in "Bell State Change 2" how to change our input state to the state $\frac{1}{\sqrt{2}} \big(|01\rangle + |10\rangle\big)$, which is almost our goal state (disregarding the phase change for the moment). So it would seem that a combination of these two gates will be what we need here. The remaining question is in what order to apply them, and to which qubit.

First of all, which qubit? Looking back at the task "Bell state change 2", it seems clear that we need to use qubit `qs[0]`, like we did there.

Second, in what order should we apply the gates? Remember that the Pauli Z gate flips the phase of the $|1\rangle$ component of the superposition and leaves the $|0\rangle$ component alone.
Let's experiment with applying X to `qs[0]` first. Looking at our "halfway answer" state $\frac{1}{\sqrt{2}} \big(|01\rangle + |10\rangle\big)$, we can see that if we apply the Z gate to `qs[0]`, it will leave the $|0_{A}\rangle$ alone but flip the phase of $|1_{A}\rangle$ to $-|1_{A}\rangle$, thus flipping the phase of the $|11\rangle$ component of our Bell state.

@[solution]({
"id": "multi_qubit_systems__bell_state_change_3_solution",
"codePath": "./Solution.qs"
})
