We need to check whether for any value of $P$ the bit string is periodic with period $P$. 
We can use the solution to the previous task as a building block to check periodicity for a specific $P$, but how do we build a complete solution out of these blocks?

You can express the function we're evaluating as "the bit string is periodic with period $1$" OR
"the bit string is periodic with period $2$" OR ... OR "the bit string is periodic with period $N − 1$".
Then, you have to allocate $N - 1$ auxiliary qubits and use them to store the evaluation results for the condition for each value of the period from $1$ to $N - 1$.

After this, you need to compute the OR of the states of these auxiliary qubits: if at least one of them is $1$, the bit string is periodic.
You can do this using the "Implement the OR Oracle" exercise in the Oracles kata, by checking whether the values of all auxiliary qubits are $0$ and then negating the result.

Finally, you have to uncompute the changes you did to the auxiliary qubits to return them to the $\ket{0}$ state before releasing them.

@[solution]({
    "id": "marking_oracles__periodic_solution",
    "codePath": "./Solution.qs"
})
