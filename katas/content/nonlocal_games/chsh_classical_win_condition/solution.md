There are four input pairs (X, Y) possible, (0,0), (0,1), (1,0), and (1,1), each with 25% probability.
In order to win, Alice and Bob have to output different bits if the input is (1,1), and same bits otherwise.

To check whether the win condition holds, you need to compute $x ∧ y$ and $a ⊕ b$ and to compare these values: if they are equal, Alice and Bob won. [`Microsoft.Quantum.Logical`](https://learn.microsoft.com/qsharp/api/qsharp-lang/microsoft.quantum.logical/xor) library offers you logical function `Xor` which you can use for this computation. Alternatively, you can compute these values using [`built-in operators`](https://learn.microsoft.com/azure/quantum/user-guide/language/expressions/logicalexpressions): $x ∧ y$ as `x and y` and $a ⊕ b$ as `a != b`.


@[solution]({
    "id": "nonlocal_games__chsh_classical_win_condition_solution",
    "codePath": "Solution.qs"
})
