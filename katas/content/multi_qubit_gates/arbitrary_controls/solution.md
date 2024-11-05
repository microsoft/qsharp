You're asked to perform an $X$ gate on the `target` qubit controlled by the state of `controls` qubits; this state should correspond to the mask given by `controlBits`.

If the `controlBits` mask consists of all `true` values, you can use a familiar `Controlled X` gate. What can you do if the mask has some `false` values in it?

Turns out you can transform the state of the control qubits depending on the corresponding elements of `controlBits`: if the element is `false`, you apply an $X$ gate to the corresponding qubit in the `controls` array. After this, `Controlled X` gate will apply an $X$ gate in the exact case that you want.
Finally, you'll need to remember to undo ("uncompute") the first step, otherwise your controlled gate will affect the state of the control qubits as well as the state of the target.

As you can see in the first cell below, this can take quite some coding.

@[solution]({
    "id": "multi_qubit_gates__arbitrary_controlled_solution_a",
    "codePath": "./SolutionA.qs"
})

You can shorten the code a bit using the `within ... apply` construct which takes care of uncomputing the steps done in the first code block automatically:

@[solution]({
    "id": "multi_qubit_gates__arbitrary_controlled_solution_b",
    "codePath": "./SolutionB.qs"
})

Finally, here is how the exact same task could be realized using the library function `ApplyControlledOnBitString`.

@[solution]({
    "id": "multi_qubit_gates__arbitrary_controlled_solution_c",
    "codePath": "./SolutionC.qs"
})
