# Introduction to Linear Algebra

@[section]({ "id": "linear_algebra__overview", "title": "Overview" })

This is a tutorial designed to introduce you to the basics of linear algebra. Linear algebra is a branch of mathematics dedicated to studying the properties of matrices and vectors, which are used extensively in quantum computing to represent quantum states and operations on them. This tutorial doesn't come close to covering the full breadth of the topic, but it should be enough to get you comfortable with the main concepts of linear algebra used in quantum computing.

This tutorial assumes familiarity with complex numbers; if you need a review of this topic, we recommend that you complete the [Complex Arithmetic tutorial](../complex_arithmetic/index.md) before tackling this one.

This tutorial covers the following topics:

* Matrices and vectors
* Basic matrix operations
* Operations and properties of complex matrices
* Inner and outer vector products
* Tensor product
* Eigenvalues and eigenvectors

If you need to look up some formulas quickly, you can find them in this [reference sheet](https://github.com/microsoft/QuantumKatas/blob/main/quickref/qsharp-quick-reference.pdf).

@[section]({ "id": "linear_algebra__part1", "title": "Part I. Matrices and Basic Operations" })

## Matrices and Vectors

A **matrix** is set of numbers arranged in a rectangular grid. Here is a $2$ by $2$ matrix:

$A = \begin{bmatrix} 1 & 2 \\\\ 3 & 4 \end{bmatrix}$

$A_{i,j}$ refers to the element in row $i$ and column $j$  of matrix $A$ (all indices are 0-based). In the above example, $A_{0,1} = 2$.

An$n \times m$ matrix will have $n$ rows and $m$ columns. For example:

$$\begin{bmatrix}
x_{0,0} & x_{0,1} & \dotsb & x_{0,m-1} \\\\
x_{1,0} & x_{1,1} & \dotsb & x_{1,m-1} \\\\
\vdots  & \vdots  & \ddots & \vdots  \\\\
x_{n-1,0} & x_{n-1,1} & \dotsb & x_{n-1,m-1}\end{bmatrix}$$

A $1 \times 1$ matrix is equivalent to a scalar:

$$\begin{bmatrix} 3 \end{bmatrix} = 3$$

Quantum computing uses complex-valued matrices. That is, the elements of a matrix can be complex numbers. This, for example, is a valid complex-valued matrix:

$$\begin{bmatrix}  1 & i \\\\ -2i & 3 + 4i \end{bmatrix}$$

Finally, a **vector** is an $n \times 1$ matrix. Here, for example, is a $3 \times 1$ vector:

$$V = \begin{bmatrix} 1 \\\\ 2i \\\\ 3 + 4i \end{bmatrix}$$

Since vectors always have a width of  $1$, vector elements are sometimes written using only one index. In the above example,  and $V_0 = 1$ and $V_1 = 2i$.

## Matrix Addition

The easiest matrix operation is **matrix addition**. Matrix addition works between two matrices of the same size, and adds each number from the first matrix to the number in the same position in the second matrix:

$$\begin{bmatrix}
x_{0,0} & x_{0,1} & \dotsb & x_{0,m-1} \\\\
x_{1,0} & x_{1,1} & \dotsb & x_{1,m-1} \\\\
\vdots  & \vdots  & \ddots & \vdots  \\\\
x_{n-1,0} & x_{n-1,1} & \dotsb & x_{n-1,m-1}
\end{bmatrix}+
\begin{bmatrix}y_{0,0} & y_{0,1} & \dotsb & y_{0,m-1} \\\\
y_{1,0} & y_{1,1} & \dotsb & y_{1,m-1} \\\\
\vdots  & \vdots  & \ddots & \vdots  \\\\
y_{n-1,0} & y_{n-1,1} & \dotsb & y_{n-1,m-1}
\end{bmatrix}=
\begin{bmatrix}
x_{0,0} + y_{0,0} & x_{0,1} + y_{0,1} & \dotsb & x_{0,m-1} + y_{0,m-1} \\\\
x_{1,0} + y_{1,0} & x_{1,1} + y_{1,1} & \dotsb & x_{1,m-1} + y_{1,m-1} \\\\
\vdots  & \vdots  & \ddots & \vdots  \\\\
x_{n-1,0} + y_{n-1,0} & x_{n-1,1} + y_{n-1,1} & \dotsb & x_{n-1,m-1} + y_{n-1,m-1}
\end{bmatrix}$$

Similarly, we can compute $A$-$B$ by subtracting elements of $B$ from corresponding elements of $A$.

Matrix addition has the following properties:

Commutativity: $A + B = B + A$

Associativity: $(A + B) + C = A + (B + C)$

### Exercise 1: Matrix addition.
**Inputs:**

1. An $n \times m$ matrix $A$, represented as a two-dimensional list.
2. An $n \times m$ matrix $B$, represented as a two-dimensional list.

**Output:** Return the sum of the matrices $A + B$ - an $n \times m$ matrix, represented as a two-dimensional list.

When representing matrices as lists, each sub-list represents a row. For example, list [[1, 2], [3, 4]] represents the following matrix:

$$\begin{bmatrix} 1 & 2 \\ 3 & 4 \end{bmatrix}$$

See the [Linear Algebra Workbook](./workbook_linear_algebra.md#exercise-1-matrix-addition) for an explanation of the solution.

A video explanation can be found [here](https://www.youtube.com/watch?v=WR9qCSXJlyY).

## Scalar Multiplication

The next matrix operation is ****scalar multiplication** - multiplying the entire matrix by a scalar (real or complex number):

$$a\cdot
\begin{bmatrix}
x_{0,0} & x_{0,1} & \dotsb & x_{0,m-1} \\\\
x_{1,0} & x_{1,1} & \dotsb & x_{1,m-1} \\\\
\vdots  & \vdots  & \ddots & \vdots  \\\\
x_{n-1,0} & x_{n-1,1} & \dotsb & x_{n-1,m-1}
\end{bmatrix}=
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

### Exercise 2: Scalar multiplication.

**Inputs:**

1. A scalar $x$.

2. An $n \times m$ matrix $A$.

Output: **Output:** Return the $n \times m$ matrix $x \cdot A$.

See the [Linear Algebra Workbook](./workbook_linear_algebra.md#exercise-2-scalar-multiplication) for an explanation of the solution.

A video explanation can be found [here](https://www.youtube.com/watch?v=WR9qCSXJlyY).

## Matrix Multiplication

Matrix multiplication is a very important and somewhat unusual operation. The unusual thing about it is that neither its operands nor its output are the same size: an $n \times m$ matrix multiplied by an $m \times k$ matrix results in an $n \times k$ matrix.

That is, for matrix multiplication to be applicable, the number of columns in the first matrix must equal the number of rows in the second matrix.

Here is how matrix product is calculated: if we are calculating $AB = C$, then, $$C_{i,j} = A_{i,0} \cdot B_{0,j} + A_{i,1} \cdot B_{1,j} + \dotsb + A_{i,m-1} \cdot B_{m-1,j} = \sum_{t = 0}^{m-1} A_{i,t} \cdot B_{t,j}$$

Here is a small example:

$$\begin{bmatrix}
\color{blue} 1 & \color{blue} 2 & \color{blue} 3 \\\\
\color{red}  4 & \color{red}  5 & \color{red}  6
\end{bmatrix}
\begin{bmatrix}
1 \\\\
2 \\\\
3
\end{bmatrix}
\begin{bmatrix}
(\color{blue} 1 \cdot 1) + (\color{blue} 2 \cdot 2) + (\color{blue} 3 \cdot 3) \\\\
(\color{red}  4 \cdot 1) + (\color{red}  5 \cdot 2) + (\color{red}  6 \cdot 3)
\end{bmatrix}
\begin{bmatrix}
14 \\\\
32
\end{bmatrix}$$  

Matrix multiplication has the following properties:

* Associativity: $A(BC) = (AB)C$
* Distributivity over matrix addition: $A(B + C) = AB + AC$ and $(A + B)C = AC + BC$
* Associativity with scalar multiplication: $xAB = x(AB) = A(xB)$

 **Note:** Matrix multiplication is **not commutative**, $AB$ rarely equals $BA$.

Another very important property of matrix multiplication is that a matrix multiplied by a vector produces another vector.

An **identity matrix** $I_n$ is a special $n \times n$ matrix which has $1$ on the main diagonal, and $0$ everywhere else:

$$I_n =
\begin{bmatrix}
1 & 0 & \dotsb & 0 \\\\
0 & 1 & \dotsb & 0 \\\\
\vdots & \vdots & \ddots & \vdots \\\\
0 & 0 & \dotsb & 1
\end{bmatrix}$$

What makes it special is that multiplying any matrix (of compatible size) by $I_n$
 returns the original matrix. To put it another way, if $A$ is an $n \times m$ matrix:

 $$AI_m = I_nA = A$$

This is why $I_n$ is called an identity matrix - it acts as a **multiplicative identity**. In other words, it is the matrix equivalent of the number $1$.

### Exercise 3: Matrix multiplication.
**Inputs:**

1. An $n \times m$ matrix $A$.
2. An $m \times k$ matrix $B$.

**Output:** Return the $n \times k$ matrix equal to the matrix product $AB$.

See the [Linear Algebra Workbook](./workbook_linear_algebra.md#exercise-3-matrix-multiplication) for an explanation of the solution.

A video explanation can be found [here](https://www.youtube.com/watch?v=WR9qCSXJlyY).

## Inverse Matrices

A square $n \times n$ matrix $A$ is **invertible** if it has an inverse $n \times n$ matrix $A^{-1}$ with the following property:

$$AA^{-1} = A^{-1}A = I_n$$

In other words, $A^{-1}$ acts as the **multiplicative inverse** of $A$.

Another, equivalent definition highlights what makes this an interesting property. For any matrices $B$ and $C$ of compatible sizes:

$$A^{-1}(AB) = A(A^{-1}B) = B$$
$$(CA)A^{-1} = (CA^{-1})A = C$$

A square matrix has a property called the **determinant**, with the determinant of matrix $A$ being written as $|A|$. A matrix is invertible if and only if its determinant isn't equal to $0$.

For a $2 \times 2$ matrix $A$, the determinant is defined as $|A| = (A_{0,0} \cdot A_{1,1}) - (A_{0,1} \cdot A_{1,0})$.

For larger matrices, the determinant is defined through determinants of sub-matrices. You can learn more from [Wikipedia](https://en.wikipedia.org/wiki/Determinant) or from [Wolfram MathWorld](http://mathworld.wolfram.com/Determinant.html).

### Exercise 4: Matrix Inversion

**Input:** An invertible $2 \times 2$ matrix $A$.

**Output:** Return the inverse of $A$, a $2 \times 2$ matrix $A^{-1}$.

For this exercise, $|A|$ is guaranteed to be non-zero.

Try to come up with a general method of doing it by hand first. If you get stuck, you may find [this Wikipedia article](https://en.wikipedia.org/wiki/Invertible_matrix#Inversion_of_2_%C3%97_2_matrices) useful.

A video explanation can be found [here](https://www.youtube.com/watch?v=01c12NaUQDw).

Finally, see the [Linear Algebra Workbook](./workbook_linear_algebra.md#exercise-4-matrix-inversion) for an explanation of the solution.

## Transpose

The **transpose** operation, denoted as $A^T$, is essentially a reflection of the matrix across the diagonal: $(A^T)_{i,j} = A_{j,i}$.

Given an $n \times m$ matrix $A$, its transpose is the $m \times n$ matrix $A^T$, such that if:

$$A =
\begin{bmatrix}
    x_{0,0} & x_{0,1} & \dotsb & x_{0,m-1} \\\\
    x_{1,0} & x_{1,1} & \dotsb & x_{1,m-1} \\\\
    \vdots & \vdots & \ddots & \vdots \\\\
    x_{n-1,0} & x_{n-1,1} & \dotsb & x_{n-1,m-1}
\end{bmatrix}$$

then:

$$A^T =
\begin{bmatrix}
    x_{0,0} & x_{1,0} & \dotsb & x_{n-1,0} \\\\
    x_{0,1} & x_{1,1} & \dotsb & x_{n-1,1} \\\\
    \vdots & \vdots & \ddots & \vdots \\\\
    x_{0,m-1} & x_{1,m-1} & \dotsb & x_{n-1,m-1}
\end{bmatrix}$$

for example:

$$\begin{bmatrix}
1 & 2 \\\\
3 & 4 \\\\
5 & 6
\end{bmatrix}^T=
\begin{bmatrix}
1 & 3 & 5 \\\\
    2 & 4 & 6
\end{bmatrix}$$

A **symmetric** matrix is a square matrix which equals its own transpose: $A = A^T$. To put it another way, it has reflection symmetry (hence the name) across the main diagonal. For example, the following matrix is symmetric:

$$\begin{bmatrix}
    1 & 2 & 3 \\\\
    2 & 4 & 5 \\\\
    3 & 5 & 6
\end{bmatrix}$$

The transpose of a matrix product is equal to the product of transposed matrices, taken in reverse order:

$$(AB)^T = B^TA^T$$

### Exercise 5: Transpose.

**Input:** An $n \times m$ matrix $A$.

**Output:** Return an $m \times n$ matrix $A^T$, the transpose of $A$.

See the [Linear Algebra Workbook](./workbook_linear_algebra.md#exercise-5-transpose) for an explanation of the solution.

A video explanation can be found [here](https://www.youtube.com/watch?v=TZrKrNVhbjI).

## Conjugate

The next important single-matrix operation is the **matrix conjugate**, denoted as $\overline{A}$. This, as the name might suggest, involves taking the [complex conjugate](../ComplexArithmetic/ComplexArithmetic.ipynb#Complex-Conjugate) of every element of the matrix: if

$$A =
\begin{bmatrix}
x_{0,0} & x_{0,1} & \dotsb & x_{0,m-1} \\\\
x_{1,0} & x_{1,1} & \dotsb & x_{1,m-1} \\\\
\vdots & \vdots & \ddots & \vdots \\\\
x_{n-1,0} & x_{n-1,1} & \dotsb & x_{n-1,m-1} \end{bmatrix}$$

Then:

$$\overline{A} =
\begin{bmatrix}
\overline{x}_{0,0} & \overline{x}_{0,1} & \dotsb & \overline{x}_{0,m-1} \\\\
\overline{x}_{1,0} & \overline{x}_{1,1} & \dotsb & \overline{x}_{1,m-1} \\\\
\vdots & \vdots & \ddots & \vdots \\\\
\overline{x}_{n-1,0} & \overline{x}_{n-1,1} & \dotsb & \overline{x}_{n-1,m-1}
\end{bmatrix}$$

The conjugate of a matrix product equals to the product of conjugates of the matrices:

$$\overline{AB} = (\overline{A})(\overline{B})$$

### Exercise 6: Conjugate.

**Input:** An $n \times m$ matrix $A$.

**Output:** Return an $n \times m$ matrix $\overline{A}$, the conjugate of $A$.

To calculate the conjugate of a matrix take the conjugate of each element. Refer to [Exercise 4 of the complex arithmetic tutorial]() to see how to calculate the conjugate of a complex number.

See the [Linear Algebra Workbook](./workbook_linear_algebra.md#exercise-6-conjugate) for an explanation of the solution.

## Adjoint

The final important single-matrix operation is a combination of the above two. The **conjugate transpose**, also called the **adjoint** of matrix $A$, is defined as $A^\dagger = \overline{(A^T)} = (\overline{A})^T$.

A matrix is known as **Hermitian** or **self-adjoint** if it equals its own adjoint: $A = A^\dagger$. For example, the following matrix is Hermitian:

$$\begin{bmatrix}
1 & i \\\\
-i & 2
\end{bmatrix}$$

The adjoint of a matrix product can be calculated as follows:

$$(AB)^\dagger = B^\dagger A^\dagger$$

### Exercise 7: Adjoint
  
**Input:** An $n \times m$ matrix $A$.

**Output:** Return an $m \times n$ matrix $A^\dagger$, the adjoint of $A$.

See the [Linear Algebra Workbook](./workbook_linear_algebra.md#exercise-7-adjoint) for an explanation of the solution.

## Unitary Matrices

**Unitary matrices** are very important for quantum computing. A matrix is unitary when it is invertible, and its inverse is equal to its adjoint: $U^{-1} = U^\dagger$. That is, an $n \times n$ square matrix $U$ is unitary if and only if $UU^\dagger = U^\dagger U = I_n$.

For example, the following matrix is unitary:

$$\begin{bmatrix}
\frac{1}{\sqrt{2}} & \frac{1}{\sqrt{2}} \\\\
\frac{i}{\sqrt{2}} & \frac{-i}{\sqrt{2}} \\\\
\end{bmatrix}$$

### Exercise 8: Unitary Verification

**Input:** An $n \times n$ matrix $A$.

**Output:** Check if the matrix is unitary and return `True` if it is, or `False` if it isn't.

**Hint:**
Keep in mind, you have only implemented matrix inverses for $2 \times 2$ matrices, and this exercise may give you larger inputs. There is a way to solve this without taking the inverse.

See the [Linear Algebra Workbook](./workbook_linear_algebra.md#exercise-8-unitary-verification) for an explanation of the solution.

## Next Steps
Congratulations! At this point, you should understand enough linear algebra to be able to get started with the tutorials on [the concept of qubit](../qubit/index.md) and on [single-qubit quantum gates]([single-qubit quantum gates](../single_qubit_gates/index.md)).

The next section covers more advanced matrix operations that help explain the properties of qubits and quantum gates.

@[section]({ "id": "linear_algebra__part2", "title": "Part II. Advanced Operations" })

## Inner Product

The **inner product** is yet another important matrix operation that is only applied to vectors. Given two vectors $V$ and $W$ of the same size, their inner product $\langle V , W \rangle$ is defined as a product of matrices $V^\dagger$ and $W$:

$$\langle V , W \rangle = V^\dagger W$$

Let's break this down so it's a bit easier to understand. A $1 \times n$ matrix (the adjoint of an $n \times 1$ vector) multiplied by an $n \times 1$ vector results in a $1 \times 1$ matrix (which is equivalent to a scalar). The result of an inner product is that scalar. 

To put it another way, to calculate the inner product of two vectors, take the corresponding elements $V_k$ and $W_k$, multiply the complex conjugate of $V_k$ by $W_k$, and add up those products:

$$\langle V , W \rangle = \sum_{k=0}^{n-1}\overline{V_k}W_k$$

Here is a simple example:

$$\langle
\begin{bmatrix}
    -6 \\\\
    9i
\end{bmatrix}
,
\begin{bmatrix}
    3 \\\\
    -8
\end{bmatrix}
\rangle =
\begin{bmatrix}
    -6 \\\\
    9i
\end{bmatrix}^\dagger
\begin{bmatrix}
    3 \\\\
    -8
\end{bmatrix}
=\begin{bmatrix} -6 & -9i \end{bmatrix}
\begin{bmatrix}
    3 \\\\
    -8
\end{bmatrix} = (-6) \cdot (3) + (-9i) \cdot (-8) = -18 + 72i$$

If you are familiar with the **dot product**, you will notice that it is equivalent to inner product for real-numbered vectors.

> We use our definition for these tutorials because it matches the notation used in quantum computing. You might encounter other sources which define the inner product a little differently: $\langle V , W \rangle = W^\dagger V = V^T\overline{W}$, in contrast to the $V^\dagger W$ that we use. These definitions are almost equivalent, with some differences in the scalar multiplication by a complex number.

An immediate application for the inner product is computing the **vector norm**. The norm of vector $V$ is defined as $||V|| = \sqrt{\langle V , V \rangle}$. This condenses the vector down to a single non-negative real value. If the vector represents coordinates in space, the norm happens to be the length of the vector. A vector is called **normalized** if its norm is equal to $1$.

The inner product has the following properties:

* Distributivity over addition: $\langle V + W , X \rangle = \langle V , X \rangle + \langle W , X \rangle$ and $\langle V , W + X \rangle = \langle V , W \rangle + \langle V , X \rangle$
* Partial associativity with scalar multiplication: $x \cdot \langle V , W \rangle = \langle \overline{x}V , W \rangle = \langle V , xW \rangle$
* Skew symmetry: $\langle V , W \rangle = \overline{\langle W , V \rangle}$
* Multiplying a vector by a unitary matrix **preserves the vector's inner product with itself** (and therefore the vector's norm): $\langle UV , UV \rangle = \langle V , V \rangle$

> Note that just like matrix multiplication, the inner product is **not commutative**: $\langle V , W \rangle$ won't always equal $\langle W , V \rangle$.
  
### Exercise 9: Inner product.

**Inputs:**

1. An $n \times 1$ vector $V$.
2. An $n \times 1$ vector $W$.

**Output:** Return a complex number - the inner product $\langle V , W \rangle$.

> Note that when this method is used with complex vectors, you should take the modulus of the complex number for the division.

A video explanation can be found [here](https://www.youtube.com/watch?v=FCmH4MqbFGs).

See the [Linear Algebra Workbook](./workbook_linear_algebra.md#exercise-9-inner-product) for an explanation of the solution.

### Exercise 10: Normalized vectors.

**Input:** A non-zero $n \times 1$ vector $V$.

**Output:** Return an $n \times 1$ vector $\frac{V}{||V||}$ - the normalized version of the vector $V$.

**Hint:** You might need the square root function to solve this exercise. As a reminder

A video explanation can be found [here](https://www.youtube.com/watch?v=7fn03DIW3Ak).

See the [Linear Algebra Workbook](./workbook_linear_algebra.md#exercise-10-normalized-vectors) for an explanation of the solution.

## Outer Product

The **outer product** of two vectors $V$ and $W$ is defined as $VW^\dagger$. That is, the outer product of an $n \times 1$ vector and an $m \times 1$ vector is an $n \times m$ matrix. If we denote the outer product of $V$ and $W$ as $X$, then $X_{i,j} = V_i \cdot \overline{W_j}$.

Here is a simple example:
outer product of $\begin{bmatrix} -3i \\ 9 \end{bmatrix}$ and $\begin{bmatrix} 9i \\ 2 \\ 7 \end{bmatrix}$ is:

$$\begin{bmatrix} \color{blue} {-3i} \\ \color{blue} 9 \end{bmatrix}
\begin{bmatrix} \color{red} {9i} \\ \color{red} 2 \\ \color{red} 7 \end{bmatrix}^\dagger =
\begin{bmatrix} \color{blue} {-3i} \\ \color{blue} 9 \end{bmatrix}
\begin{bmatrix} \color{red} {-9i} & \color{red} 2 & \color{red} 7 \end{bmatrix} =
\begin{bmatrix}
    \color{blue} {-3i} \cdot \color{red} {(-9i)} & \color{blue} {-3i} \cdot \color{red} 2 & \color{blue} {-3i} \cdot \color{red} 7 \\
    \color{blue} 9 \cdot \color{red} {(-9i)} & \color{blue} 9 \cdot \color{red} 2 & \color{blue} 9 \cdot \color{red} 7
\end{bmatrix} =
\begin{bmatrix}
    -27 & -6i & -21i \\
    -81i & 18 & 63
\end{bmatrix}$$

### Exercise 11: Outer product

**Inputs:**

1. An $n \times 1$ vector $V$.
2. An $m \times 1$ vector $W$.

**Output:** Return an $n \times m$ matrix that represents the outer product of $V$ and $W$.

See the [Linear Algebra Workbook](./workbook_linear_algebra.md#exercise-11-outer-product) for an explanation of the solution.

## Tensor Product

The **tensor product** is a different way of multiplying matrices. Rather than multiplying rows by columns, the tensor product multiplies the second matrix by every element of the first matrix.

Given $n \times m$ matrix $A$ and $k \times l$ matrix $B$, their tensor product $A \otimes B$ is an $(n \cdot k) \times (m \cdot l)$ matrix defined as follows:

$A \otimes B$ = $$\begin{bmatrix}
    A_{0,0} \cdot B & A_{0,1} \cdot B & \dotsb & A_{0,m-1} \cdot B \\
    A_{1,0} \cdot B & A_{1,1} \cdot B & \dotsb & A_{1,m-1} \cdot B \\
    \vdots & \vdots & \ddots & \vdots \\
    A_{n-1,0} \cdot B & A_{n-1,1} \cdot B & \dotsb & A_{n-1,m-1} \cdot B
\end{bmatrix}=
\begin{bmatrix}
    A_{0,0} \cdot \color{red} {\begin{bmatrix}B_{0,0} & \dotsb & B_{0,l-1} \\ \vdots & \ddots & \vdots \\ B_{k-1,0} & \dotsb & b_{k-1,l-1} \end{bmatrix}} & \dotsb &
    A_{0,m-1} \cdot \color{blue} {\begin{bmatrix}B_{0,0} & \dotsb & B_{0,l-1} \\ \vdots & \ddots & \vdots \\ B_{k-1,0} & \dotsb & B_{k-1,l-1} \end{bmatrix}} \\
    \vdots & \ddots & \vdots \\
    A_{n-1,0} \cdot \color{blue} {\begin{bmatrix}B_{0,0} & \dotsb & B_{0,l-1} \\ \vdots & \ddots & \vdots \\ B_{k-1,0} & \dotsb & B_{k-1,l-1} \end{bmatrix}} & \dotsb &
    A_{n-1,m-1} \cdot \color{red} {\begin{bmatrix}B_{0,0} & \dotsb & B_{0,l-1} \\ \vdots & \ddots & \vdots \\ B_{k-1,0} & \dotsb & B_{k-1,l-1} \end{bmatrix}}
\end{bmatrix}= \begin{bmatrix}
    A_{0,0} \cdot \color{red} {B_{0,0}} & \dotsb & A_{0,0} \cdot \color{red} {B_{0,l-1}} & \dotsb & A_{0,m-1} \cdot \color{blue} {B_{0,0}} & \dotsb & A_{0,m-1} \cdot \color{blue} {B_{0,l-1}} \\
    \vdots & \ddots & \vdots & \dotsb & \vdots & \ddots & \vdots \\
    A_{0,0} \cdot \color{red} {B_{k-1,0}} & \dotsb & A_{0,0} \cdot \color{red} {B_{k-1,l-1}} & \dotsb & A_{0,m-1} \cdot \color{blue} {B_{k-1,0}} & \dotsb & A_{0,m-1} \cdot \color{blue} {B_{k-1,l-1}} \\
    \vdots & \vdots & \vdots & \ddots & \vdots & \vdots & \vdots \\
    A_{n-1,0} \cdot \color{blue} {B_{0,0}} & \dotsb & A_{n-1,0} \cdot \color{blue} {B_{0,l-1}} & \dotsb & A_{n-1,m-1} \cdot \color{red} {B_{0,0}} & \dotsb & A_{n-1,m-1} \cdot \color{red} {B_{0,l-1}} \\
    \vdots & \ddots & \vdots & \dotsb & \vdots & \ddots & \vdots \\
    A_{n-1,0} \cdot \color{blue} {B_{k-1,0}} & \dotsb & A_{n-1,0} \cdot \color{blue} {B_{k-1,l-1}} & \dotsb & A_{n-1,m-1} \cdot \color{red} {B_{k-1,0}} & \dotsb & A_{n-1,m-1} \cdot \color{red} {B_{k-1,l-1}}
\end{bmatrix}$$

Here is a simple example:
$$\begin{bmatrix} 1 & 2 \\ 3 & 4 \end{bmatrix} \otimes \begin{bmatrix} 5 & 6 \\ 7 & 8 \end{bmatrix} =
\begin{bmatrix}
    1 \cdot \begin{bmatrix} 5 & 6 \\ 7 & 8 \end{bmatrix} & 2 \cdot \begin{bmatrix} 5 & 6 \\ 7 & 8 \end{bmatrix} \\
    3 \cdot \begin{bmatrix} 5 & 6 \\ 7 & 8 \end{bmatrix} & 4 \cdot \begin{bmatrix} 5 & 6 \\ 7 & 8 \end{bmatrix}
\end{bmatrix} =
\begin{bmatrix}
    1 \cdot 5 & 1 \cdot 6 & 2 \cdot 5 & 2 \cdot 6 \\
    1 \cdot 7 & 1 \cdot 8 & 2 \cdot 7 & 2 \cdot 8 \\
    3 \cdot 5 & 3 \cdot 6 & 4 \cdot 5 & 4 \cdot 6 \\
    3 \cdot 7 & 3 \cdot 8 & 4 \cdot 7 & 4 \cdot 8
\end{bmatrix}=
\begin{bmatrix}
    5 & 6 & 10 & 12 \\
    7 & 8 & 14 & 16 \\
    15 & 18 & 20 & 24 \\
    21 & 24 & 28 & 32
\end{bmatrix}$$

Notice that the tensor product of two vectors is another vector: if $V$ is an $n \times 1$ vector, and $W$ is an $m \times 1$ vector, $V \otimes W$ is an $(n \cdot m) \times 1$ vector.

The tensor product has the following properties:

* Distributivity over addition: $(A + B) \otimes C = A \otimes C + B \otimes C$, $A \otimes (B + C) = A \otimes B + A \otimes C$
* Associativity with scalar multiplication: $x(A \otimes B) = (xA) \otimes B = A \otimes (xB)$
* Mixed-product property (relation with matrix multiplication): $(A \otimes B) (C \otimes D) = (AC) \otimes (BD)$

### Exercise 12*: Tensor Product

**Inputs:**

1. An $n \times m$ matrix $A$.
2. A $k \times l$ matrix $B$.

**Output:** Return an $(n \cdot k) \times (m \cdot l)$ matrix $A \otimes B$, the tensor product of $A$ and $B$.

## Next Steps
At this point, you know enough to complete the tutorials on [the concept of qubit](../qubit/index.md), [single-qubit gates](../single_qubit_gates/index.md), [multi-qubit systems](./multi_qubit_systems/index.md), and [multi-qubit gates](./multi_qubit_gates/index.md). The last part of this tutorial is a brief introduction to eigenvalues and eigenvectors, which are used for more advanced topics in quantum computing. Feel free to move on to the next tutorials, and come back here once you encounter eigenvalues and eigenvectors elsewhere.

@[section]({ "id": "linear_algebra__part3", "title": "Part III: Eigenvalues and Eigenvectors" })

Consider the following example of multiplying a matrix by a vector:

$$\begin{bmatrix}
1 & -3 & 3 \\
3 & -5 & 3 \\
6 & -6 & 4
\end{bmatrix}
\begin{bmatrix} 1 \\
1 \\
2 \end{bmatrix} =
\begin{bmatrix} 4 \\
4 \\
8 \end{bmatrix}$$

Notice that the resulting vector is just the initial vector multiplied by a scalar (in this case 4). This behavior is so noteworthy that it is described using a special set of terms.

Given a nonzero $n \\times n$ matrix $A$, a nonzero vector $V$, and a scalar $x$, if $AV = xV$, then $x$ is an **eigenvalue** of $A$, and $V$ is an **eigenvector** of $A$ corresponding to that eigenvalue.

The properties of eigenvalues and eigenvectors are used extensively in quantum computing. You can learn more about eigenvalues, eigenvectors, and their properties at [Wolfram MathWorld](http://mathworld.wolfram.com/Eigenvector.html) or on [Wikipedia](https://en.wikipedia.org/wiki/Eigenvalues_and_eigenvectors).

Can't come up with a solution? See the explained solution in "Finding an Eigenvector" in the [Linear Algebra Workbook](./workbook_linear_algebra.md#exercise-12-tensor-product).

### Exercise 13: Finding an eigenvalue

**Inputs:**

1. An $n \times n$ matrix $A$.
2. An eigenvector $V$ of matrix $A$.

**Output:** Return a real number - the eigenvalue of $A$ that is associated with the given eigenvector.

>**Hint:**
>Multiply the matrix by the vector, then divide the elements of the result by the elements of the original vector. Don't forget though, some elements of the vector may be $0$.

See the [Linear Algebra Workbook](./workbook_linear_algebra.md#exercise-13-finding-an-eigenvalue) for an explanation of the solution.

### Exercise 14**: Finding an eigenvector

**Inputs:**

1. A $2 \times 2$ matrix $A$.
2. An eigenvalue $x$ of matrix $A$.

**Output:** Return any non-zero eigenvector of $A$ that is associated with $x$.

>**Hint:**
>A matrix and an eigenvalue will have multiple eigenvectors (infinitely many, in fact), but you only need to find one.
Try treating the elements of the vector as variables in a system of two equations. Watch out for division by $0$!

See the [Linear Algebra Workbook](./workbook_linear_algebra.md#exercise-14-finding-an-eigenvector) for an explanation of the solution.
