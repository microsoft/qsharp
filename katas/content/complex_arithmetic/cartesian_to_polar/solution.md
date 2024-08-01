You need to calculate the $r$ and $\theta$ values as seen in the complex plane. 
$r$ should be familiar to you already, since it is the modulus of a number (exercise 6):

$$ r = \sqrt{a^2 + b^2} $$

$\theta$ can be calculated using trigonometry: since you know that the polar and the Cartesian forms of the number represent the same value, you can write

$$ re^{i \theta} = a + bi $$

Euler's formula allows us to express the left part of the equation as 

$$ re^{i \theta} = r \cos \theta + i r \sin \theta $$

For two complex numbers to be equal, their real and imaginary parts have to be equal. This gives you the following system of equations:

$$ \begin{cases} a = r \cos \theta \\ b = r \sin \theta \end{cases} $$

To calculate $\theta$, you can divide the second equation by the first one to get

$$ \tan \theta = \frac{b}{a} $$

$$ \theta = \arctan \left(\frac{b}{a}\right) $$

@[solution]({"id": "complex_arithmetic__cartesian_to_polar_solution", "codePath": "Solution.qs"})
