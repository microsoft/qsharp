This problem is similar to the previous one: we will be calculating an expression modulo $3$, so we'll use the `IncrementMod3` operation defined in the previous task as a building block. 

However, this time we'll need to both increment and decrement numbers modulo $3$. Fortunately, decrement is just an adjoint of increment, and Q# compiler can generate it for you automatically, without you implementing it by hand.

This time you need an extra trick: instead of just counting the number of $1$ bits modulo $3$, you need to calculate the remainder of dividing the number by $3$. You can use the fact that consecutive powers of $2$ give remainders $1$ and $-1$ in turn: $2^0 \equiv 1 \mod 3$, $2^1 = 2 \equiv -1 \mod 3$, $2^2 = 4 \equiv 1 \mod 3$, $2^3 = 8 \equiv -1 \mod 3$, and so on. You start iterating from the least significant bit of the number, performing controlled increment of the counter qubits with qubits 0, 2, 4, ... as controls, and controlled decrement with qubits 1, 3, 5, ... as controls.

Finally, the number itself will be divisible by $3$ only if the counter qubits are in the $\ket{00}$ state.

@[solution]({
    "id": "marking_oracles__num_div_3_solution",
    "codePath": "./Solution.qs"
})
