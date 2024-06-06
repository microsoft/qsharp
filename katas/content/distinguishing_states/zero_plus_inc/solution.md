A simple strategy that gives an inconclusive result with probability 0.75 and never errs in case it yields a conclusive result can be obtained from randomizing the choice of measurement basis between the computational basis and the Hadamard basis.
    
Notice that when measured in the standard basis, the state $\ket{0}$ will always lead to the outcome "0", and the state $\ket{+}$ will lead to outcomes "0" and "1" with probability $\frac12$ each. This means that if we measure "1", we can with certainty conclude that the state was $\ket{+}$.
    
A similar argument applies to the scenario where we measure in the Hadamard basis, where $\ket{0}$ can lead to both "+" and "-" outcomes, and $\ket{+}$ always leads to "+". Then if we measured "-", we can with certainty conclude that the state was $\ket{0}$.
    
This leads to the following scenarios (shown are the conditional probabilities
    of the resulting answers in each of the above scenarios).

<table>
    <tr>
        <th>State</th>
        <th>Basis</th>
        <th>P(0)</th>
        <th>P(1)</th>
        <th>P(-1)</th>
    </tr>
    <tr>
        <td>$\ket{0}$</td>
        <td>Computational</td>
        <td>$0$</td>
        <td>$0$</td>
        <td>$1$</td>
    </tr>
    <tr>
        <td>$\ket{+}$</td>
        <td>Computational</td>
        <td>$0$</td>
        <td>$\frac12$</td>
        <td>$\frac12$</td>
    </tr>
    <tr>
        <td>$\ket{0}$</td>
        <td>Hadamard</td>
        <td>$\frac12$</td>
        <td>$0$</td>
        <td>$\frac12$</td>
    </tr>
    <tr>
        <td>$\ket{+}$</td>
        <td>Hadamard</td>
        <td>$0$</td>
        <td>$0$</td>
        <td>$1$</td>
    </tr>
</table>

Since each of the four scenarios occurs with probability 25%, overall this strategy ends up correctly identifying $\ket{0}$ and $\ket{+}$ states with 12.5% probability each and giving inconclusive result with 75% probability.

> The easiest way to implement the measurement in the Hadamard basis in Q# is to apply an H gate followed by a regular measurement in computational basis.

@[solution]({
    "id": "distinguishing_states__zero_plus_inc_solution",
    "codePath": "Solution.qs"
})
