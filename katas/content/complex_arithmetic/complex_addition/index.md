**Inputs:**

1. A complex number $x = a + bi$.
2. A complex number $y = c + di$.

**Goal:**
Return the sum of x and y. That is $x + y = z = g + hi$.

Revise the following Q# code to return the sum of x and y.

<details>
  <summary><b>Need a hint?</b></summary>
  
* Use the Complex data type defined in the Q# math library. For example, $x = a + bi$:

   ```qsharp

      let x = complex(real_value, imaginary_coefficient);
      let a = x::Real;
      let b = x::Imag;
   ```

* Remember, adding complex numbers is just like adding polynomials. Add components of the same type - add the real part to the real part, add the imaginary part to the imaginary part.

$$ z = x + y = (a + bi) + (c + di) = \underset{real}{\underbrace{(a + c)}} + \underset{imaginary}{\underbrace{(b + d)}}i $$

A video explanation of adding complex numbers can be found [here](https://www.youtube.com/watch?v=SfbjqVyQljk).
</details>
