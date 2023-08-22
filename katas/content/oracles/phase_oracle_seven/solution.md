@[solution]({
    "id": "phase_oracle_seven_solution",
    "codePath": "solution.qs"
})

Consider how the oracle acts on two basis states:
$$U_{7,phase} |111\rangle = -|111\rangle$$
$$U_{7,phase} |110\rangle = |110\rangle$$

You can see that $U_{7,phase}$ does not change the input if it's a basis state (other than adding a global phase), and $U_{7,phase}$ does not change the norm of the state ($U_{7,phase}$ is a unitary operator).  

However, if we applied this oracle to a superposition state instead, what will that look like?

Suppose that $|\beta\rangle$ is an equal superposition of the $|6\rangle$ and $|7\rangle$ states (encoded in big endian, with most significant bit first): 
$$|\beta\rangle = \frac{1}{\sqrt{2}} \big(|110\rangle + |111\rangle\big) = |11\rangle \otimes \frac{1}{\sqrt{2}} \big(|0\rangle + |1\rangle\big) = |11\rangle \otimes |+\rangle = |11+\rangle$$

Let's consider how our operator $U_{7,phase}$ acts on this state:

$$U_{7,phase} |\beta\rangle = U_{7,phase} \Big[\frac{1}{\sqrt{2}} \big(|110\rangle + |111\rangle\big)\Big] =$$

$$\frac{1}{\sqrt{2}} \big(U_{7,phase} |110\rangle + U_{7,phase} |111\rangle\big) =$$

$$\frac{1}{\sqrt{2}} \big(|110\rangle - |111\rangle\big) := |\gamma\rangle$$

Was our input state modified during this operation? Let's simplify $|\gamma\rangle$:

$$|\gamma\rangle = \frac{1}{\sqrt{2}} \big(|110\rangle - |111\rangle\big) =$$

$$|11\rangle \otimes \frac{1}{\sqrt{2}} \big(|0\rangle - |1\rangle\big) = $$

$$|11\rangle \otimes |-\rangle = |11-\rangle \neq |\beta\rangle$$

Here we see that the oracle modifies the input, if the input state was a *superposition* of the basis states, as a phase oracle will only modify the sign of the basis states.  Thus when a superposition state is provided as input to an oracle, the input state can be modified via the application of the quantum oracle.

> It is also worth noting that while the oracle modified the input when provided a superposition state, it did *not* modify the norm of that state.  As an exercise, you can verify this yourself by taking the norm of $|\beta\rangle$ and $|\gamma\rangle$, which both will result in a value of $1$.
>
> As another exercise, consider how you could distinguish between the input and output state programmatically?  Is there an operation that you could apply to the initial state $|\beta\rangle$ and the final state $|\gamma\rangle$ to show that the two states are not equivalent through measurement?  As a hint, think about how you could convert the superposition states $|\beta\rangle$ and $|\gamma\rangle$ into the basis states.
