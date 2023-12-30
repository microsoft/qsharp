# Linear Algebra Tutorial Workbook

**What is this workbook?**

A workbook is a collection of problems, accompanied by solutions to them. 
The explanations focus on the logical steps required to solve a problem; they illustrate the concepts that need to be applied to come up with a solution to the problem, explaining the mathematical steps required. 

Note that a workbook should not be the primary source of knowledge on the subject matter; it assumes that you've already read a tutorial or a textbook and that you are now seeking to improve your problem-solving skills. You should attempt solving the tasks of the respective kata first, and turn to the workbook only if stuck. While a textbook emphasizes knowledge acquisition, a workbook emphasizes skill acquisition.

 This workbook describes the solutions to the problems offered in the [Linear Algebra tutorial](./index.md).

**What you should know for this workbook:**

1. Complex arithmetic.
  
## Exercise 1: Matrix addition

**Inputs:**

1. An $n \times m$ matrix $A$, represented as a two-dimensional list.
2. An $n \times m$ matrix $B$, represented as a two-dimensional list.

**Output:** Return the sum of the matrices $A + B$ - an $n \times m$ matrix, represented as a two-dimensional list.

**Solution:**

Following the definition given in the tutorial, the sum of two matrices is a matrix of element-wise sums of matrix elements; for example, for $2 \times 2$ matrices

$$ A + B =\begin{bmatrix} a & b \\ c & d \end{bmatrix} + \begin{bmatrix} e & f \\ g & h \end{bmatrix} = \begin{bmatrix} a + e & b + f \\ c + g & d + h \end{bmatrix}$$

[Return to task 1 of the Linear Algebra tutorial.](./index.md#exercise-1-matrix-addition)
  
## Exercise 2: Scalar multiplication

**Inputs:**

1. A scalar $x$.
2. An $n \times m$ matrix $A$.

**Output:** Return the $n \times m$ matrix $x \cdot A$.

**Solution:**

We can again follow the definition given in the tutorial: to calculate the product of a number and a matrix, multiply each matrix element by that number. For example, for a $2 \times 2$ matrix:

$$x \cdot A = x \cdot \begin{bmatrix} a & b \\ c & d \end{bmatrix} = \begin{bmatrix} x \cdot a & x \cdot b \\ x \cdot c & x \cdot d \end{bmatrix}  $$ 

[Return to task 2 of the Linear Algebra tutorial.](./index.md#exercise-2-scalar-multiplication)

## Exercise 3: Matrix multiplication

**Inputs:**

1. An $n \times m$ matrix $A$.
2. An $m \times k$ matrix $B$.

**Output:** Return the $n \times k$ matrix equal to the matrix product $AB$.

**Solution:**

Again, the tutorial gives us the definition of how multiplication works, and we just need to implement it in code. Here is an example of multiplying a $2 \times 3$ matrix by a $3 \times 2$ matrix:

$$ A \cdot B =\begin{bmatrix} a & b & c \\ d & e & f \end{bmatrix} \cdot \begin{bmatrix} h & i \\ j & k \\ l & m \end{bmatrix} = \begin{bmatrix} a \cdot h + b \cdot j + c \cdot l & a \cdot i + b \cdot k + c \cdot m \\ 
 d \cdot h + e \cdot j + f \cdot l & d \cdot i + e \cdot k + f \cdot m \end{bmatrix} $$

[Return to task 3 of the Linear Algebra tutorial.](./index.md#exercise-3-matrix-multiplication)
  
## Exercise 4: Matrix Inversion

**Input:** An invertible $2 \times 2$ matrix $A$.

**Output:** Return the inverse of $A$, a $2 \times 2$ matrix $A^{-1}$.

**Solution:**

Since we only need to invert a $2 \times 2$ matrix, we will not consider a solution which can be used for arbitrary-sized matrices.
We will follow the algorithm described in the [Wikipedia article](https://en.wikipedia.org/wiki/Invertible_matrix#Inversion_of_2_%C3%97_2_matrices).

$$ A = \begin{bmatrix} a & b \\ c & d \end{bmatrix} $$

The determinant of the matrix is defined as

$$ |A| = a \cdot d - b \cdot c $$

$$A^{-1} = \frac{1}{|A|} \cdot \begin{bmatrix} d & -b \\ -c & a \end{bmatrix} = \begin{bmatrix} \frac{d}{|A|} & \frac{-b}{|A|} \\ \frac{-c}{|A|} & \frac{a}{|A|} \end{bmatrix} $$  
  
## Exercise 5: Transpose

**Input:** An $n \times m$ matrix $A$.

**Output:** Return an $m \times n$ matrix $A^T$, the transpose of $A$.
  
**Solution:**

Again, the tutorial gives us the definition of matrix transpose, so we just need to fill the resulting matrix with the elements of the original matrix in the right order. For example, for a $3 \times 2$ matrix:

$$\begin{bmatrix}
a & b \\
c & d \\
e & f
\end{bmatrix}^T =
\begin{bmatrix}
a & c & e \\
b & d & f
\end{bmatrix}$$
  
[Return to task 5 of the Linear Algebra tutorial.](./index.md#exercise-5-transpose)
  
## Exercise 6: Conjugate

**Input:** An $n \times m$ matrix $A$.

**Output:** Return an $n \times m$ matrix $\overline{A}$, the conjugate of $A$.

**Solution:**

To get the conjugate of a matrix you take the conjugate of each individual element. Refer to the [Complex Arithmetic tutorial](../ComplexArithmetic/ComplexArithmetic.md#Complex-Conjugate) for the definition.
 
[Return to task 6 of the Linear Algebra tutorial.](./index.md#exercise-6-conjugate)

## Exercise 7: Adjoint

**Input:** An $n \times m$ matrix $A$.

**Output:** Return an $m \times n$ matrix $A^\dagger$, the adjoint of $A$.

**Solution:**

To get the adjoint we perform both**transpose** and**conjugate** operations on the input matrix.

[Return to task 7 of the Linear Algebra tutorial.](./index.md#exercise-7-adjoint)

## Exercise 8: Unitary Verification

**Input:** An $n \times n$ matrix $A$.

**Output:** Check if the matrix is unitary and return `True` if it is, or `False` if it isn't.

**Solution:**

A matrix is unitary if this holds true:  $UU^\dagger = U^\dagger U = I$. As a reminder, an identity matrix is a matrix with 1s on the main diagonal and 0s everywhere else.

Thus, to check if the input matrix is unitary we will need to perform the following steps:

1. Calculate the adjoint of the input matrix.
2. Multiply it by the input matrix.
3. Check if the multiplication result is equal to an identity matrix.  

[Return to task 8 of the Linear Algebra tutorial.](./index.md#exercise-8-unitary-verification)

## Exercise 9: Inner product

**Inputs:**

1. An $n \times 1$ vector $V$.
2. An $n \times 1$ vector $W$.

**Output:** Return a complex number - the inner product $\langle V , W \rangle$.

**Solution:**

Following the definition of the inner product, $\langle V , W \rangle = V^\dagger W$. For example, for vectors of length 2:

$$\langle \begin{bmatrix} a \\ b \end{bmatrix} , \begin{bmatrix} c \\ d \end{bmatrix} \rangle = \begin{bmatrix} a \\ b \end{bmatrix}^\dagger \begin{bmatrix} c \\ d \end{bmatrix} = \begin{bmatrix} \overline{a} & \overline{b} \end{bmatrix} \begin{bmatrix} c \\ d \end{bmatrix} = \overline{a} \cdot c + \overline{b} \cdot d$$

We need to keep in mind that the task asks us to return a complex number and not a $1 \times 1$ matrix which is the result of the multiplication. 
Therefore at the end we'll extract the top left element of the `resultMatrix` and return it.
  
## Exercise 10: Normalized vectors

**Input:** A non-zero $n \times 1$ vector $V$.

**Output:** Return an $n \times 1$ vector $\frac{V}{||V||}$ - the normalized version of the vector $V$.

**Solution:**

If the vector $V = \begin{bmatrix}a & b & c \end{bmatrix}$, its norm $ ||V|| = \sqrt{|a|^2 + |b|^2 + |c|^2} $, and its normalized version is $ \begin{bmatrix}\frac{a}{||V||} & \frac{b}{||V||} & \frac{c}{||V||}  \end{bmatrix} $.

Thus, we need to calculate the norm of the vector and to divide each element of the vector by it. We will calculate the norm as a square root of an inner product of the vector with itself.

[Return to task 10 of the Linear Algebra tutorial.](./index.md#exercise-10-normalized-vectors)

## Exercise 11: Outer product

**Inputs:**

1. An $n \times 1$ vector $V$.
2. An $m \times 1$ vector $W$.

**Output:** Return an $n \times m$ matrix that represents the outer product of $V$ and $W$.
  
**Solution:**

By definition, the outer product of $V$ and $W$ is $VW^\dagger$. We can use a similar approach to calculating the inner product, except here we will return the whole multiplication result rather than a specific number.

[Return to task 11 of the Linear Algebra tutorial.](./index.md#exercise-11-outer-product)

## Exercise 12*: Tensor Product

**Inputs:**

1. An $n \times m$ matrix $A$.
2. A $k \times l$ matrix $B$.

**Output:** Return an $(n \cdot k) \times (m \cdot l)$ matrix $A \otimes B$, the tensor product of $A$ and $B$.

**Solution:**

We will follow the definition of the tensor product. For example, tensor product of $2 \times 2$ matrices look as follows:

$$\begin{bmatrix} a & b \\ c & d \end{bmatrix} \otimes \begin{bmatrix} e & f \\ g & h \end{bmatrix} = \begin{bmatrix} a \cdot \begin{bmatrix} e & f \\ g & h \end{bmatrix} & b \cdot \begin{bmatrix} e & f \\ g & h \end{bmatrix} \\ c \cdot \begin{bmatrix} e & f \\ g & h \end{bmatrix} & d \cdot \begin{bmatrix} e & f \\ g & h \end{bmatrix} \end{bmatrix}  = \begin{bmatrix} a \cdot e & a \cdot f & b \cdot e & b \cdot f \\ a \cdot g & a \cdot h & b \cdot g & b \cdot h \\ c \cdot e & c \cdot f & d \cdot e & d \cdot f \\ c \cdot g & c \cdot h & d \cdot g & d \cdot h \end{bmatrix} $$
 
[Return to task 12 of the Linear Algebra tutorial.](./index.md#exercise-12-tensor-product)
  
## Exercise 13: Finding an eigenvalue

**Inputs:**

1. A real-valued $n \times n$ matrix $A$.
2. An eigenvector $V$ of matrix $A$.

**Output:** Return a real number - the eigenvalue of $A$ that is associated with the given eigenvector.

**Solution:**

Let's consider what happens when we multiply the matrix by its eigenvector for a $3 \times 3$ example:

$$ A \cdot V = \begin{bmatrix} a & b & c \\ d & e & f \\ g & h & i \end{bmatrix} \cdot \begin{bmatrix}j \\ k \\ l \end{bmatrix} = \begin{bmatrix} m \\ n \\ o \end{bmatrix} = \alpha \begin{bmatrix}j \\ k \\ l \end{bmatrix} = \alpha V$$

This means you can find the eigenvalue $\alpha$ from the equations

$$ \begin{cases} \alpha j = m \\ \alpha k = n \\ \alpha l = o \end{cases}$$

We can use any of them, keeping in mind that we need an equation in which the element of the eigenvector is not zero (otherwise we get an equation $0 \alpha = 0$ which doesn't help us find $\alpha$).

Since eigenvectors are defined as non-zero vectors, we are guaranteed that at least one element of the vector will not be zero.
  
[Return to task 13 of the Linear Algebra tutorial.](/index.md#exercise-13-finding-an-eigenvalue)
  
## Exercise 14**: Finding an eigenvector

**Inputs:**

1. A $2 \times 2$ matrix $A$.
2. An eigenvalue $x$ of matrix $A$.

**Output:** Return any non-zero eigenvector of $A$ that is associated with $x$.

**Solution:**

Searching for an eigenvector $V$ associated with a specific eigenvalue $x$ asks for solving the following equation:

$$ AV = xV $$

or, equivalently, $$(A - xI_n)V = 0$$

In other words, for a $2 \times 2$ matrix the following happens:

1. Multiply the identity matrix $I_2$ by the eigenvalue:
$$ x \cdot \begin{bmatrix} 1 & 0 \\ 0 & 1 \end{bmatrix} = \begin{bmatrix} x & 0 \\ 0 & x \end{bmatrix} $$

2. Subtract this new matrix from the given matrix $A$:
$$ \begin{bmatrix} a & b \\ c & d \end{bmatrix} - \begin{bmatrix} x & 0 \\ 0 & x \end{bmatrix} = \begin{bmatrix} a -x & b \\ c & d -x \end{bmatrix} $$

3. Find a vector that, when multiplied by the resulting matrix, will produce a 0 vector:
$$ \begin{bmatrix} a - x & b \\ c & d - x \end{bmatrix} \cdot \begin{bmatrix} v_0 \\ v_1 \end{bmatrix} = \begin{bmatrix} 0 \\ 0 \end{bmatrix}$$

This can be rewritten as the following system of equations:

$$\begin{cases}
(a - x) \cdot v_0 + b \cdot v_1 = 0  \\
c \cdot v_0 + (d - x) \cdot v_1 = 0  
\end{cases}$$

Each eigenvalue has infinitely many eigenvectors associated with it (since multiplying an eigenvector by a number gives another valid eigenvector). We can limit our search and say that $v_0 = 1$, if possible. In this case, the system of equations becomes

$$\begin{cases}
(a - x) + b \cdot v_1 = 0  \\
c + (d - x) \cdot v_1 = 0  
\end{cases}$$

and finally we get $v_1 = \frac{a-x}{-b}$.

If $b = 0$, we can not perform this division, so we need to reconsider our choices. The first equation becomes $(a-x)v_0 = 0$, which is possible in two cases:

* If $a - x \neq 0$, we get $v_0 = 0$ and thus $v_1$ has to be non-zero (we can pick $v_1 = 1$).
* If $a - x = 0$, we can not get any information from the first equation and have to fall back to the second one:
$c \cdot v_0 + (d - x) \cdot v_1 = 0$. Following a similar logic:
* If $c = 0$, we get $(d - x) \cdot v_1 = 0$, so $v_0 = 1, v_1 = 0$.
* If $c \neq 0$, we get $v_1 = 1, v_0 = \frac{d-x}{-c}$.
  
 [Return to task 14 of the Linear Algebra tutorial.](./index.md#exercise-14-finding-an-eigenvector)
