@[solution]({
    "id": "marking_oracle_seven_solution",
    "codePath": "solution.qs"
})

Consider how the oracle from this exercise acts on two input basis states and two "output" basis states:

$$U_{7,mark} |111\rangle |0\rangle = |111\rangle |0 \oplus f(111)\rangle = |111\rangle |0 \oplus 1\rangle = |111\rangle |1\rangle$$
$$U_{7,mark} |111\rangle |1\rangle = |111\rangle |1 \oplus f(111)\rangle = |111\rangle |1 \oplus 1\rangle = |111\rangle |0\rangle$$

$$U_{7,mark} |110\rangle |0\rangle = |110\rangle |0 \oplus f(110)\rangle = |110\rangle |0 \oplus 0\rangle = |110\rangle |0\rangle$$
$$U_{7,mark} |110\rangle |1\rangle = |110\rangle |1 \oplus f(110)\rangle = |110\rangle |1 \oplus 0\rangle = |110\rangle |1\rangle$$

You can see that the state of the input qubit array is unchanged, and the state of the output qubit changes if $f(x) = 1$ and is unchanged if $f(x) = 0$ - this matches the definition of a marking oracle precisely.

Now let's again apply this oracle to a superposition state $|\alpha\rangle$ such that $|x\rangle$ is a superposition of the $|6\rangle$ and $|7\rangle$ basis states and $|y\rangle = |0\rangle$:
$$|\alpha\rangle = \frac{1}{\sqrt{2}}\big(|110\rangle + |111\rangle\big)|0\rangle = 
|11\rangle \otimes \frac{1}{\sqrt{2}} \big(|0\rangle + |1\rangle\big) \otimes |0\rangle = |11+\rangle |0\rangle$$

Let's consider how our operator $U_{7,mark}$ acts on this state.

> Recall that oracles are linear operators, thus they can be applied to each term individually.

$$U_{7,mark} |\alpha\rangle = \frac{1}{\sqrt{2}} \big(U_{7,mark}|110\rangle |0\rangle + U_{7,mark}|111\rangle |0\rangle\big) =$$
$$= \frac{1}{\sqrt{2}} \big(|110\rangle |0\rangle + |111\rangle |1\rangle\big) := |\epsilon\rangle$$

Was our input state modified during this operation?  Let's simplify the resulting state $|\epsilon\rangle$:
$$|\epsilon\rangle = \frac{1}{\sqrt{2}} \big(|110\rangle |0\rangle + |111\rangle |1\rangle\big) = |11\rangle \otimes \frac{1}{\sqrt{2}} \big(|0\rangle |0\rangle + |1\rangle |1\rangle\big) = $$
$$= |11\rangle \otimes \frac{1}{\sqrt{2}} \big(|00\rangle + |11\rangle\big) = |11\rangle \otimes |\Phi^+\rangle = |11\Phi^+\rangle$$

We have entangled the states of qubits $|x\rangle$ and $|y\rangle$!  This is a common occurrence for marking oracles when the input is a superposition of basis states: after applying the oracle, the input $|x\rangle$ will often become entangled with $|y\rangle$. Thus, while applying the marking oracle to a basis state will leave the input array unchanged, applying the marking oracle to a superposition state will change the state of both the input array and the output qubit.



>As an exercise, what entangled state would we get in the previous example if $|y\rangle = |1\rangle$ instead of $|y\rangle = |0\rangle$?
>
> <details>
>   <summary><b>Answer</b></summary>
> $$U_{7,mark} |11+\rangle |1\rangle = |11\rangle \otimes \frac1{\sqrt2}\big(|01\rangle + |10\rangle\big) = |11\rangle |\Psi^+\rangle$$
> </details>
