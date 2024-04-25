### Solution 1 - using intrinsic gates

In vector form the transformation we need is:

$$
\begin{bmatrix}\color{blue}\alpha\\\ \color{blue}\beta\\\ \gamma\\\ \delta\\\ \end{bmatrix}
\rightarrow
\begin{bmatrix} \color{red}\beta\\\ \color{red}\alpha\\\ \gamma\\\ \delta\\\ \end{bmatrix}
$$

This can be represented by a matrix:
$$
U = \begin{bmatrix}0 & 1 & 0 & 0\\\ 1 & 0 & 0 & 0\\\ 0 & 0 & 1 & 0\\\ 0 & 0 & 0 & 1\\\ \end{bmatrix}
$$

We remember a two-qubit gate with a similar matrix representation - the CNOT gate:

$$
\text{CNOT} = 
 \begin{bmatrix}1 & 0 & 0 & 0\\\ 0 & 1 & 0 & 0\\\ 0 & 0 & 0 & 1\\\ 0 & 0 & 1 & 0\\\ \end{bmatrix}
$$

We need a way to transform the $\text{CNOT}$ gate into the unitary transformation represented by $U$.   
We remember that the Pauli X gate flips the state in the single-qubit case. Here we need to use a 2-qubit version of this gate, which would affect only the second qubit. We conclude, that the idenity gate needs to be used on the first qubit. In the end, the required gate is a tensor product: $I \otimes X$.

We validate that composition of $I \otimes X$ and the $\text{CNOT}$ gate produces the required unitary transformation represented by $U$. 

$$
 (I \otimes X)\cdot \text{CNOT} =  
 \left(
\begin{bmatrix}1 & 0 \\\  0 & 1 \\\ \end{bmatrix}\otimes
\begin{bmatrix} 0 & 1 \\\ 1 & 0 \\\ \end{bmatrix}
\right) \cdot
\begin{bmatrix}1 & 0 & 0 & 0\\\ 0 & 1 & 0 & 0\\\ 0 & 0 & 0 & 1\\\ 0 & 0 & 1 & 0\\\ \end{bmatrix}=
\begin{bmatrix}0 & 1 & 0 & 0\\\ 1 & 0 & 0 & 0\\\ 0 & 0 & 0 & 1\\\ 0 & 0 & 1 & 0\\\ \end{bmatrix}
\begin{bmatrix} 1 & 0 & 0 & 0\\\ 0 & 1 & 0 & 0\\\ 0 & 0 & 0 & 1\\\ 0 & 0 & 1 & 0\\\ \end{bmatrix}=
\begin{bmatrix} 0 & 1 & 0 & 0\\\ 1 & 0 & 0 & 0\\\ 0 & 0 & 1 & 0\\\ 0 & 0 & 0 & 1\\\ \end{bmatrix}=
U
$$

> Note that the order in which the gates $I \otimes X$ and $\text{CNOT}$ are applied doesn't matter in this case.

@[solution]({
    "id": "multi_qubit_gates__amplitudes_swap_solution_a",
    "codePath": "./SolutionA.qs"
})
Alternatively, we can express this gate using the intrinsic gate Z and its controlled variant using the Controlled functor:

### Solution 2 - using a library function

We observe that the task requires application of a Pauli X gate on the second qubit when the first qubit is in the $|0\rangle$ state. This can be achieved with [`ApplyControlledOnInt`](https://learn.microsoft.com/en-us/qsharp/api/qsharp/microsoft.quantum.canon.applycontrolledonbitstring) library function.

Notice that the `ApplyControlledOnInt` function creates a gate controlled by a register - not by a single qubit. 

@[solution]({
    "id": "multi_qubit_gates__amplitudes_swap_solution_b",
    "codePath": "./SolutionB.qs"
})