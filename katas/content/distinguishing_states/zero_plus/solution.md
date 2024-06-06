Let ${\ket{E_a}, \ket{E_b}}$ be a measurement with two outcomes $a$ and $b$, which we identify with the answers, i.e., outcome "a" means we answer "state was $\ket{0}$" and outcome "b" means we answer "state was $\ket{+}$". Then we define

* $P(a|0)$ = probability to observe first outcome given that the state was $\ket{0}$
* $P(b|0)$ = probability to observe second outcome given that the state was $\ket{0}$
* $P(a|+)$ = probability to observe first outcome given that the state was $\ket{+}$
* $P(b|+)$ = probability to observe second outcome given that the state was $\ket{+}$

The task is to maximize the probability to be correct on a single shot experiment, which is the same as to minimize the probability to be wrong on a single shot.

Since the task promises uniform prior distribution of the inputs $\ket{0}$ and $\ket{+}$, i.e., $P(+) = P(0) = \frac{1}{2}$, we get the following expression for the probability of giving a correct answer:

$$P_{correct} = P(0) P(a|0) + P(+) P(b|+) = \frac{1}{2} (P(a|0) + P(b|+))$$

We can represent our measurement as a von Neumann measurement of the following form:

$$\ket{E_a} = R_y(2\alpha) \begin{bmatrix} 1 \\ 0 \end{bmatrix} = \begin{bmatrix} \cos \alpha \\ \sin \alpha \end{bmatrix}$$
$$\ket{E_b} = R_y(2\alpha) \begin{bmatrix} 0 \\ 1 \end{bmatrix} = \begin{bmatrix} - \sin \alpha \\ \cos \alpha \end{bmatrix}$$

Using this representation, we can express our probabilities as follows:

$$P(a|0) = |\braket{E_a|0}|^2 = \cos^2 \alpha$$
    
$$P(b|+) = |\braket{E_b|+}|^2 = \frac{1}{2} - \cos \alpha \sin \alpha$$
    
$$P_{correct} = \frac{1}{2} (\cos^2 \alpha + \frac{1}{2} - \cos \alpha \sin \alpha)$$
    
Maximizing this for $\alpha$, we get max $P_{success} = \frac{1}{2} (1 + \frac{1}{\sqrt{2}}) = 0.8535...$, which is attained for $\alpha = -\pi/8$.
    
This means that $\ket{E_a}$ and $\ket{E_b}$ are the result of rotating $\ket{0}$ and $\ket{1}$, respectively, by $-\pi/8$. If we rotate the whole system by $-\alpha = \pi/8$, we will get $\ket{E_a}=\ket{0}$ and $\ket{E_b}=\ket{1}$, and a measurement in the computational basis will give us the correct result with a probability of 85%.
    
> In Q#, rotating the input state by some angle $\theta$ can be done by applying $Ry$ gate with angle parameter $2\theta$.

@[solution]({ "id": "measurements_nonorthogonal_states_solution", "codePath": "./Solution.qs" })
