In each of the above exercises, all generated numbers were equally likely. Now let's create an operation that will return a random bit with different probabilities of outcomes. 

> Remember that by setting the amplitudes of basis states $\alpha$ and $\beta$, we can control the probability of getting measurement outcomes $0$ and $1$ when the qubit is measured.

**Input:** 
A floating-point number $x$, $0 \le x \le 1$. 

**Goal:** Generate $0$ or $1$ with probability of $0$ equal to $x$ and probability of $1$ equal to $1 - x$.

> Useful Q# documentation: 
> * [`Math` namespace](https://docs.microsoft.com/qsharp/api/qsharp/microsoft.quantum.math)
> * [`ArcCos` function](https://docs.microsoft.com/qsharp/api/qsharp/microsoft.quantum.math.arccos)
> * [`Sqrt` function](https://docs.microsoft.com/qsharp/api/qsharp/microsoft.quantum.math.sqrt)
