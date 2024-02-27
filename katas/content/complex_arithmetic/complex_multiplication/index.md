**Inputs:**

1. A complex number $x = a + bi$.
2. A complex number $y = c + di$.

**Goal:**
Return the product of x and y. That is $x * y = z = g + hi$.

Revise the following Q# code to return the product of x and y.

<details>
  <summary><b>Need a hint?</b></summary>
  
* Use the Complex data type defined in the Q# math library. For example, $x = a + bi$:

   ```qsharp

      let x = complex(real_value, imaginary_coefficient);
      let a = x::Real;
      let b = x::Imag;
   ```

* Remember, multiplying complex numbers is just like multiplying polynomials. Distribute one of the complex numbers: $$(a + bi)(c + di) = a(c + di) + bi(c + di)$$ Then multiply through, and group the real and imaginary terms together.

A video explanation of multiplying complex numbers can be found [here](https://www.youtube.com/watch?v=cWn6g8Qqvs4).

</details>
