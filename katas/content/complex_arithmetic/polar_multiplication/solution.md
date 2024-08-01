Multiplying two complex numbers in polar form can be done efficiently in the following way:

$$ z = x \cdot y = r_{1}e^{\theta_1 i} \cdot r_{2}e^{\theta_2 i} = r_{1}r_{2} \cdot e^{(\theta_1 + \theta_2)i} $$  

> Here is the longer approach of converting the numbers to the Cartesian from and doing multiplication in it:  
> $$ x = r_{1}e^{i\theta_1} = r_{1}(\cos \theta_1 + i \sin \theta_1) $$
> $$ y = r_{2}e^{i\theta_2} = r_{2}(\cos \theta_2 + i \sin \theta_2) $$
> $$ z = x \cdot y = r_1r_2 \cdot \left( \cos \theta_1 \cos \theta_2 − \sin \theta_1 \sin \theta_2 + 
i (\sin \theta_1 \cos \theta_2 + \sin \theta_2 \cos \theta_1 ) \right) $$
>
> You can simplify this using the following trigonometric identities:
> * $\cos a \cos b  \mp \sin a \sin b = \cos(a \pm b)$
> * $\sin a \cos b  \pm \sin b \cos a = \sin(a \pm b)$
>
> Finally, this solution gives the same answer as the short solution above:
>$$r_{1}r_{2}(\cos(\theta_1 + \theta_2) + i \sin(\theta_1 + \theta_2)) = r_{1}r_{2} \cdot e^{(\theta_1 + \theta_2)i} $$

@[solution]({"id": "complex_arithmetic__polar_multiplication_solution", "codePath": "Solution.qs"})
