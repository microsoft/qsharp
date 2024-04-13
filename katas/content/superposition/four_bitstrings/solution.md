We are going to use the same trick of auxiliary qubits that we used in the previous task.

Since the desired superposition has 4 basis states with equal amplitudes, we are going to need two qubits to define a unique basis to control preparation of each of the basis states in the superposition.

We start by allocating two extra qubits and preparing an equal superposition of all 2-qubit states on them by applying an H gate to each of them:

$$\frac12 (|00\rangle + |01\rangle + |10\rangle + |11\rangle)_a \otimes |0 \dots 0\rangle_r$$

Then, for each of the four given bit strings, we walk through it and prepare the matching basis state on the main register of qubits, using controlled X gates with the corresponding basis state of the auxiliary qubits as control.

For example, when preparing the bit string `bits[0]`, we apply X gates controlled on the basis state $|00\rangle$; when preparing the bit string `bits[1]`, we apply X gates controlled on $|10\rangle$, and so on.

> We can choose an arbitrary matching of the 2-qubit basis states used as controls and the bit strings prepared on the main register.
> Since all amplitudes are the same, the result does not depend on which state controlled which bit string preparation.
> It can be convenient to use indices of the bit strings, converted to little-endian, to control preparation of the bit strings.
> Q# library function [`ApplyControlledOnInt`](https://learn.microsoft.com/en-us/qsharp/api/qsharp-lang/microsoft.quantum.canon/applycontrolledonint) does exactly that.

After this the system will be in the state

$$\frac12 (|00\rangle_a |bits_0\rangle_r + |10\rangle_a |bits_1\rangle_r + |01\rangle_a |bits_2\rangle_r + |11\rangle_a |bits_3\rangle_r)$$

As the last step, we must uncompute the auxiliary qubits, i.e., return them to the $|00\\rangle$ state to unentangle them from the main register.

Same as we did in the previous task, we will use [`ApplyControlledOnBitString`](https://learn.microsoft.com/en-us/qsharp/api/qsharp-lang/microsoft.quantum.canon/applycontrolledonbitstring) with the corresponding bit string and the X operation as arguments, the quantum register as the control, and the auxiliary qubits as the target.

We will uncompute each of them separately, so one of the auxiliary qubits will be uncomputed with the `bits[1]` and `bits[3]` bit strings as controls, and the other - with the `bits[2]` and `bits[3]`.
