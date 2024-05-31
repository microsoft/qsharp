Entanglement between three entities can also be achieved using $CNOT$ and Hadamard gate:
- Apply Hadamard to $qBob$
- Then perform $CNOT$ from $qBob$ to $qMessage$
- Apply Hadamard on $qAlice$ qubit now
- Again perform $CNOT$ but this time from $qAlice$ to $qMessage$

@[solution]({
    "id": "teleportation__entangled_trio_solution",
    "codePath": "./Solution.qs"
})