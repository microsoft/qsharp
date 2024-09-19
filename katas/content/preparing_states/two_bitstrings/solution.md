The strategy of using an auxiliary qubit to control the preparation process described in the previous task can be applied to this task as well. 

You'll start by allocating an auxiliary qubit and preparing it in the $\frac{1}{\sqrt2} (\ket{0} + \ket{1})$ state using the $H$ gate. The overall state of the system will be 

$$\frac{1}{\sqrt2} (\ket{0} + \ket{1})_a \otimes \ket{0 \dots 0}_r = \frac{1}{\sqrt2} (\ket{0}_a \otimes \ket{0 \dots 0}_r + \ket{1}_a \otimes \ket{0 \dots 0}_r)$$

At this point, you can prepare the two basis states of the target state separately, bit by bit, controlling the preparation of one of them on the $\ket{0}$ state of the auxiliary qubit and the preparation of the other one - on the $\ket{1}$ state. 
If a bit in one of the bit strings is `true`, you'll apply a controlled $X$ gate with the auxiliary qubit as control, the qubit in the corresponding position of the register as target, and control it on the $\ket{0}$ or the $\ket{1}$ state depending on which bit string you're considering at the moment. 
Such controlled gate can be implemented using the [`ApplyControlledOnInt`](https://learn.microsoft.com/qsharp/api/qsharp-lang/microsoft.quantum.canon/applycontrolledonint) library function.

After this the state of the system will be 
$$\frac{1}{\sqrt2} (\ket{0}_a \otimes \ket{bits_1}_r + \ket{1}_a \otimes \ket{bits_2}_r)$$

Finally, you'll uncompute the auxiliary qubit by using [`ApplyControlledOnBitString`](https://learn.microsoft.com/qsharp/api/qsharp-lang/microsoft.quantum.canon/applycontrolledonbitstring) library function with the second bit string and the `X` operation as arguments, the quantum register as the control, and the auxiliary qubit as the target. 
This will affect only the $\ket{1}_a \otimes \ket{bits_2}_r$ term, flipping the state of the auxiliary qubit in it and bringing the system to its final state:

$$\ket{0}_a \otimes \frac{1}{\sqrt2} (\ket{bits_1} + \ket{bits_2})_r$$

@[solution]({
    "id": "preparing_states__two_bitstrings_solution_a",
    "codePath": "./SolutionA.qs"
})

It's also possible to solve the task without using an extra qubit, if instead you use one of the qubits in the register in this role. 
While walking through the register and bit strings, the first time the bit strings disagreed, the qubit in the corresponding position would take on the role of the auxiliary qubit; you'd put it in superposition using the $H$ gate and perform all subsequent bit flips using that qubit as the control. 

This saves you an additional qubit and allows to skip the uncomputing step, though the code becomes less elegant. 
You'll move the classical logic of comparing two bit strings to find the first position in which they differ to a function `FindFirstDiff`.

@[solution]({
    "id": "preparing_states__two_bitstrings_solution_b",
    "codePath": "./SolutionB.qs"
})
