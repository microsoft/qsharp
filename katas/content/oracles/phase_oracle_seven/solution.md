You can flip the phase of the basis state $\ket{111}$ using a controlled $Z$ gate, with any two of the qubits as controls and the third one as the target.

@[solution]({
    "id": "oracles__phase_oracle_seven_solution",
    "codePath": "Solution.qs"
})

Consider how this oracle acts on two basis states:
$$U_{7,phase} \ket{111} = -\ket{111}$$
$$U_{7,phase} \ket{110} = \ket{110}$$

You can see that $U_{7,phase}$ doesn't change the input if it's a basis state (other than adding a global phase), and $U_{7,phase}$ doesn't change the norm of the state ($U_{7,phase}$ is a unitary operator).  

However, if you applied this oracle to a superposition state instead, what will that look like?

Suppose that $\ket{\beta}$ is an equal superposition of the states  $\ket{110}$ and $\ket{111}$: 
$$\ket{\beta} = \frac{1}{\sqrt{2}} \big(\ket{110} + \ket{111}\big) = \ket{11} \otimes \frac{1}{\sqrt{2}} \big(\ket{0} + \ket{1}\big) = \ket{11} \otimes \ket{+} = \ket{11+}$$

Let's consider how the operator $U_{7,phase}$ acts on this state:

$$U_{7,phase} \ket{\beta} = U_{7,phase} \Big[\frac{1}{\sqrt{2}} \big(\ket{110} + \ket{111}\big)\Big] =$$

$$= \frac{1}{\sqrt{2}} \big(U_{7,phase} \ket{110} + U_{7,phase} \ket{111}\big) =$$

$$= \frac{1}{\sqrt{2}} \big(\ket{110} - \ket{111}\big) := \ket{\gamma}$$

Was your input state modified during this operation? Let's simplify $\ket{\gamma}$:

$$\ket{\gamma} = \frac{1}{\sqrt{2}} \big(\ket{110} - \ket{111}\big) =$$

$$= \ket{11} \otimes \frac{1}{\sqrt{2}} \big(\ket{0} - \ket{1}\big) =$$

$$= \ket{11} \otimes \ket{-} = \ket{11-} \neq \ket{\beta}$$

Here you see that the oracle modifies the input, if the input state is a *superposition* of the basis states, as a phase oracle will only modify the sign of the basis states.  Thus when a superposition state is provided as input to an oracle, the input state can be modified via the application of the quantum oracle.

> It's also worth noting that while the oracle modified the input when provided a superposition state, it did *not* modify the norm of that state.  As an exercise, you can verify this yourself by taking the norm of $\ket{\beta}$ and $\ket{\gamma}$, which both will result in a value of $1$.
>
> As another exercise, consider how you could distinguish between the input and output state programmatically?  Is there an operation that you could apply to the initial state $\ket{\beta}$ and the final state $\ket{\gamma}$ to show that the two states aren't equivalent through measurement?  As a hint, think about how you could convert the superposition states $\ket{\beta}$ and $\ket{\gamma}$ into the basis states.
