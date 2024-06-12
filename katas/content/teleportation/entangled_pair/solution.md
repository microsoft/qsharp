It is worth remembering the state we have already seen in "Multi-Qubit Gates" and "Preparing Quantum States" katas.

The solution can be divided in two steps:
- Apply a Hadamard gate to Alice's qubit: 
$\big(\frac{\ket{0} + \ket{1}}{\sqrt{2}}\big) \otimes \ket{0}$
- Apply a $CNOT$ gate with Alice's qubit as the control and Bob's qubit as the target. 

@[solution]({
    "id": "teleportation__entangled_pair_solution",
    "codePath": "./Solution.qs"
})
