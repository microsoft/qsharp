In this task we have to distinguish 4 gates that were identical up to a global phase, i.e., gates $Y$, $iY$, $-Y$ and $-iY$ (in that order). 

One way to do this is "by hand", similar to the previous tasks. First we'll apply the controlled variant of the unitary twice and check whether the result is $I$ or $-I$ (which allows us to distinguish $\pm Y$ and $\pm iY$, since the first two gates squared will be equivalent to the $I$ gate, and the last two to the $-I$ gate). Then we distinguish the gates in each group ($Y$ from $-Y$ or $iY$ from $-iY$) by applying the controlled variant of the unitary once.

@[solution]({
    "id": "distinguishing_unitaries__y_xz_minusy_minusxz_solution",
    "codePath": "Solution.qs"
})
