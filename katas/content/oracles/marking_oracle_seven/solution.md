You can flip the state of the target qubit if the basis state of the control qubits is $\ket{111}$ using a controlled $X$ gate.

@[solution]({
    "id": "oracles__marking_oracle_seven_solution",
    "codePath": "Solution.qs"
})

Consider how the oracle from this exercise acts on two input basis states and two "output" basis states:

$$U_{7,mark} \ket{111} \ket{0} = \ket{111} \ket{0 \oplus f(111)} = \ket{111} \ket{0 \oplus 1} = \ket{111} \ket{1}$$

$$U_{7,mark} \ket{111} \ket{1} = \ket{111} \ket{1 \oplus f(111)} = \ket{111} \ket{1 \oplus 1} = \ket{111} \ket{0}$$

$$U_{7,mark} \ket{110} \ket{0} = \ket{110} \ket{0 \oplus f(110)} = \ket{110} \ket{0 \oplus 0} = \ket{110} \ket{0}$$

$$U_{7,mark} \ket{110} \ket{1} = \ket{110} \ket{1 \oplus f(110)} = \ket{110} \ket{1 \oplus 0} = \ket{110} \ket{1}$$

You can see that the state of the input qubit array is unchanged, and the state of the output qubit changes if $f(x) = 1$ and is unchanged if $f(x) = 0$ - this matches the definition of a marking oracle precisely.

Now let's again apply this oracle to a superposition state $\ket{\alpha}$ such that $\ket{x}$ is a superposition of the basis states $\ket{110}$ and $\ket{111}$ and $\ket{y} = \ket{0}$:
$$\ket{\alpha} = \frac{1}{\sqrt{2}}\big(\ket{110} + \ket{111}\big)\ket{0} = 
\ket{11} \otimes \frac{1}{\sqrt{2}} \big(\ket{0} + \ket{1}\big) \otimes \ket{0} = \ket{11+} \ket{0}$$

Let's consider how the operator $U_{7,mark}$ acts on this state.

> Recall that oracles are linear operators, thus they can be applied to each term individually.

$$U_{7,mark} \ket{\alpha} = \frac{1}{\sqrt{2}} \big(U_{7,mark}\ket{110} \ket{0} + U_{7,mark}\ket{111} \ket{0}\big) =$$

$$= \frac{1}{\sqrt{2}} \big(\ket{110} \ket{0} + \ket{111} \ket{1}\big) := \ket{\epsilon}$$

Was your input state modified during this operation?  Let's simplify the resulting state $\ket{\epsilon}$:

$$\ket{\epsilon} = \frac{1}{\sqrt{2}} \big(\ket{110} \ket{0} + \ket{111} \ket{1}\big) = \ket{11} \otimes \frac{1}{\sqrt{2}} \big(\ket{0} \ket{0} + \ket{1} \ket{1}\big) =$$

$$= \ket{11} \otimes \frac{1}{\sqrt{2}} \big(\ket{00} + \ket{11}\big) = \ket{11} \otimes \ket{\Phi^+} = \ket{11\Phi^+}$$

You have entangled the states of qubits $\ket{x}$ and $\ket{y}$!  This is a common occurrence for marking oracles when the input is a superposition of basis states: after applying the oracle, the input $\ket{x}$ will often become entangled with $\ket{y}$. Thus, while applying the marking oracle to a basis state will leave the input array unchanged, applying the marking oracle to a superposition state will change the state of both the input array and the output qubit.

>As an exercise, what entangled state would you get in the previous example if $\ket{y} = \ket{1}$ instead of $\ket{y} = \ket{0}$?
>
> <details>
>   <summary><b>Answer</b></summary>
> $$U_{7,mark} \ket{11+} \ket{1} = \ket{11} \otimes \frac1{\sqrt2}\big(\ket{01} + \ket{10}\big) = \ket{11} \ket{\Psi^+}$$
> </details>
