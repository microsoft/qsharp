# Linear Algebra

@[section]({
    "id": "linear_algebra__overview", 
    "title": "Overview" 
})

This kata introduces you to the basics of linear algebra. Linear algebra is a branch of mathematics dedicated to studying the properties of matrices and vectors, which are used extensively in quantum computing to represent quantum states and operations on them.
This kata doesn't come close to covering the full breadth of the topic, but it should be enough to get you comfortable with the main concepts of linear algebra used in quantum computing.

**This kata covers the following topics:**

* Matrices and vectors
* Basic matrix operations
* Operations and properties of complex matrices
* Inner and outer vector products
* Tensor product
* Eigenvalues and eigenvectors

**What you should know to start working on this kata:**

* Complex numbers. If you need a review of this topic, we recommend that you complete the Complex Arithmetic kata before tackling this one.

*In this kata, the exercises focus on pen-and-paper work rather than on writing programs, since you're much more likely to do the linear algebra for a quantum computing problem before you start writing the code for it than to do it in Q#. Every task asks you to fill in the results of a math calculation for a specific example that you can do by hand. The Q# code is used as a way to check the results of the calculations rather than as a way to carry them out.*

@[section]({ 
    "id": "linear_algebra__matrices_and_vectors", 
    "title": "Matrices and Vectors" 
})

A **matrix** is set of numbers arranged in a rectangular grid. Here is a $2$ by $2$ matrix:

$$A =
\begin{bmatrix} 1 & 2 \\\ 3 & 4 \end{bmatrix}$$

$A_{i,j}$ refers to the element in row $i$ and column $j$ of matrix $A$ (all indices are 0-based). In the above example, $A_{0,1} = 2$.

An $n \times m$ matrix will have $n$ rows and $m$ columns:

$$\begin{bmatrix}
    x_{0,0} & x_{0,1} & \dotsb & x_{0,m-1} \\\\
    x_{1,0} & x_{1,1} & \dotsb & x_{1,m-1} \\\\
    \vdots  & \vdots  & \ddots & \vdots  \\\\
    x_{n-1,0} & x_{n-1,1} & \dotsb & x_{n-1,m-1}
\end{bmatrix}$$

A $1 \times 1$ matrix is equivalent to a scalar:

$$\begin{bmatrix} 3 \end{bmatrix} = 3$$

Quantum computing uses complex-valued matrices: the elements of a matrix can be complex numbers. This, for example, is a valid complex-valued matrix:

$$\begin{bmatrix}
    1 & i \\\\
    -2i & 3 + 4i
\end{bmatrix}$$

Finally, a **vector** is an $n \times 1$ matrix. Here, for example, is a $3 \times 1$ vector:

$$V = \begin{bmatrix} 1 \\\ 2i \\\ 3 + 4i \end{bmatrix}$$

Since vectors always have a width of $1$, vector elements are sometimes written using only one index. In the above example, $V_0 = 1$ and $V_1 = 2i$.

@[section]({ 
    "id": "linear_algebra__matrix_addition", 
    "title": "Matrix Addition" 
})

The easiest matrix operation is **matrix addition**. Matrix addition works between two matrices of the same size, and adds each number from the first matrix to the number in the same position in the second matrix:

$$\begin{bmatrix}
    x_{0,0} & x_{0,1} & \dotsb & x_{0,m-1} \\\\
    x_{1,0} & x_{1,1} & \dotsb & x_{1,m-1} \\\\
    \vdots  & \vdots  & \ddots & \vdots  \\\\
    x_{n-1,0} & x_{n-1,1} & \dotsb & x_{n-1,m-1}
\end{bmatrix} +$$
$$+ \begin{bmatrix}
    y_{0,0} & y_{0,1} & \dotsb & y_{0,m-1} \\\\
    y_{1,0} & y_{1,1} & \dotsb & y_{1,m-1} \\\\
    \vdots  & \vdots  & \ddots & \vdots  \\\\
    y_{n-1,0} & y_{n-1,1} & \dotsb & y_{n-1,m-1}
\end{bmatrix} =$$
$$= \begin{bmatrix}
    x_{0,0} + y_{0,0} & x_{0,1} + y_{0,1} & \dotsb & x_{0,m-1} + y_{0,m-1} \\\\
    x_{1,0} + y_{1,0} & x_{1,1} + y_{1,1} & \dotsb & x_{1,m-1} + y_{1,m-1} \\\\
    \vdots  & \vdots  & \ddots & \vdots  \\\\
    x_{n-1,0} + y_{n-1,0} & x_{n-1,1} + y_{n-1,1} & \dotsb & x_{n-1,m-1} + y_{n-1,m-1}
\end{bmatrix}$$

Similarly, we can compute $A - B$ by subtracting elements of $B$ from corresponding elements of $A$.

Matrix addition has the following properties:

* Commutativity: $A + B = B + A$
* Associativity: $(A + B) + C = A + (B + C)$

@[exercise]({ 
    "id": "linear_algebra__addition", 
    "title": "Add Matrices", 
    "path": "./addition/", 
    "qsDependencies": [
        "./Common.qs"
    ] 
})


@[section]({ 
    "id": "linear_algebra__scalar_multiplication", 
    "title": "Scalar Multiplication" 
})

The next matrix operation is **scalar multiplication** - multiplying the entire matrix by a scalar (real or complex number):

$$a \cdot
\begin{bmatrix}
    x_{0,0} & x_{0,1} & \dotsb & x_{0,m-1} \\\\
    x_{1,0} & x_{1,1} & \dotsb & x_{1,m-1} \\\\
    \vdots  & \vdots  & \ddots & \vdots  \\\\
    x_{n-1,0} & x_{n-1,1} & \dotsb & x_{n-1,m-1}
\end{bmatrix} =
\begin{bmatrix}
    a \cdot x_{0,0} & a \cdot x_{0,1} & \dotsb & a \cdot x_{0,m-1} \\\\
    a \cdot x_{1,0} & a \cdot x_{1,1} & \dotsb & a \cdot x_{1,m-1} \\\\
    \vdots  & \vdots  & \ddots & \vdots  \\\\
    a \cdot x_{n-1,0} & a \cdot x_{n-1,1} & \dotsb & a \cdot x_{n-1,m-1}
\end{bmatrix}$$

Scalar multiplication has the following properties:

* Associativity: $x \cdot (yA) = (x \cdot y)A$
* Distributivity over matrix addition: $x(A + B) = xA + xB$
* Distributivity over scalar addition: $(x + y)A = xA + yA$

@[exercise]({ 
    "id": "linear_algebra__scalar_multiplication_ex", 
    "title": "Multiply a Matrix by a Scalar", 
    "path": "./scalar_multiplication/", 
    "qsDependencies": [
        "./Common.qs"
    ] 
})


@[section]({ 
    "id": "linear_algebra__matrix_multiplication", 
    "title": "Matrix Multiplication" 
})

**Matrix multiplication** is a very important and somewhat unusual operation. The unusual thing about it is that neither its operands nor its output are the same size: an $n \times m$ matrix multiplied by an $m \times k$ matrix results in an $n \times k$ matrix. 
That is, for matrix multiplication to be applicable, the number of columns in the first matrix must equal the number of rows in the second matrix.

Here is how matrix product is calculated: if we are calculating $AB = C$, then

$$C_{i,j} = A_{i,0} \cdot B_{0,j} + A_{i,1} \cdot B_{1,j} + \dotsb + A_{i,m-1} \cdot B_{m-1,j} = \sum_{t = 0}^{m-1} A_{i,t} \cdot B_{t,j}$$

Here is a small example:

$$\begin{bmatrix}
    1 & 2 & 3 \\\\
    4 & 5 & 6
\end{bmatrix}
\begin{bmatrix}
    1 \\\\
    2 \\\\
    3
\end{bmatrix} =
\begin{bmatrix}
    1 \cdot 1 + 2 \cdot 2 + 3 \cdot 3 \\\\
    4 \cdot 1 + 5 \cdot 2 + 6 \cdot 3
\end{bmatrix} =
\begin{bmatrix}
    14 \\\\
    32
\end{bmatrix}$$

Matrix multiplication has the following properties:

* Associativity: $A(BC) = (AB)C$
* Distributivity over matrix addition: $A(B + C) = AB + AC$ and $(A + B)C = AC + BC$
* Associativity with scalar multiplication: $xAB = x(AB) = A(xB)$

> Note that matrix multiplication is **not commutative:** $AB$ rarely equals $BA$.

Another very important property of matrix multiplication is that a matrix multiplied by a vector produces another vector.

An **identity matrix** $I_n$ is a special $n \times n$ matrix which has $1$s on the main diagonal, and $0$s everywhere else:

$$I_n =
\begin{bmatrix}
    1 & 0 & \dotsb & 0 \\\\
    0 & 1 & \dotsb & 0 \\\\
    \vdots & \vdots & \ddots & \vdots \\\\
    0 & 0 & \dotsb & 1
\end{bmatrix}$$

What makes it special is that multiplying any matrix (of compatible size) by $I_n$ returns the original matrix. To put it another way, if $A$ is an $n \times m$ matrix:

$$AI_m = I_nA = A$$

This is why $I_n$ is called an identity matrix - it acts as a **multiplicative identity**. In other words, it is the matrix equivalent of the number $1$.

@[exercise]({ 
    "id": "linear_algebra__matrix_multiplication_ex", 
    "title": "Multiply Two Matrices", 
    "path": "./multiplication/", 
    "qsDependencies": [
        "./Common.qs"
    ] 
})


@[section]({
    "id": "linear_algebra__inverse_matrices", 
    "title": "Inverse Matrices" 
})

A square $n \times n$ matrix $A$ is **invertible** if it has an inverse $n \times n$ matrix $A^{-1}$ with the following property:

$$AA^{-1} = A^{-1}A = I_n$$

In other words, $A^{-1}$ acts as the **multiplicative inverse** of $A$.

Another, equivalent definition highlights what makes this an interesting property. For any matrices $B$ and $C$ of compatible sizes:

$$A^{-1}(AB) = A(A^{-1}B) = B$$
$$(CA)A^{-1} = (CA^{-1})A = C$$

A square matrix has a property called the **determinant**, with the determinant of matrix $A$ being written as $|A|$. A matrix is invertible if and only if its determinant isn't equal to $0$.

For a $2 \times 2$ matrix $A$, the determinant is defined as $|A| = A_{0,0} \cdot A_{1,1} - A_{0,1} \cdot A_{1,0}$.

For larger matrices, the determinant is defined through determinants of sub-matrices. You can learn more from [Wikipedia](https://en.wikipedia.org/wiki/Determinant) or from [Wolfram MathWorld](http://mathworld.wolfram.com/Determinant.html).

@[exercise]({ 
    "id": "linear_algebra__inverse_matrix_ex", 
    "title": "Invert a Matrix", 
    "path": "./inverse/", 
    "qsDependencies": [
        "./Common.qs"
    ] 
})



@[section]({
    "id": "linear_algebra__conclusion", 
    "title": "Conclusion" 
})

Congratulations! You should now know enough linear algebra to get started with quantum computing!
