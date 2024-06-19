This task is very similar to the task in which we checked whether the string is periodic. 
Same as there, we have a building block that checks our condition (whether the bit string contains a given pattern at the specific position), and our function is an OR of conditions that apply with different parameters (the given pattern can start with position $0$, $1$, $2$, and so on).

The solution is similar as well:

1. Allocate $N - K + 1$ auxiliary qubits, one for each position that can be the beginning of the pattern.
2. Evaluate the condition with each possible position as the beginning of the pattern, and store the results in these qubits.
3. Evaluate the overall function as an OR of the values of the auxiliary qubits.
4. Uncompute the changes done to the states of the auxiliary qubits before releasing them.

@[solution]({
    "id": "marking_oracles__contains_substring_solution",
    "codePath": "./Solution.qs"
})
