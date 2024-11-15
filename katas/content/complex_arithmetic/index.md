# Complex Arithmetic

@[section]({
    "id": "complex_arithmetic__overview",
    "title": "Overview"
})

This kata introduces you to complex arithmetic. This topic isn't particularly expansive, but it's important to understand it to be able to work with quantum computing.

**This kata covers the following topics:**

* Imaginary and complex numbers
* Basic complex arithmetic
* Complex plane
* Modulus operator
* Imaginary exponents
* Polar representation

If you are curious to learn more, you can find more information at [Wikipedia](https://en.wikipedia.org/wiki/Complex_number).

This kata has several tasks that require you to write Q# code to test your understanding of the concepts. The kata will introduce the necessary Q# language constructs as it goes.

@[section]({
    "id": "complex_arithmetic__algebraic_perspective",
    "title": "Imaginary Numbers"
})

For some purposes, real numbers aren't enough. Probably the most famous example is the equation:

$$x^{2} = -1$$

This equation has no solution among real numbers. But if you abandon that constraint, you can do something interesting - you can define your own number. Let's say there exists some number that solves that equation. Let's call that number $i$.

$$i^2 = -1$$

As said before, the number $i$ can't be a real number. In that case, let's call it an **imaginary number**. However, there's no reason to define it as acting any different from any other number, other than the fact that $i^2 = -1$. This means that you can do all the same operations on $i$ that you can do on any other number. For example:

$$i+i=2i$$

$$i-i=0$$

$$-1 \cdot i=-i$$

$$(-i)^{2} = -1$$

The number $i$ and its real multiples (numbers obtained by multiplying $i$ by a real number) are called **imaginary numbers**.

> A good video introduction to imaginary numbers can be found [here](https://youtu.be/SP-YJe7Vldo).

@[exercise]({
    "id": "complex_arithmetic__powers_of_i",
    "title": "Powers of Imaginary Unit",
    "path": "./powers_of_i/"
})

@[section]({
    "id": "complex_arithmetic__complex_numbers",
    "title": "Complex Numbers"
})

Adding imaginary numbers to each other is quite simple, but what happens when you add a real number to an imaginary number? The result of that addition will be partly real and partly imaginary, otherwise known as a **complex number**. A complex number is simply the real part and the imaginary part being treated as a single number. Complex numbers are generally written as the sum of their two parts $a$ and $bi$, where both $a$ and $b$ are real numbers: 

$$a+bi$$

For example, $3+4i$ or $-5-7i$ are valid complex numbers. Note that purely real or purely imaginary numbers can also be written as complex numbers: $2$ is $2+0i$, and $-3i$ is $0-3i$.

When performing operations on complex numbers, it's often helpful to treat them as polynomials in terms of $i$.
Let's see how to do the main arithmetic operations on complex numbers.

> In Q#, complex numbers are represented as user-defined struct type `Complex` from the `Std.Math` namespace.
>
> Given a complex number $x = a + bi$, you can access its real and imaginary parts using their names: `let (a, b) = (x.Real, x.Imag);`.
>
> You can construct a complex number from its real and imaginary parts as follows: `let x = Complex(a, b);`.

@[exercise]({
    "id": "complex_arithmetic__complex_addition",
    "title": "Add Complex Numbers",
    "path": "./complex_addition/"
})

@[exercise]({
    "id": "complex_arithmetic__complex_multiplication",
    "title": "Multiply Complex Numbers",
    "path": "./complex_multiplication/"
})

@[section]({
    "id": "complex_arithmetic__complex_conjugate",
    "title": "Complex Conjugate"
})

Before discussing any other operations on complex numbers, let's review the **complex conjugate**. The conjugate is a simple operation: given a complex number  $x = a + bi$, its complex conjugate is $\overline{x} = a - bi$.

The conjugate allows you to do some interesting things. The first and probably most important is multiplying a complex number by its conjugate:

$$x \cdot \overline{x} = (a + bi)(a - bi)$$

Notice that the second expression is a difference of squares:

$$(a + bi)(a - bi) = a^2 - (bi)^2 = a^2 - b^2i^2 = a^2 + b^2$$

This means that a complex number multiplied by its conjugate always produces a non-negative real number.

Another property of the conjugate is that it distributes over both complex addition and complex multiplication:

$$\overline{x + y} = \overline{x} + \overline{y}$$
$$\overline{x \cdot y} = \overline{x} \cdot \overline{y}$$

@[exercise]({
    "id": "complex_arithmetic__complex_conjugate_exercise",
    "title": "Find Conjugate",
    "path": "./complex_conjugate/"
})

@[section]({
    "id": "complex_arithmetic__complex_division",
    "title": "Complex Division"
})

The next use for the conjugate is complex division. Let's take two complex numbers: $x = a + bi$ and $y = c + di \neq 0$ (not even complex numbers let you divide by $0$). What does $\frac{x}{y}$ mean?

Let's expand $x$ and $y$ into their component forms:

$$\frac{x}{y} = \frac{a + bi}{c + di}$$

Unfortunately, it isn't very clear what it means to divide by a complex number. You need some way to move either all real parts or all imaginary parts into the numerator. And thanks to the conjugate, you can do just that. Using the fact that any number (except $0$) divided by itself equals $1$, and any number multiplied by $1$ equals itself, you get:

$$\frac{x}{y} = \frac{x}{y} \cdot 1 = \frac{x}{y} \cdot \frac{\overline{y}}{\overline{y}} = \frac{x\overline{y}}{y\overline{y}} = \frac{(a + bi)(c - di)}{(c + di)(c - di)} = \frac{(a + bi)(c - di)}{c^2 + d^2}$$

By doing this, you re-wrote your division problem to have a complex multiplication expression in the numerator, and a real number in the denominator. You already know how to multiply complex numbers, and dividing a complex number by a real number is as simple as dividing both parts of the complex number separately:

$$\frac{a + bi}{r} = \frac{a}{r} + \frac{b}{r}i$$

@[exercise]({
    "id": "complex_arithmetic__complex_division_exercise",
    "title": "Divide Complex Numbers",
    "path": "./complex_division/"
})

@[section]({
    "id": "complex_arithmetic__geometric_perspective",
    "title": "Geometric Perspective: the Complex Plane"
})

You may recall that real numbers can be represented geometrically using the number line - a line on which each point represents a real number. You can extend this representation to include imaginary and complex numbers, which gives rise to an entirely different number line: the imaginary number line, which is orthogonal to the real number line and only intersects with it at $0$.

A complex number has two components - a real component and an imaginary component. As you no doubt noticed from the exercises, these can be represented by two real numbers - the real component, and the real coefficient of the imaginary component. This allows you to map complex numbers onto a two-dimensional plane - the **complex plane**. The most common mapping is the obvious one: $a+bi$ can be represented by the point $(a,b)$ in the **Cartesian coordinate system**.

This mapping allows you to apply complex arithmetic to geometry, and, more importantly, apply geometric concepts to complex numbers. Many properties of complex numbers become easier to understand when viewed through a geometric lens.

## Modulus

One such property is the **modulus operator**. This operator generalizes the **absolute value** operator on real numbers to the complex plane. Just like the absolute value of a number is its distance from $0$, the modulus of a complex number is its distance from $0+0i$. Using the distance formula, if $x=a+bi$, then:

$$|x| = \sqrt{a^2 + b^2}$$

There is also a slightly different, but algebraically equivalent definition:

$$|x| = \sqrt{x \cdot \overline{x}}$$

Like the conjugate, the modulus distributes over multiplication.

$$|x \cdot y| = |x| \cdot |y|$$

Unlike the conjugate, however, the modulus doesn't distribute over addition. Instead, the interaction of the two comes from the triangle inequality:

$$|x + y| \leq |x| + |y|$$

@[exercise]({
    "id": "complex_arithmetic__complex_modulus_exercise",
    "title": "Find Modulus",
    "path": "./complex_modulus/"
})

@[section]({
    "id": "complex_arithmetic__imaginary_exponents",
    "title": "Imaginary Exponents"
})

The next complex operation is **exponentiation**. Raising an imaginary number to an integer power is a fairly simple task, but raising a number to an imaginary power, or raising an imaginary (or complex) number to a real power isn't quite as simple.

Let's start with raising real numbers to imaginary powers. Specifically, let's start with a rather special real number - Euler's constant, $e$:

$$e^{i\theta} = \cos \theta + i\sin \theta$$

Here and later in this tutorial $\theta$ is measured in radians.

> Explaining why that happens is beyond the scope of this tutorial. If you are curious, you can see [this video](https://youtu.be/v0YEaeIClKY) for a beautiful intuitive explanation, or [this Wikipedia article](https://en.wikipedia.org/wiki/Complex_number) for a more mathematically rigorous proof.

Here are some examples of this formula in action:

$$e^{i\pi/4} = \frac{1}{\sqrt{2}} + \frac{i}{\sqrt{2}}$$
$$e^{i\pi/2} = i$$
$$e^{i\pi} = -1$$
$$e^{2i\pi} = 1$$

> One interesting consequence of this is Euler's identity:
>
> $$e^{i\pi} + 1 = 0$$
> 
> While this doesn't have any notable uses, it's still an interesting identity to consider, as it combines five fundamental constants of algebra into one expression.

You can also calculate complex powers of $e$ as follows:

$$e^{a + bi} = e^a \cdot e^{bi}$$

Finally, using logarithms to express the base of the exponent as $r = e^{\ln r}$, you can use this to find complex powers of any positive real number.

@[exercise]({
    "id": "complex_arithmetic__complex_exponents_exercise",
    "title": "Find Complex Exponent",
    "path": "./complex_exponents/"
})

@[exercise]({
    "id": "complex_arithmetic__complex_powers_real_exercise",
    "title": "Find Complex Power of Real Number",
    "path": "./complex_powers_real/"
})

@[section]({
    "id": "complex_arithmetic__polar_coordinates",
    "title": "Polar Coordinates"
})

Consider the expression  $e^{i\theta} = \cos\theta + i\sin\theta$. Notice that if you map this number onto the complex plane, it'll land on a **unit circle** around $0 + 0i$. This means that its modulus is always $1$. You can also verify this algebraically: $\cos^2\theta + \sin^2\theta = 1$.

Using this fact you can represent complex numbers using **polar coordinates**. In a polar coordinate system, a point is represented by two numbers: its direction from origin, represented by an angle from the $x$ axis, and how far away it is in that direction.

Another way to think about this is that you're taking a point that is $1$ unit away (which is on the unit circle) in the specified direction, and multiplying it by the desired distance. And to get the point on the unit circle, you can use $e^{i\theta}$.

A complex number of the format $r \cdot e^{i\theta}$ will be represented by a point which is $r$ units away from the origin, in the direction specified by the angle $\theta$.
Sometimes $\theta$ will be referred to as the number's **argument** or **phase**.

> In Q#, complex numbers in polar form are represented as user-defined struct type `ComplexPolar` from the `Std.Math` namespace.
>
> Given a complex number $x = r \cdot e^{i\theta}$, you can access its magnitude and phase using their names: `let r = x.Magnitude;` and `let theta = x.Argument;`.
>
> You can construct a complex number from its magnitude and phase as follows: `let x = ComplexPolar(r, theta);`.

@[exercise]({
    "id": "complex_arithmetic__cartesian_to_polar",
    "title": "Convert Cartesian to Polar",
    "path": "./cartesian_to_polar/"
})

@[exercise]({
    "id": "complex_arithmetic__polar_to_cartesian",
    "title": "Convert Polar to Cartesian",
    "path": "./polar_to_cartesian/"
})

@[exercise]({
    "id": "complex_arithmetic__polar_multiplication",
    "title": "Multiply Polar Numbers",
    "path": "./polar_multiplication/"
})

@[section]({
    "id": "complex_arithmetic__conclusion",
    "title": "Conclusion"
})

Congratulations! You should now know enough complex arithmetic to get started with quantum computing!
