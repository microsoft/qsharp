# Complex Arithmetic Tutorial Workbook

**What is this workbook?**
A workbook is a collection of problems, accompanied by solutions to them. 
The explanations focus on the logical steps required to solve a problem; they illustrate the concepts that need to be applied to come up with a solution to the problem, explaining the mathematical steps required. 

Note that a workbook should not be the primary source of knowledge on the subject matter; it assumes that you've already read a tutorial or a textbook and that you are now seeking to improve your problem-solving skills. You should attempt solving the tasks of the respective kata first, and turn to the workbook only if stuck. While a textbook emphasizes knowledge acquisition, a workbook emphasizes skill acquisition.

This workbook describes the solutions to the problems offered in the [Complex Arithmetic tutorial](./index.md).

**What you should know for this workbook:**

1. Basic math.

## Exercise 1: Powers of i

**Input:** An even integer $n$.

**Goal:** Return the $n$ th power of $i$, or $i^n$.

**Solution:**

When raising $i$ to an integer power, the answer will vary according to a certain pattern.
To figure it out, notice that raising $i$ to the power of 4 gives: $i^4 = i^2 \cdot i^2 = (-1) \cdot (-1) = 1$. 
Thus, when the power $n$ is divisible by 4, $i^n$ will always be 1.

When the power $n$ is not divisible by 4, you can use the previous observation to see that $i^n = i^{n \mod 4}$. 
For an even power $n$ that is not divisible by 4 you'll have $i^n = i^2 = -1.$

Here is the complete pattern that arises when raising $i$ to non-negative powers. Note that it is periodic with period 4.

|Power of $i$ | $i^0$ | $i^1$ | $i^2$ | $i^3$ | $i^4$ | $i^5$ | $i^6$ | $i^7$ | $i^8$ | $\dots$ |
|----|----|----|----|----|----|----|----|----|----|----|
|Result | $1$ | $i$ | $-1$ | $-i$ | $1$ | $i$ | $-1$ | $-i$ | $1$ | $\dots$ |

[Return to exercise 1 of the Complex Arithmetic tutorial.](./index.md#exercise-1-powers-of-i)

## Exercise 2: Complex addition

**Inputs:**

1. A complex number $x = a + bi$, represented as a tuple `(a, b)`.
2. A complex number $y = c + di$, represented as a tuple `(c, d)`.

**Goal:** Return the sum of these two numbers $x + y = z = g + hi$, represented as a tuple `(g, h)`.

**Solution:**

Adding two complex numbers can be done by separately adding the real parts of the numbers and the imaginary parts:  

$$ z = x + y = (a + bi) + (c + di) = \underset{real}{\underbrace{(a + c)}} + \underset{imaginary}{\underbrace{(b + d)}}i $$

[Return to exercise 2 of the Complex Arithmetic tutorial.](./index.md#exercise-2-complex-addition)

## Exercise 3: Complex multiplication

**Inputs:**

1. A complex number $x = a + bi$, represented as a tuple `(a, b)`.
2. A complex number $y = c + di$, represented as a tuple `(c, d)`.

**Goal:** Return the product of these two numbers $x \cdot y = z = g + hi$, represented as a tuple `(g, h)`.

**Solution:**

Multiplying complex numbers is like multiplying polynomials, therefore the same rules apply. **Remember** $i^2 =-1$.  

$$z = x \cdot y = (a + bi)(c + di) = a \cdot c + a \cdot di + c \cdot bi + bi \cdot di = \underset{real}{\underbrace{a \cdot c - b \cdot d}} + \underset{imaginary}{\underbrace{(a \cdot d + c \cdot b)}}i $$

[Return to exercise 3 of the Complex Arithmetic tutorial.](./index.md#exercise-3-complex-multiplication)

## Exercise 4: Complex conjugate

**Input:** A complex number $x = a + bi$, represented as a tuple `(a, b)`.

**Goal:** Return $\overline{x} = g + hi$, the complex conjugate of $x$, represented as a tuple `(g, h)`.

**Solution:**

To get the complex conjugate of a complex number you change the sign of the imaginary part of the complex number:  

$$\overline{x} = a - bi$$

[Return to exercise 4 of the Complex Arithmetic tutorial.](./index.md#exercise-4-complex-conjugate)

## Exercise 5: Complex division

**Inputs:**

1. A complex number $x = a + bi$, represented as a tuple `(a, b)`.
2. A complex number $y = c + di \neq 0$, represented as a tuple `(c, d)`.

**Goal:** Return the result of the division $\frac{x}{y} = \frac{a + bi}{c + di} = g + hi$, represented as a tuple `(g, h)`.

**Solution:**

$$z = \frac{x}{y} = \frac{x}{y} \cdot 1 = \frac{x}{y} \cdot \frac{\overline{y}}{\overline{y}} = 
\frac{x\overline{y}}{y\overline{y}} = \frac{(a + bi)(c - di)}{(c + di)(c - di)} = 
\frac{a \cdot c + bi \cdot c - a \cdot di - bi \cdot di}{c \cdot c + di \cdot c - c \cdot di - di \cdot di} = 
\frac{a \cdot c + b \cdot d + (a \cdot (-d) + c \cdot b)i}{c^2 + d^2}$$

[Return to exercise 5 of the Complex Arithmetic tutorial.](./index.md#exercise-5-complex-division)

## Exercise 6: Modulus

**Input:** A complex number $x = a + bi$, represented as a tuple `(a, b)`.

**Goal:** Return the modulus of this number, $|x|$.

**Solution:**

The modulus of the complex number can be seen as the distance from the origin 0 to the $z$ point in the complex plane. This can be calculated using the Pythagorean theorem: $c^2 = a^2 + b^2$, which means $ c = \sqrt{a^2 + b^2} $.

<img src="img/pythagorean_theorem.png" style="max-height: 350px;">

[Return to exercise 6 of the Complex Arithmetic tutorial.](./index.md#exercise-6-modulus)

## Exercise 7: Complex exponents

**Input:** A complex number $x = a + bi$, represented as a tuple `(a, b)`.

**Goal:** Return the complex number $e^x = e^{a + bi} = g + hi$, represented as a tuple `(g, h)`.

**Solution:**

To start, we will rewrite the expression $e^{a + bi}$ as a product of two simpler expressions: $ e^a \cdot\ e^{bi} $.
The first part is a real number. 
The second part can be expressed using the formula $e^{i\theta} = \cos \theta + i\sin \theta$.  
Substituting this into our expression gives:
$$ e^a(\cos b + i\sin b) = \underset{real}{\underbrace{e^a \cos b}} + \underset{imaginary}{\underbrace{e^a \sin b}}i  $$

[Return to exercise 7 of the Complex Arithmetic tutorial.](./index.md#exercise-7-complex-exponents)

## Exercise 8*: Complex powers of real numbers

**Inputs:**

1. A non-negative real number $r$.
2. A complex number $x = a + bi$, represented as a tuple `(a, b)`.

**Goal:** Return the complex number $r^x = r^{a + bi} = g + hi$, represented as a tuple `(g, h)`.

**Solution:**

Let's rewrite the expression $r^x$ to use Euler's constant, which will allow us to use an approach similar to the solution to the previous exercise.

First, we rewrite $r^x$ into a product of two powers: $$ r^{a+bi} = r^a \cdot r^{bi} $$

Given that $r = e^{\ln r} $ ($\ln$ is the natural logarithm), we can rewrite the second part of the product as follows:
$$ r^{bi} =  e^{bi\ln r} $$

Now, given $e^{i\theta} = \cos \theta + i\sin \theta$, we can rewrite it further as follows: 
$$ e^{bi\ln r} = \cos( b \cdot \ln r) + i \sin(b \cdot \ln r) $$

When substituting this into our original expression, we get:  
$$ \underset{real}{\underbrace{r^a \cos(b \cdot \ln r)}} + \underset{imaginary}{\underbrace{r^a \sin(b \cdot \ln r)}} i $$

[Return to exercise 8 of the Complex Arithmetic tutorial.](./index.md#exercise-8-complex-powers-of-real-numbers)

## Exercise 9: Cartesian to polar conversion

**Input:** A complex number $x = a + bi$, represented as a tuple `(a, b)`.

**Goal:** Return the polar representation of $x = re^{i\theta}$, i.e., the distance from origin $r$ and phase $\theta$ as a tuple `(r, θ)`.

* $r$ should be non-negative: $r \geq 0$
* $\theta$ should be between $-\pi$ and $\pi$: $-\pi < \theta \leq \pi$

**Solution:**

We need to calculate the $r$ and $\theta$ values as seen in the complex plane.

$r$ should be familiar to you already, since it is the modulus of a number (exercise 6):

$$ r = \sqrt{a^2 + b^2} $$

$\theta$ can be calculated using trigonometry: since we know that the polar and the Cartesian forms of the number represent the same value, we can write

$$ re^{i \theta} = a + bi $$

Euler's formula allows us to express the left part of the equation as 

$$ re^{i \theta} = r \cos \theta + i r \sin \theta $$

For two complex numbers to be equal, their real and imaginary parts have to be equal. This gives us the following system of equations:

$$ \begin{cases} a = r \cos \theta \\ b = r \sin \theta \end{cases} $$

To calculate $\theta$, we can divide the second equation by the first one to get

$$ \tan \theta = \frac{b}{a} $$

$$ \theta = \arctan \left(\frac{b}{a}\right) $$

[Return to exercise 9 of the Complex Arithmetic tutorial.](./index.md#exercise-9-cartesian-to-polar-conversion)

## Exercise 10: Polar to Cartesian conversion

**Input:** A complex number $x = re^{i\theta}$, represented in polar form as a tuple `(r, θ)`.

**Goal:** Return the Cartesian representation of $x = a + bi$, represented as a tuple `(a, b)`.

<img src="img/Polar_to_Cartesian.png" style="max-height:350px;">

**Solution:**

Using the trigonometric functions of the right triangle, you can get the following expressions (which you've obtained algebraically in the previous exercise):

$$ \begin{cases} a = r \cos \theta \\ b = r \sin \theta \end{cases} $$

[Return to exercise 10 of the Complex Arithmetic tutorial.](./index.md#exercise-10-polar-to-cartesian-conversion)

## Exercise 11: Polar multiplication

**Inputs:**

1. A complex number $x = r_{1}e^{i\theta_1}$ represented in polar form as a tuple `(r1, θ1)`.
2. A complex number $y = r_{2}e^{i\theta_2}$ represented in polar form as a tuple `(r2, θ2)`.

**Goal:** Return the result of the multiplication $x \cdot y = z = r_3e^{i\theta_3}$, represented in polar form as a tuple `(r3, θ3)`.

* $r_3$ should be non-negative: $r_3 \geq 0$
* $\theta_3$ should be between $-\pi$ and $\pi$: $-\pi < \theta_3 \leq \pi$
* Try to avoid converting the numbers into Cartesian form.

**Solution:**

Multiplying two complex numbers in polar form can be done efficiently in the following way:

$$ z = x \cdot y = r_{1}e^{\theta_1 i} \cdot r_{2}e^{\theta_2 i} = r_{1}r_{2} \cdot e^{(\theta_1 + \theta_2)i} $$  

Here is the longer approach of converting the numbers to the Cartesian from and doing multiplication in it:
  
$$ x = r_{1}e^{i\theta_1} = r_{1}(\cos \theta_1 + i \sin \theta_1) $$
$$ y = r_{2}e^{i\theta_2} = r_{2}(\cos \theta_2 + i \sin \theta_2)$$
$$ z = x \cdot y = r_1r_2 \cdot \left( \cos \theta_1 \cos \theta_2 − \sin \theta_1 \sin \theta_2 + i (\sin \theta_1 \cos \theta_2 + \sin \theta_2 \cos \theta_1 ) \right) $$

We can simplify this using the following trigonometric identities:
* $\cos a \cos b  \mp \sin a \sin b = \cos(a \pm b)$
* $\sin a \cos b  \pm \sin b \cos a = \sin(a \pm b)$

Finally, this solution gives the same answer as the short solution above:
$$z = r_{1}r_{2}(\cos(\theta_1 + \theta_2) + i \sin(\theta_1 + \theta_2)) = r_{1}r_{2} \cdot e^{(\theta_1 + \theta_2)i} $$

[Return to exercise 11 of the Complex Arithmetic tutorial.](./index.md#exercise-11-polar-multiplication)

## Exercise 12**: Arbitrary complex exponents

**Inputs:**

1. A complex number $x = a + bi$, represented as a tuple `(a, b)`.
2. A complex number $y = c + di$, represented as a tuple `(c, d)`.

**Goal:** Return the result of raising $x$ to the power of $y$: $x^y = (a + bi)^{c + di} = z = g + hi$, represented as a tuple `(g, h)`.

**Solution:**

Let's convert the number $x$ to polar form $x = re^{i\theta}$ and rewrite the complex exponent as follows:

$$ x^y = \left( re^{i\theta} \right)^{c + di} = e^{(\ln(r) + i\theta)(c + di)} = $$
$$ = e^{\ln(r) \cdot c \, + \, \ln(r) \cdot di \, + \, i\theta \cdot c \, + \, (d\theta)i^2} = 
e^{\left(\ln(r) \cdot c \, - \, d\theta \right) \, + \, \left(\ln(r) \cdot d \, + \, \theta c\right)i} $$

Finally, this needs to be converted back to Cartesian form using Euler's formula:

$$ e^{\ln(r) \cdot c - d\theta} \cdot (\cos (\ln(r) \cdot d + \theta c) + i\sin (\ln(r) \cdot d + \theta c)) $$

[Return to exercise 12 of the Complex Arithmetic tutorial.](./index.md#exercise-12-arbitrary-complex-exponents)
