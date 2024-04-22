We are going to use the same trick of auxiliary qubits that we used in the previous task.

Since the desired superposition has 4 basis states with equal amplitudes, we are going to need two qubits to define a unique basis to control preparation of each of the basis states in the superposition.

We start by allocating two extra qubits and preparing an equal superposition of all 2-qubit states on them by applying an H gate to each of them:

$$\frac12 (|00\rangle + |01\rangle + |10\rangle + |11\rangle)_a \otimes |0 \dots 0\rangle_r$$

Then, for each of the four given bit strings, we walk through it and prepare the matching basis state on the main register of qubits, using controlled X gates with the corresponding basis state of the auxiliary qubits as control.

For example, when preparing the bit string `bits[0]`, we apply X gates controlled on the basis state $|00\rangle$; when preparing the bit string `bits[1]`, we apply X gates controlled on $|10\rangle$, and so on.

> We can choose an arbitrary matching of the 2-qubit basis states used as controls and the bit strings prepared on the main register.
> Since all amplitudes are the same, the result does not depend on which state controlled which bit string preparation.
> It can be convenient to use indices of the bit strings, converted to little-endian, to control preparation of the bit strings.
> Q# library function [`ApplyControlledOnInt`](https://learn.microsoft.com/qsharp/api/qsharp-lang/microsoft.quantum.canon/applycontrolledonint) does exactly that.

After this the system will be in the state

$$\frac12 (|00\rangle_a |bits_0\rangle_r + |10\rangle_a |bits_1\rangle_r + |01\rangle_a |bits_2\rangle_r + |11\rangle_a |bits_3\rangle_r)$$

As the last step, we must uncompute the auxiliary qubits, i.e., return them to the $|00\rangle$ state to unentangle them from the main register.

Same as we did in the previous task, we will use [`ApplyControlledOnBitString`](https://learn.microsoft.com/qsharp/api/qsharp-lang/microsoft.quantum.canon/applycontrolledonbitstring) with the corresponding bit string and the X operation as arguments, the quantum register as the control, and the auxiliary qubits as the target.

We will uncompute each of them separately, so one of the auxiliary qubits will be uncomputed with the `bits[1]` and `bits[3]` bit strings as controls, and the other - with the `bits[2]` and `bits[3]`.

@[solution]({
    "id": "superposition__four_bitstrings_solution_a",
    "codePath": "./SolutionA.qs"
})

Alternatively, we can leverage the recursion abilities of Q# to create a superposition of the four bit strings.  This solution also extends to an arbitrary number of bit strings with no code changes.

For this process we will look at the first bits of each string and adjust the probability of measuring a $|0\rangle$ or $|1\rangle$ accordingly on the first qubit of our answer.  We will then recursively call (as needed) the process again to adjust the probabilities of measurement on the second bit depending on the first bit.  This process recurses until no more input bits are provided.

Consider, for example, the following four bit strings on which to create a superposition: $|001\rangle, |101\rangle, |111\rangle, |110\rangle$.

We can rewrite the superposition state we need to prepare as

$$\frac12 \big(|001\rangle + |101\rangle + |111\rangle + |110\rangle \big) = \frac12 |0\rangle \otimes |01\rangle + \frac{\sqrt3}{2} |1\rangle \otimes \frac{1}{\sqrt3} \big(|10\rangle + |11\rangle + |10\rangle \big)$$

As the first step of the solution, we need to prepare a state $\frac12 |0\rangle + \frac{\sqrt3}{2} |1\rangle$ on the first qubit (to measure $|0\rangle$ with $\frac14$ probability and to measure $|1\rangle$ with $\frac34$ probability).  To do this, we will apply an [`Ry`](https://learn.microsoft.com/qsharp/api/qsharp-lang/microsoft.quantum.intrinsic/ry) rotation to the first qubit.

After this, we'll need to prepare the rest of the qubits in appropriate states depending on the state of the first qubit - state $|01\rangle$ if the first qubit is in state $|0\rangle$ and state $\frac{1}{\sqrt3} \big(|10\rangle + |11\rangle + |10\rangle \big)$ if the first qubit is in state $|1\rangle$. We can do this recursively using the same logic. Let's finish walking through this example in detail.

The second qubit of the recursion follows similarly but depends on the first qubit.  If the first qubit measures $|0\rangle$, then we want the second qubit to measure $|0\rangle$ with 100% probability, but if it measures $|1\rangle$, we want it to measure $|0\rangle$ with $\frac13$ probability and $|1\rangle$ with $\frac23$ probability.  For this, we can do a controlled [`Ry`](https://learn.microsoft.com/qsharp/api/qsharp-lang/microsoft.quantum.intrinsic/ry) rotation on the second qubit with the first qubit as control.

The third qubit in this example will have three cases because it depends on the first two qubits; this follows naturally from the recursion.

1. If the first two qubits measure $|00\rangle$, then we need the third qubit to measure $|0\rangle$ with 100% probability.
2. If the first two qubits measure $|10\rangle$, then we need the third qubit to measure $|1\rangle$ with 100% probability.
3. If the first two qubits measure $|11\rangle$, then we need the third qubit to measure $|0\rangle$ with $\frac12$ probability and $|1\rangle$ with $\frac12$ probability.  Just as with the second qubit, a controlled [`Ry`](https://learn.microsoft.com/qsharp/api/qsharp-lang/microsoft.quantum.intrinsic/ry) rotation on the third qubit will accomplish this goal.

> We will use [`ApplyControlledOnBitString`](https://learn.microsoft.com/qsharp/api/qsharp-lang/microsoft.quantum.canon/applycontrolledonbitstring) operation to perform rotations depending on the state of several previous qubits.

@[solution]({
    "id": "superposition__four_bitstrings_solution_b",
    "codePath": "./SolutionB.qs"
})
