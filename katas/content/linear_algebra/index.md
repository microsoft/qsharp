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
    "title": "Matrix Addition", 
    "path": "./addition/", 
    "qsDependencies": [
        "./Common.qs"
    ] 
})



@[section]({
    "id": "linear_algebra__conclusion", 
    "title": "Conclusion" 
})

Congratulations! You should now know enough linear algebra to get started with quantum computing!
