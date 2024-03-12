The first thing we notice is that, unlike in the previous task, we cannot represent this state as a tensor product of two individual qubit states - this goal state is NOT separable. 

> How can we see this? Let's assume that this state can be represented as a tensor product of two qubit states: 
>
> $$|\psi_1\rangle \otimes |\psi_2\rangle = (\alpha_1|0\rangle + \beta_1|1\rangle) \otimes (\alpha_2|0\rangle + \beta_2|1\rangle) = \alpha_1\alpha_2|00\rangle + \alpha_1\beta_2|01\rangle + \beta_1\alpha_2|10\rangle + \beta_1\beta_2|11\rangle$$ 
>
>In order for this state to be equal to $\frac{1}{\sqrt2}\big(|00\rangle + |11\rangle\big)$, we need to have $\alpha_1\alpha_2 = \beta_1\beta_2 = \frac{1}{\sqrt2}$ and at the same time $\alpha_1\beta_2 = \beta_1\alpha_2 = 0$, which is impossible.
>
>This is the first time we encounter the phenomena called **entanglement**, in which the states of the qubits are linked together and can not be considered individually.  

Let's see what steps we can take to prepare this state without factoring it into states of individual qubits.

---

First, we notice that we should end with a superposition of two of the four computational basis for two qubits: $|00\rangle, |01\rangle, |10\rangle, |11\rangle$.

This gives us a hint that we should start by preparing a superposition on at least one of the qubits. Let’s try creating a superposition on the first qubit with a Hadamard gate: 

$$H|0\rangle \otimes |0\rangle = \frac{1}{\sqrt2} (|0\rangle + |1\rangle) \otimes |0\rangle = \frac{1}{\sqrt2} (|00\rangle + |10\rangle)$$

Well, we got pretty close, except we need to transform the $|10\rangle$ state to $|11\rangle$.
How can we do this? 

We can take advantage of controlled gates, specifically the [controlled NOT gate](https://en.wikipedia.org/wiki/Controlled_NOT_gate), also referred to as CNOT. This gate acts on two qubits, hence it is represented as a $4 \times 4$ unitary matrix. The CNOT gate changes the target qubit from state $|0\rangle$ to $|1\rangle$ and vice versa when the control qubit is $|1\rangle$ and does nothing to the target qubit when the control qubit is $|0\rangle$. The control qubit always remains unchanged. 

<center>
<table style="background-color: white; border:1px solid; tr  { background-color:transparent; }">
    <col width=300>
    <col width=300>
    <tr>
        <th style="text-align:center; border:1px solid">Matrix</th>
        <th style="text-align:center; border:1px solid">Circuit</th>
    </tr>
    <tr>
        <td style="text-align:center; border:1px solid">$\text{CNOT} = \begin{bmatrix} 1 & 0 & 0 & 0 \\ 0 & 1 & 0 & 0 \\ 0 & 0 & 0 & 1 \\ 0 & 0 & 1 & 0 \end{bmatrix}$</td>
        <td style="text-align:center; border:1px solid"><img src="./img/CNOTGateCircuit.png"/></td>    
      </tr>      
</table> <br>
The matrix and circuit representation of CNOT
</center><br>

If we apply the CNOT gate to the state $\frac{1}{\sqrt2} (|00\rangle + |10\rangle)$, taking the first qubit as the control and the second one as target, we'll get exactly the desired goal state. 
<img src="./img/Task6OutputHadamardasControl.png" width="200"/>
 
<table style="background-color: white; border:1px solid; tr  { background-color:transparent; }">
    <col width=500>
    <col width=300>
    <col width=300>
    <tr>
        <th style="text-align:center; border:1px solid">Steps required to reach goal state</th>
        <th style="text-align:center; border:1px solid">Notation</th>
        <th style="text-align:center; border:1px solid">Circuit</th>
    </tr>
    <tr>
        <td style="text-align:left; border:1px solid">1. Apply a Hadamard gate to the first qubit. <br/> 2. Applying a CNOT with first qubit as control and second qubit as target.</td>
        <td style="text-align:center; border:1px solid; font-bold; font-size: 16px; ">$\frac{1}{\sqrt2} (|00\rangle + |11\rangle)$</td>
        <td style="text-align:center; border:1px solid"><img src="./img/Task6HadamardCNOTCircuit.png"/></td>
      </tr>      
</table>

In matrix representation we can represent this operation as a product of two $4 \times 4$ matrices, with the matrix corresponding to the first step being the tensor product of a Hadamard gate on the first qubit and identity gate on the second qubit.

$$H \otimes I = \frac{1}{\sqrt2} \begin{bmatrix} 1 & 1  \\\ 1 & -1 \end{bmatrix} \otimes \begin{bmatrix} 1 & 0  \\\ 0 & 1 \end{bmatrix} = 
\frac{1}{\sqrt2}\begin{bmatrix} 1 & 0 & 1 & 0 \\\ 0 & 1 & 0 & 1 \\\ 1 & 0 & -1 & 0 \\\ 0 & 1 & 0 & -1 \end{bmatrix}$$

$$\underset{\text{CNOT}}{\underbrace{\begin{bmatrix} 1 & 0 & 0 & 0 \\\ 0 & 1 & 0 & 0 \\\ 0 & 0 & 0 & 1 \\\ 0 & 0 & 1 & 0 \end{bmatrix}}} 
\cdot 
\underset{H \otimes I}{\underbrace{\frac{1}{\sqrt2} \begin{bmatrix} 1 & 0 & 1 & 0 \\\ 0 & 1 & 0 & 1 \\\ 1 & 0 & -1 & 0 \\\ 0 & 1 & 0 & -1 \end{bmatrix}}}
\cdot
\underset{|0\rangle}{\underbrace{ \begin{bmatrix} 1 \\\ 0 \\\ 0 \\\ 0 \end{bmatrix}}}
= \frac{1}{\sqrt2} \begin{bmatrix} 1 & 0 & 1 & 0 \\\ 0 & 1 & 0 & 1 \\\ 0 & 1 & 0 & -1 \\\ 1 & 0 & -1 & 0 \end{bmatrix}
\cdot
\begin{bmatrix} 1 \\\ 0 \\\ 0 \\\ 0 \end{bmatrix}
= \underset{goal}{\underbrace{ \frac{1}{\sqrt2} \begin{bmatrix} 1 \\\ 0 \\\ 0 \\ 1 \end{bmatrix}}}
\label{6.1} \tag{6.1}
$$

Note that in the matrix representation and in Dirac notation the gates are applied from right to left (the rightmost operation happens firts), while in circuit notation the operations are applied from left to right (the leftmost operation happens first).

@[solution]({
    "id": "superposition__bell_state_solution",
    "codePath": "./Solution.qs"
})
