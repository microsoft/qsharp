To start, you'll rewrite the expression $e^{a + bi}$ as a product of two simpler expressions: $ e^a \cdot\ e^{bi} $.
The first part is a real number.
The second part can be expressed using the formula $e^{i\theta} = \cos \theta + i\sin \theta$.
Substituting this into the expression gives:
$$ e^a(\cos b + i\sin b) = \underset{real}{\underbrace{e^a \cos b}} + \underset{imaginary}{\underbrace{e^a \sin b}}i  $$

@[solution]({"id": "complex_arithmetic__complex_exponents_solution", "codePath": "Solution.qs"})
