Let's denote the first qubit in state $\alpha |0\rangle + \beta |1\rangle$ as A and the second qubit in state $|0\rangle$ as B.

Compare our input state $\alpha |0_A0_B\rangle + \beta |1_A0_B\rangle$ with the goal state $\alpha |0_A0_B\rangle + \beta |1_A1_B\rangle$. 
We want to pass our input qubit through a gate or gates (to be decided) that do the following. If qubit A is in the $|0\rangle$ state, then we want to leave qubit B alone (the first term of the superposition). 
However, if A is in the $|1\rangle$ state, we want to flip qubit B from $|0\rangle$ into $|1\rangle$ state. In other words, the state of B is to be made contingent upon the state of A. 
This gate exists and is called a CNOT (controlled-not) gate. Depending upon the state of the **control** qubit (A in our case), the value of the controlled or **target** qubit (B in our case) is inverted or unchanged.  Thus we get the goal state $\alpha |00\rangle + \beta |11\rangle$.  

@[solution]({
    "id": "multi_qubit_gates__two_qubit_gate_1_solution",
    "codePath": "./Solution.qs"
})
