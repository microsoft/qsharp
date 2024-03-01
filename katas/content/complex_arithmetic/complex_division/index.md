
**Inputs:**

1. A complex number $x = a + bi$.
2. A complex number $y = c + di \neq 0$.

**Goal:** Return the result of the division $\frac{x}{y} = \frac{a + bi}{c + di} = g + hi$.

<details>
  <summary><b>Need a hint?</b></summary>
  
* Use the Complex data type defined in the Q# math library. For example, $x = a + bi$:

   ```qsharp

      let x = complex(real_value, imaginary_coefficient);
      let a = x::Real;
      let b = x::Imag;
   ```

A video explanation of complex division can be found [here](https://www.youtube.com/watch?v=Z8j5RDOibV4)

</details>
