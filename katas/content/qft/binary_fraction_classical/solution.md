Since you can express the exponent of a sum as a product of exponents ($e^{a+b} = e^a \cdot e^b$), you can implement the required transformation as a sequence of rotation gates from the previous task with increasing $k$.

Each of the individual rotations will use the bit $j_k$ to decide whether to apply the rotation (you only want to apply the rotation if $j_k$ is true), and the index of that bit $k$ to define the rotation angle.
The gate applies for each $k$ will be:

$$U_k = \begin{cases}
I, j_k=0 \\
\textrm{R1Frac}(2, k), j_k=1
\end{cases}$$

Recall that 

$$ \textrm{R1Frac}(2, k)( \alpha \ket{0} + \beta \ket{1}) = \alpha \ket{0} + \beta \cdot e^{2\pi i/2^{k}} \ket{1} = \alpha \ket{0} + \beta \cdot e^{2\pi i \cdot 0. \underset{k-1}{\underbrace{0\dots0}} 1} \ket{1}$$

This means that the overall effect of the gate $U_k$ for each $k$ is 

$$U_k (\alpha \ket{0} + \beta \ket{1}) = \alpha \ket{0} + \beta \cdot e^{2\pi i \cdot 0. \underset{k-1}{\underbrace{0\dots0}} j_k} \ket{1}$$

As you iterate over $k$, the resulting state will get closer and closer to the required one:

<table>
    <tr>
        <th>$k$</th>
        <th>State after step $k$</th>
    </tr>
    <tr>
        <td>$1$</td>
        <td>$$\alpha \ket{0} + \beta \cdot e^{2\pi i \cdot 0.j_1} \ket{1}$$</td>
    </tr>
    <tr>
        <td>$2$</td>
        <td>$$\alpha \ket{0} + \beta \cdot e^{2\pi i \cdot 0.j_1j_2} \ket{1}$$</td>
    </tr>
    <tr>
        <td>...</td>
        <td>...</td>
    </tr>
    <tr>
        <td>$n$</td>
        <td>$$\alpha \ket{0} + \beta \cdot e^{2\pi i \cdot 0.j_1j_2 \dots j_n}\ket{1}$$</td>
    </tr>
</table>

@[solution]({
"id": "qft__binary_fraction_classical_solution_a",
"codePath": "./SolutionA.qs"
})

Alternatively, you can do this in a single rotation using the $R1$ gate if you convert the array $j$ into a rotation angle. You'll need the angle $2\pi \cdot 0.j_1j_2 \dots j_n$, and this fraction can be calculated by converting the bit notation into an integer and dividing it by $2^n$.

This solution is better when considered on its own, but it will not be as helpful as the first one once you get to the next task!

@[solution]({
"id": "qft__binary_fraction_classical_solution_b",
"codePath": "./SolutionB.qs"
})
