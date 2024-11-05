The first thing you notice is that, unlike in the previous task, you can't represent this state as a tensor product of two individual qubit states - this goal state is **not** separable. 

> How can you see this? Let's assume that this state can be represented as a tensor product of two qubit states: 
>
> $$\ket{\psi_1} \otimes \ket{\psi_2} = (\alpha_1\ket{0} + \beta_1\ket{1}) \otimes (\alpha_2\ket{0} + \beta_2\ket{1}) = \alpha_1\alpha_2\ket{00} + \alpha_1\beta_2\ket{01} + \beta_1\alpha_2\ket{10} + \beta_1\beta_2\ket{11}$$ 
>
>In order for this state to be equal to $\frac{1}{\sqrt2}\big(\ket{00} + \ket{11}\big)$, you need to have $\alpha_1\alpha_2 = \beta_1\beta_2 = \frac{1}{\sqrt2}$ and at the same time $\alpha_1\beta_2 = \beta_1\alpha_2 = 0$, which is impossible.
>
>This is the phenomena called **entanglement**, in which the states of the qubits are linked together and can't be considered individually.  

Let's see what steps you can take to prepare this state without factoring it into states of individual qubits.

---

First, notice that you should end with a superposition of two of the four computational basis for two qubits: $\ket{00}, \ket{01}, \ket{10}, \ket{11}$.

This gives you a hint that you should start by preparing a superposition on at least one of the qubits. Let’s try creating a superposition on the first qubit with a Hadamard gate: 

$$H\ket{0} \otimes \ket{0} = \frac{1}{\sqrt2} (\ket{0} + \ket{1}) \otimes \ket{0} = \frac{1}{\sqrt2} (\ket{00} + \ket{10})$$

Well, you got pretty close, except you need to transform the $\ket{10}$ state to $\ket{11}$.
How can you do this? 

You can take advantage of controlled gates, specifically the [controlled NOT gate](https://en.wikipedia.org/wiki/Controlled_NOT_gate), also referred to as $CNOT$. This gate acts on two qubits, hence it's represented as a $4 \times 4$ unitary matrix. The $CNOT$ gate changes the target qubit from state $\ket{0}$ to $\ket{1}$ and vice versa when the control qubit is $\ket{1}$ and does nothing to the target qubit when the control qubit is $\ket{0}$. The control qubit always remains unchanged. 

$$\text{CNOT} = \begin{bmatrix} 1 & 0 & 0 & 0 \\ 0 & 1 & 0 & 0 \\ 0 & 0 & 0 & 1 \\ 0 & 0 & 1 & 0 \end{bmatrix}$$

If you apply the $CNOT$ gate to the state $\frac{1}{\sqrt2} (\ket{00} + \ket{10})$, taking the first qubit as the control and the second one as target, you'll get exactly the desired goal state. 
 
Steps required to reach goal state:
1. Apply a Hadamard gate to the first qubit.
2. Applying a $CNOT$ gate with first qubit as control and second qubit as target.

In matrix representation, you can represent this operation as a product of two $4 \times 4$ matrices, with the matrix corresponding to the first step being the tensor product of a Hadamard gate on the first qubit and identity gate on the second qubit.

$$H \otimes I = \frac{1}{\sqrt2} \begin{bmatrix} 1 & 1  \\ 1 & -1 \end{bmatrix} \otimes \begin{bmatrix} 1 & 0  \\ 0 & 1 \end{bmatrix} = 
\frac{1}{\sqrt2}\begin{bmatrix} 1 & 0 & 1 & 0 \\ 0 & 1 & 0 & 1 \\ 1 & 0 & -1 & 0 \\ 0 & 1 & 0 & -1 \end{bmatrix}$$

$$\underset{\text{CNOT}}{\underbrace{\begin{bmatrix} 1 & 0 & 0 & 0 \\ 0 & 1 & 0 & 0 \\ 0 & 0 & 0 & 1 \\ 0 & 0 & 1 & 0 \end{bmatrix}}} 
\cdot 
\underset{H \otimes I}{\underbrace{\frac{1}{\sqrt2} \begin{bmatrix} 1 & 0 & 1 & 0 \\ 0 & 1 & 0 & 1 \\ 1 & 0 & -1 & 0 \\ 0 & 1 & 0 & -1 \end{bmatrix}}}
\cdot
\underset{\ket{0}}{\underbrace{ \begin{bmatrix} 1 \\ 0 \\ 0 \\ 0 \end{bmatrix}}}
= \frac{1}{\sqrt2} \begin{bmatrix} 1 & 0 & 1 & 0 \\ 0 & 1 & 0 & 1 \\ 0 & 1 & 0 & -1 \\ 1 & 0 & -1 & 0 \end{bmatrix}
\cdot
\begin{bmatrix} 1 \\ 0 \\ 0 \\ 0 \end{bmatrix}
= \underset{goal}{\underbrace{ \frac{1}{\sqrt2} \begin{bmatrix} 1 \\ 0 \\ 0 \\ 1 \end{bmatrix}}}
$$

Note that in the matrix representation and in Dirac notation the gates are applied from right to left (the rightmost operation happens firts), while in circuit notation the operations are applied from left to right (the leftmost operation happens first).

@[solution]({
    "id": "preparing_states__bell_state_solution",
    "codePath": "./Solution.qs"
})
