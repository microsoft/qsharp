Again, this problem is similar to the previous one. This time, each input qubit `x[j]` always affects the state of the target qubit, and the matching classical input `r[i]` specifies the way how it does that. 
If the bit `r[i]` is `true`, the state of the target qubit is flipped if the input qubit `x[j]` is in the $\ket{1}$ state;
otherwise, it is flipped if `x[j]` is in the $\ket{0}$ state. (You can check that this is exactly what the formula 
$\left(r_i x_i \oplus (1 - r_i) (1 - x_i) \right)$ evaluates to!)

This means that we need to modify our solution to the previous task to add a second clause to the if statement, to handle the case when `r[i]` is `false`. The gate we need to apply in this scenario is controlled-on-zero $X$, which we can implement using the library operation `ApplyControlledOnInt`.

@[solution]({
    "id": "marking_oracles__product_negation_solution",
    "codePath": "./Solution.qs"
})
