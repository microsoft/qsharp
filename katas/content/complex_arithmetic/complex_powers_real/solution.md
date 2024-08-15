Rewrite the expression $r^x$ to use Euler's constant, which will allow you to use an approach similar to the solution to the previous exercise.

First, rewrite $r^x$ into a product of two powers: $$ r^{a+bi} = r^a \cdot r^{bi} $$

Given that $r = e^{\ln r} $ ($\ln$ is the natural logarithm), you can rewrite the second part of the product as follows: 
    $$ r^{bi} =  e^{bi\ln r} $$

Now, given $e^{i\theta} = \cos \theta + i\sin \theta$, you can rewrite it further as follows: 
    $$ e^{bi\ln r} = \cos( b \cdot \ln r) + i \sin(b \cdot \ln r) $$

When substituting this into the original expression, you get:
    $$ \underset{real}{\underbrace{r^a \cos(b \cdot \ln r)}} + \underset{imaginary}{\underbrace{r^a \sin(b \cdot \ln r)}} i $$

@[solution]({"id": "complex_arithmetic__complex_powers_real_solution", "codePath": "Solution.qs"})
