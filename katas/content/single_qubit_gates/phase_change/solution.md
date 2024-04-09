We know that:

$$
R1(\alpha)=
 \begin{bmatrix}1 & 0\\\0 & \color{red}{e^{i\alpha}}\end{bmatrix}
$$

So we have:

$$
R1(\beta |0\rangle + \gamma |1\rangle) =
 \begin{bmatrix}1 & 0 \\\0 & \color{red}{e^{i\alpha}} \end{bmatrix}
 \begin{bmatrix}\beta\\\ \gamma\\\ \end{bmatrix}=
\begin{bmatrix}1.\beta + 0.\gamma\\\ 0.\beta + {\color{red}{e^{i\alpha}}}\gamma\end{bmatrix}=
 \begin{bmatrix}\beta\\\ {\color{red}{e^{i\alpha}}}\gamma\end{bmatrix}=
 \beta |0\rangle + {\color{red}{e^{i\alpha}}} \gamma |1\rangle
$$

> Note that the results produced by the test harness can be unexpected.
> If you run the kata several times and examine the output, you'll notice that success is signaled even though the corresponding amplitudes of the desired and actual states look very different.
>
> So what's going on? The full state simulator used in these tests performs the computations "up to a global phase", that is, sometimes the resulting state acquires a global phase that doesn't affect the computations or the measurement outcomes, but shows up in DumpMachine output. (You can read more about the global phase in the [Qubit tutorial](../tutorials/Qubit/Qubit.ipynb#Relative-and-Global-Phase).)
>
> For example, in one run you can get the desired state $(0.6000 + 0000i)|0\rangle + (-0.1389 +0.7878i)|1\rangle$ and the actual state $(-0.1042 + 0.5909i)|0\rangle + (-0.7518 -0.2736i)|1\rangle$.
> You can verify that the ratios of amplitudes of the respective basis states are equal: $\frac{-0.1042 + 0.5909i}{0.6} = -0.173667 +0.984833 i = \frac{-0.7518 -0.2736i}{-0.1389 +0.7878i}$, so the global phase acquired by the state is (-0.173667 +0.984833 i).
> You can also check that the absolute value of this multiplier is approximately 1, so it doesn't impact the measurement probabilities.
>
> The testing harness for this and the rest of the tasks checks that your solution implements the required transformation exactly, without introducing any global phase, so it shows up only in the helper output and does not affect the verification of your solution.
> Suppose now that $\alpha = \frac{\pi}{2}$.
> Then $e^{i\alpha}= \cos\frac{\pi}{2} + i\sin\frac{\pi}{2}$.
> And, since $\cos\frac{\pi}{2}= 0$ and $\sin\frac{\pi}{2} = 1$, then we have that $\cos\frac{\pi}{2} + i \sin\frac{\pi}{2} = i$, and  
> $R1(\frac{\pi}{2}) = S$, which we used in the second solution to task phase_filp.
> @[solution]({

    "id": "single_qubit_gates__phase_change_solution",
    "codePath": "./Solution.qs"

})
