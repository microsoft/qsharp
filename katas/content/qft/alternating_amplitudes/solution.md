Since the amplitudes of this state have no imaginary parts, it is clear that the value of exponents for an arbitrary amplitude $e^{2\pi i \cdot \frac{Fk}{2^n}}$ must be either $2\pi i$ to get an amplitude equal $1$ or $\pi i$ for an amplitude of $-1$. 
This means that only $j_1$ can be $1$: all other bits of $F$ must be $0$. 
Indeed, you can check that using $F = 10...0 = 2^{n - 1}$ with the solution of the previous task yields the required state.

You can simplify preparing this state: instead of using the general encoding of $F$ in the register, you can apply an $X$ gate to the first qubit of the register.

@[solution]({
    "id": "qft__alternating_amplitudes_solution",
    "codePath": "./Solution.qs"
})
