### Solution

Let’s look at some examples of basis states to illustrate the binary numbering system. 

The 4 basis states on $N = 2$ qubits can be split in two columns, where the left column represents the basis states that form the required superposition state for `isEven = false` and the right column - the basis states that form the required superposition state for `isEven = true`.
 
If we do the same basis state split for $N = 3$ qubits, the pattern becomes more obvious.

The two leftmost qubits go through all possible basis states for `isEven = false` and for `isEven = true`, and the rightmost qubit stays in the $|1\rangle$ state for `isEven = false` and in the $|0\rangle$ state for `isEven = true`. 

A quick sanity check for $N = 4$ qubits re-confirms the pattern.
 
Again, the three leftmost qubits go through all possible basis states in both columns, and the rightmost qubit stays in the same state in each column. 

The solution is to put all qubits except the rightmost one into an equal superposition (similar to what we did in Task 9) and to set the rightmost qubit to $|0\rangle$ or $|1\rangle$ depending on the `isEven` flag, using the X operator to convert $|0\rangle$ to $|1\rangle$ if `isEven = false`.
 

@[solution]({
    "id": "superposition__even_odd_solution",
    "codePath": "./Solution.qs"
})
