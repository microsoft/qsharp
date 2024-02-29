The easiest way to solve this task is to build upon your implementation of the marking OR oracle from an earlier exercise
and use the phase kickback trick to convert it into a phase oracle.

Since in this task you're evaluating OR of all bits except the $k$-th one, you need to exclude the qubit `x[k]` from the list 
of control qubits for the marking oracle. You can do that using slicing: `x[...k - 1]` gets the subarray of qubits before the $k$-th 
one, and `x[k + 1...]` gets the subarray of qubits after the $k$-th one. Your array of control qubits will be a concatenation of these 
two arrays.

@[solution]({
    "id": "oracles__or_but_kth_oracle_solution",
    "codePath": "Solution.qs"
})
