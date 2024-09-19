You're going to use the same trick of auxiliary qubits that you used in the previous task.

Since the desired superposition has 4 basis states with equal amplitudes, you're going to need two qubits to define a unique basis to control preparation of each of the basis states in the superposition.

You start by allocating two extra qubits and preparing an equal superposition of all 2-qubit states on them by applying an $H$ gate to each of them:

$$\frac12 (\ket{00} + \ket{01} + \ket{10} + \ket{11})_a \otimes \ket{0 \dots 0}_r$$

Then, for each of the four given bit strings, you walk through it and prepare the matching basis state on the main register of qubits, using controlled $X$ gates with the corresponding basis state of the auxiliary qubits as control.

For example, when preparing the bit string `bits[0]`, you apply $X$ gates controlled on the basis state $\ket{00}$; when preparing the bit string `bits[1]`, you apply $X$ gates controlled on $\ket{10}$, and so on.

> You can choose an arbitrary matching of the 2-qubit basis states used as controls and the bit strings prepared on the main register.
> Since all amplitudes are the same, the result doesn't depend on which state controlled which bit string preparation.
> It can be convenient to use indices of the bit strings, converted to little-endian, to control preparation of the bit strings.
> Q# library function [`ApplyControlledOnInt`](https://learn.microsoft.com/qsharp/api/qsharp-lang/microsoft.quantum.canon/applycontrolledonint) does exactly that.

After this, the system will be in the state

$$\frac12 (\ket{00}_a \ket{bits_0}_r + \ket{10}_a \ket{bits_1}_r + \ket{01}_a \ket{bits_2}_r + \ket{11}_a \ket{bits_3}_r)$$

As the last step, you must uncompute the auxiliary qubits, that is, return them to the $\ket{00}$ state to unentangle them from the main register.

Same as you did in the previous task, you'll use [`ApplyControlledOnBitString`](https://learn.microsoft.com/qsharp/api/qsharp-lang/microsoft.quantum.canon/applycontrolledonbitstring) with the corresponding bit string and the `X` operation as arguments, the quantum register as the control, and the auxiliary qubits as the target.

You'll uncompute each of them separately, so one of the auxiliary qubits will be uncomputed with the `bits[1]` and `bits[3]` bit strings as controls, and the other - with the `bits[2]` and `bits[3]`.

@[solution]({
    "id": "preparing_states__four_bitstrings_solution_a",
    "codePath": "./SolutionA.qs"
})

Alternatively, you can leverage the recursion abilities of Q# to create a superposition of the four bit strings.  This solution also extends to an arbitrary number of bit strings with no code changes.

For this process, you'll look at the first bits of each string and adjust the probability of measuring a $\ket{0}$ or $\ket{1}$ accordingly on the first qubit of your answer. You'll then recursively call (as needed) the process again to adjust the probabilities of measurement on the second bit depending on the first bit.  This process recurses until no more input bits are provided.

Consider, for example, the following four bit strings on which to create a superposition: $\ket{001}, \ket{101}, \ket{111}, \ket{110}$.

You can rewrite the superposition state you need to prepare as

$$\frac12 \big(\ket{001} + \ket{101} + \ket{111} + \ket{110} \big) = \frac12 \ket{0} \otimes \ket{01} + \frac{\sqrt3}{2} \ket{1} \otimes \frac{1}{\sqrt3} \big(\ket{10} + \ket{11} + \ket{10} \big)$$

As the first step of the solution, you need to prepare a state $\frac12 \ket{0} + \frac{\sqrt3}{2} \ket{1}$ on the first qubit (to measure $\ket{0}$ with $\frac14$ probability and to measure $\ket{1}$ with $\frac34$ probability). To do this, you will apply an [`Ry`](https://learn.microsoft.com/qsharp/api/qsharp-lang/microsoft.quantum.intrinsic/ry) rotation to the first qubit.

After this, you'll need to prepare the rest of the qubits in appropriate states depending on the state of the first qubit - state $\ket{01}$ if the first qubit is in state $\ket{0}$ and state $\frac{1}{\sqrt3} \big(\ket{10} + \ket{11} + \ket{10} \big)$ if the first qubit is in state $\ket{1}$. You can do this recursively using the same logic. Let's finish walking through this example in detail.

The second qubit of the recursion follows similarly but depends on the first qubit. If the first qubit measures $\ket{0}$, then you want the second qubit to measure $\ket{0}$ with $100\%$ probability, but if it measures $\ket{1}$, you want it to measure $\ket{0}$ with $\frac13$ probability and $\ket{1}$ with $\frac23$ probability.  For this, you can do a controlled [`Ry`](https://learn.microsoft.com/qsharp/api/qsharp-lang/microsoft.quantum.intrinsic/ry) rotation on the second qubit with the first qubit as control.

The third qubit in this example will have three cases because it depends on the first two qubits; this follows naturally from the recursion.

1. If the first two qubits measure $\ket{00}$, then you need the third qubit to measure $\ket{0}$ with $100\%$ probability.
2. If the first two qubits measure $\ket{10}$, then you need the third qubit to measure $\ket{1}$ with $100\%$ probability.
3. If the first two qubits measure $\ket{11}$, then you need the third qubit to measure $\ket{0}$ with $\frac12$ probability and $\ket{1}$ with $\frac12$ probability.  Just as with the second qubit, a controlled [`Ry`](https://learn.microsoft.com/qsharp/api/qsharp-lang/microsoft.quantum.intrinsic/ry) rotation on the third qubit will accomplish this goal.

> You'll use [`ApplyControlledOnBitString`](https://learn.microsoft.com/qsharp/api/qsharp-lang/microsoft.quantum.canon/applycontrolledonbitstring) operation to perform rotations depending on the state of several previous qubits.

@[solution]({
    "id": "preparing_states__four_bitstrings_solution_b",
    "codePath": "./SolutionB.qs"
})
