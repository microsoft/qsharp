You can follow the algorithm described in the [Wikipedia article](https://en.wikipedia.org/wiki/Invertible_matrix#Inversion_of_2_%C3%97_2_matrices) for $2 \times 2$ matrices:

$$ A = \begin{bmatrix} a & b \\ c & d \end{bmatrix} $$

Then the determinant of the matrix is defined as 
$$ |A| = a \cdot d - b \cdot c = 1 \cdot 4 - 2 \cdot 3 = -2$$

And the inverse of the matrix is

$$A^{-1} = \frac{1}{|A|} \cdot \begin{bmatrix} d & -b \\ -c & a \end{bmatrix} = -\frac12 \begin{bmatrix} 4 & -2 \\ -3 & 1 \end{bmatrix} = \begin{bmatrix} -2 & 1 \\ \frac32 & -\frac12 \end{bmatrix}$$

@[solution]({"id": "linear_algebra__inverse_solution", "codePath": "Solution.qs"})
