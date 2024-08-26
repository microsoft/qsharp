
In vector form, the transformation you need is:

$$
\begin{bmatrix}\color{blue}\alpha\\ \color{blue}\beta\\ \gamma\\ \delta \end{bmatrix}
\rightarrow
\begin{bmatrix} \color{red}\beta\\ \color{red}\alpha\\ \gamma\\ \delta \end{bmatrix}
$$

This can be represented by a matrix:
$$
U = \begin{bmatrix}0 & 1 & 0 & 0\\ 1 & 0 & 0 & 0\\ 0 & 0 & 1 & 0\\ 0 & 0 & 0 & 1 \end{bmatrix}
$$

Remember a two-qubit gate with a similar matrix representation - the $CNOT$ gate:

$$
CNOT = 
 \begin{bmatrix}1 & 0 & 0 & 0\\ 0 & 1 & 0 & 0\\ 0 & 0 & 0 & 1\\ 0 & 0 & 1 & 0 \end{bmatrix}
$$

You need a way to transform the $CNOT$ gate into the unitary transformation represented by $U$.   
Remember that the Pauli X gate flips the state in the single-qubit case. Here you need to use a 2-qubit version of this gate, which would affect only the second qubit. You can conclude that the identity gate needs to be used on the first qubit. In the end, the required gate is a tensor product: $I \otimes X$.

You validate that composition of $I \otimes X$ and the $CNOT$ gate produces the required unitary transformation represented by $U$. 

$$
 (I \otimes X)\cdot CNOT =  
 \left(
\begin{bmatrix}1 & 0 \\  0 & 1 \end{bmatrix}\otimes
\begin{bmatrix} 0 & 1 \\ 1 & 0 \end{bmatrix}
\right) \cdot
\begin{bmatrix}1 & 0 & 0 & 0\\ 0 & 1 & 0 & 0\\ 0 & 0 & 0 & 1\\ 0 & 0 & 1 & 0 \end{bmatrix}=
\begin{bmatrix}0 & 1 & 0 & 0\\ 1 & 0 & 0 & 0\\ 0 & 0 & 0 & 1\\ 0 & 0 & 1 & 0 \end{bmatrix}
\begin{bmatrix} 1 & 0 & 0 & 0\\ 0 & 1 & 0 & 0\\ 0 & 0 & 0 & 1\\ 0 & 0 & 1 & 0 \end{bmatrix}=
\begin{bmatrix} 0 & 1 & 0 & 0\\ 1 & 0 & 0 & 0\\ 0 & 0 & 1 & 0\\ 0 & 0 & 0 & 1 \end{bmatrix}=
U
$$

> Note that the order in which the gates $I \otimes X$ and $CNOT$ are applied doesn't matter in this case.

@[solution]({
    "id": "multi_qubit_gates__anti_controlled_gate_a",
    "codePath": "./SolutionA.qs"
})
Alternatively, you can notice that the task requires application of a Pauli X gate on the second qubit when the first qubit is in the $\ket{0}$ state. This can be achieved with [`ApplyControlledOnInt`](https://learn.microsoft.com/qsharp/api/qsharp-lang/microsoft.quantum.canon/applycontrolledonint) library operation.

Notice that the `ApplyControlledOnInt` operation uses an array of qubits as control, not by a single qubit. 

@[solution]({
    "id": "multi_qubit_gates__anti_controlled_gate_b",
    "codePath": "./SolutionB.qs"
})
