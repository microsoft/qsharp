### Solution
    
A simple strategy that gives an inconclusive result with probability 0.75 and never errs in case it yields a conclusive result can be obtained from randomizing the choice of measurement basis between the computational basis (std) and the Hadamard basis (had).
    
Notice that when measured in the standard basis, the state $\ket{0}$ will always lead to the outcome "0", and the state $\ket{+}$ will lead to outcomes "0" and "1" with probability $\frac12$ each. This means that if we measure "1", we can with certainty conclude that the state was $\ket{+}$.
    
A similar argument applies to the scenario where we measure in the Hadamard basis, where $\ket{0}$ can lead to both "+" and "-" outcomes, and $\ket{+}$ always leads to "+". Then if we measured "-", we can with certainty conclude that the state was $\ket{0}$.
    
This leads to the following scenarios (shown are the conditional probabilities
    of the resulting answers in each of the above scenarios).
    
    
   State     | Basis |    P(0)   |    P(1)   |   P(-1)
-------------|-------|-----------|-----------|----------
 $\ket{0}$ |  std  |     $0$   |     $0$   |    $1$
 $\ket{+}$ |  std  |     $0$   | $\frac12$ | $\frac12$
 $\ket{0}$ |  had  | $\frac12$ |     $0$   | $\frac12$
 $\ket{+}$ |  had  |     $0$   |     $0$   |    $1$
    
> The easiest way to implement the measurement in the Hadamard basis in Q# is to apply an H gate followed by a regular measurement in computational basis.

@[solution]({
    "id": "distinguishing_states__zero_one_solution",
    "codePath": "Solution.qs"
})
