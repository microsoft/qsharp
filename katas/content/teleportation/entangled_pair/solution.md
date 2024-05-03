Begin by understanding what is the output we are expected to give. The resultant state is a superposition of two states, $|00\rangle$ and $|11\rangle$. Starting with the state $|00\rangle$, which is a basis state of two qubit system, need is to introduce superposition in such a way that we get a total of two states. This can be achieved by applying Hadamard gate to any one of them, let's say qAlice qubit. 

Now the state is in the state $\frac{1}{\sqrt{2}}(|00\rangle + |10\rangle)$. Only thing left is to fix the second term. Looking closely one realizes that when qAlice is in state $|0\rangle$, qBob remains the same but it needs to be flipped otherwise. This is the clear condition of controlled not gate where the controlling qubit is qAlice.

@[solution]({
    "id": "teleportation__entangled_pair_solution",
    "codePath": "./Solution.qs"
})
