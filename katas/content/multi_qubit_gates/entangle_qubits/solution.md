Let's denote the first qubit in state $\alpha \ket{0} + \beta \ket{1}$ as A and the second qubit in state $\ket{0}$ as B.

Compare the input state $\alpha \ket{0_A0_B} + \beta \ket{1_A0_B}$ with the goal state $\alpha \ket{0_A0_B} + \beta \ket{1_A1_B}$. 
You want to pass our input qubit through a gate or gates (to be decided) that do the following. If qubit A is in the $\ket{0}$ state, then you want to leave qubit B alone (the first term of the superposition). 
However, if A is in the $\ket{1}$ state, you want to flip qubit B from $\ket{0}$ into $\ket{1}$ state. In other words, the state of B is to be made contingent upon the state of A. 
This is exactly the effect of the $CNOT$ gate. Depending upon the state of the **control** qubit (A in your case), the value of the controlled or **target** qubit (B in your case) is inverted or unchanged. Thus, you get the goal state $\alpha \ket{00} + \beta \ket{11}$.  

@[solution]({
    "id": "multi_qubit_gates__entangle_qubits_solution",
    "codePath": "./Solution.qs"
})
