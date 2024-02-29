**Inputs:**

1. A complex number $x = a + bi$.

**Goal:**
Return the complex conjugate of $x$. That is, $\overline{x} = a - bi$.

Revise the following Q# code to return the complex conjugate of x.

<details>
  <summary><b>Need a hint?</b></summary>
  
* Use the Complex data type defined in the Q# math library. For example, $x = a + bi$:

   ```qsharp

      let x = complex(real_value, imaginary_coefficient);
      let a = x::Real;
      let b = x::Imag;
   ```

A video explanation of the complex conjugate can be found [here](https://www.youtube.com/watch?v=BZxZ_eEuJBM).

</details>
