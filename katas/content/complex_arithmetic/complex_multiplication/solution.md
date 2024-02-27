**Inputs:**

1. A complex number $x = a + bi$.
2. A complex number $y = c + di$.

**Goal:** Return the product of x and y. That is $x * y = z = g + hi$.

**Solution:**

Multiplying complex numbers is like multiplying polynomials, therefore the same rules apply.

Multiplying complex numbers is just like multiplying polynomials. Distribute one of the complex numbers. Then multiply through, and group the real and imaginary terms together. **Remember** $i^2 =-1$:

$z = x \cdot y = (a + bi)(c + di) = a \cdot c + a \cdot di + c \cdot bi + bi \cdot di = \underset{real}{\underbrace{a \cdot c - b \cdot d}} + \underset{imaginary}{\underbrace{(a \cdot d + c \cdot b)}}i $

@[solution]({"id": "complex_arithmetic__complex_multiplication_solution", "codePath": "Solution.qs"})
