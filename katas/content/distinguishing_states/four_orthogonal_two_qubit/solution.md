Similarly to the previous task, let's see whether these states can be converted back to the basis states from the task "Distinguish Four Basis States".

To find a transformation that would convert the basis states to $\ket{S_0},...\ket{S_3}$, let's write out the coefficients of these states as column vectors side by side, so that they form a matrix.

 $$\frac12 \begin{bmatrix} 1 & 1 & 1 & 1 \\ 1 & -1 & 1 & -1 \\ 1 & 1 & -1 & -1 \\
\underset{\ket{S_0}}{\underbrace{1}} & \underset{\ket{S_1}}{\underbrace{-1}} & \underset{\ket{S_2}}{\underbrace{-1}} & \underset{\ket{S_3}}{\underbrace{1}} \end{bmatrix}$$

Applying this matrix to each of the basis states will produce the given states. You can check explicitly that applying this transformation to the basis state $\ket{00}$ gives:

$$\frac{1}{2} \begin{bmatrix} 1 & 1 & 1 & 1 \\ 1 & -1 & 1 & -1 \\ 1 & 1 & -1 & -1 \\ 1 & -1 & -1 & 1 \end{bmatrix} \cdot \begin{bmatrix}1 \\ 0 \\ 0 \\ 0 \end{bmatrix} = 
\frac{1}{2}\begin{bmatrix}1 \\ 1 \\ 1 \\ 1 \end{bmatrix} = 
\frac{1}{2} \big(\ket{00} + \ket{01} + \ket{10} + \ket{11}\big) = \ket{S_0}$$

and similarly for the rest of the states.

Notice that the top left $2\times2$ block of this matrix is the same as the top right and the bottom left, and same as the bottom right block multiplied by $-1$. This means that we can represent this transformation as a tensor product of two $H$ gates (for background on this, check the Multi-Qubit Gates kata):

$$ H \otimes H = 
\frac{1}{\sqrt{2}} \begin{bmatrix} 1 & 1 \\ 1 & -1 \end{bmatrix} \otimes \frac{1}{\sqrt{2}} \begin{bmatrix} 1 & 1 \\ 1 & -1 \end{bmatrix} = 
\frac{1}{2} \begin{bmatrix} 1 & 1 & 1 & 1 \\ 1 & -1 & 1 & -1 \\ 1 & 1 & -1 & -1 \\ 1 & -1 & -1 & 1 \end{bmatrix}  $$

Knowing how to prepare the given states, we can convert the input state back to the corresponding basis state, like we've done in the previous task, and measure both qubits to get the answer.

@[solution]({
    "id": "distinguishing_states__four_orthogonal_two_qubit_solution",
    "codePath": "Solution.qs"
})
