There are four inputs possible, (0,0,0), (0,1,1), (1,0,1), and (1,1,0), each with $25\%$ probability.
Therefore, in order to win, the sum of the output bits has to be even if the input is (0,0,0) and odd otherwise.

To check whether the win condition holds, you need to compute the expressions $r \vee s \vee t$ and $a \oplus b \oplus c$ and to compare them: if they are equal, the game is won. To compute the expressions, you can use [built-in operators](https://learn.microsoft.com/azure/quantum/user-guide/language/expressions/logicalexpressions) and logical function `Xor` from the [`Microsoft.Quantum.Logical`](https://learn.microsoft.com/qsharp/api/qsharp-lang/microsoft.quantum.logical/xor) library.

@[solution]({
    "id": "nonlocal_games__ghz_win_condition_solution",
    "codePath": "Solution.qs"
})
