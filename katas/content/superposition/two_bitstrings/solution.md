The strategy of using an auxiliary qubit to control the preparation process described in the previous task can be applied to this task as well. 

We will start by allocating an auxiliary qubit and preparing it in the $\frac{1}{\sqrt2} (|0\rangle + |1\rangle)$ state using the $H$ gate. The overall state of the system will be 

$$\frac{1}{\sqrt2} (|0\rangle + |1\rangle)_a \otimes |0 \dots 0\rangle_r = \frac{1}{\sqrt2} (|0\rangle_a \otimes |0 \dots 0\rangle_r + |1\rangle_a \otimes |0 \dots 0\rangle_r)$$

At this point, we can prepare the two basis states of the target state separately, bit by bit, controlling the preparation of one of them on the $|0\rangle$ state of the auxiliary qubit and the preparation of the other one - on the $|1\rangle$ state. 
If a bit in one of the bit strings is `true`, we will apply a controlled $X$ gate with the auxiliary qubit as control, the qubit in the corresponding position of the register as target, and control it on the $|0\rangle$ or the $|1\rangle$ state depending on which bit string we are considering at the moment. 
Such controlled gate can be implemented using [`ControlledOnInt`](https://learn.microsoft.com/en-us/qsharp/api/qsharp-lang/microsoft.quantum.canon/applycontrolledonint) library function.

After this the state of the system will be 
$$\frac{1}{\sqrt2} (|0\rangle_a \otimes |bits_1\rangle_r + |1\rangle_a \otimes |bits_2\rangle_r)$$

Finally, we will uncompute the auxiliary qubit by using [`ControlledOnBitString`](https://learn.microsoft.com/en-us/qsharp/api/qsharp-lang/microsoft.quantum.canon/applycontrolledonbitstring) library function with the second bit string and the `X` operation as arguments, the quantum register as the control, and the auxiliary qubit as the target. 
This will affect only the $|1\rangle_a \otimes |bits_2\rangle_r$ term, flipping the state of the auxiliary qubit in it and bringing the system to its final state:

$$|0\rangle_a \otimes \frac{1}{\sqrt2} (|bits_1\rangle + |bits_2\rangle)_r$$

@[solution]({
    "id": "superposition__two_bitstrings_solution_a",
    "codePath": "./SolutionA.qs"
})

It is also possible to solve the task without using an extra qubit, if instead we use one of the qubits in the register in this role. 
While walking through the register and bit strings, the first time the bit strings disagreed, the qubit in the corresponding position would take on the role of the auxiliary qubit; we would put it in superposition using the $H$ gate and perform all subsequent bit flips using that qubit as the control. 

This saves us an additional qubit and allows to skip the uncomputing step, though the code becomes less elegant. 
We will move the classical logic of comparing two bit strings to find the first position in which they differ to a function `FindFirstDiff`.

@[solution]({
    "id": "superposition__two_bitstrings_solution_b",
    "codePath": "./SolutionB.qs"
})
