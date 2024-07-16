We see that these two gates differ by a global phase $e^{i\theta/2}$.
In this problem we're free to choose the angle parameter which we'll pass to our gate, so we can choose an angle that make this global phase difference something easy to detect: for $\theta = 2\pi$ $e^{i\theta/2} = -1$, so $R_z(\theta) = -I$, and $R_1(\theta) = I$.

Now we need to distinguish $I$ gate from $-I$ gate, which can be done using controlled variant of the gate in exactly the same way as distinguishing $Z$ gate from $-Z$ gate in the task 'Z or -Z'.

@[solution]({
    "id": "distinguishing_unitaries__rz_r1_solution",
    "codePath": "Solution.qs"
})
