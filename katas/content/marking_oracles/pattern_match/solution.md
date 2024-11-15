This task is similar to the previous one: the function we're evaluating depends only on the states of a subset of qubits, so we can ignore the rest of them. The main difference here is getting the set of qubits we need to use as controls. 

Q# library function `Subarray` from the `Std.Arrays` namespace extracts the elements of the given array at the given indices, which is exactly what we need here.

@[solution]({
    "id": "marking_oracles__pattern_match_solution",
    "codePath": "./Solution.qs"
})
