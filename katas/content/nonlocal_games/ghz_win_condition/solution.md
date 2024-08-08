There are four inputs possible, (0,0,0), (0,1,1), (1,0,1), and (1,1,0), each with $25\%$ probability.
Therefore, in order to win, the sum of the output bits has to be even if the input is (0,0,0) and odd otherwise.

To check whether the win condition holds, you need to compute the expressions R $\lor$ S $\lor$ T and A $\oplus$ B $\oplus$ C and to compare them:
if they are equal, the game is won. To compute the expressions, you can use [built-in operators](https://learn.microsoft.com/azure/quantum/user-guide/language/expressions/logicalexpressions):
X $\lor$ Y as `x or y` and A $\oplus$ B as `a != b`.

@[solution]({
    "id": "nonlocal_games__ghz_win_condition_solution",
    "codePath": "Solution.qs"
})
