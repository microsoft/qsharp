**Inputs:**

1. A complex number $x = a + bi$.
2. A complex number $y = c + di$.

**Goal:** Return the sum of these two numbers $x + y = z = g + hi$.

**Solution:**

Adding two complex numbers can be done by separately adding the real parts of the numbers and the imaginary parts:  

$$ z = x + y = (a + bi) + (c + di) = \underset{real}{\underbrace{(a + c)}} + \underset{imaginary}{\underbrace{(b + d)}}i $$

@[solution]({"id": "complex_arithmetic__complex_addition_solution", "codePath": "Solution.qs"})
