In this oracle the answer depends on all bits of the input. We can write $f(x)$ as follows (here $\bigoplus$ denotes sum modulo $2$):

$$f(x) = \bigoplus_{k=0}^{N-1} x_k$$ 

Let's substitute this expression in the expression for the oracle effect on the quantum state:

$$U_f |x\rangle = (-1)^{f(x)} |x\rangle = (-1)^{\bigoplus_{k=0}^{N-1} x_k} |x\rangle$$

Since $(-1)^2 = 1$, we can replace sum modulo $2$ with a regular sum in the exponent. Then we'll be able to rewrite it as a product of individual exponents for each bit:

$$U_f |x\rangle = (-1)^{\sum_{k=0}^{N-1} x_k} |x\rangle = \prod_{k=0}^{N-1} {(-1)^{x_k}} |x\rangle$$

Now let's spell out the system state as a tensor product of individual qubit states:

$$U_f |x\rangle = \prod_{k=0}^{N-1} {(-1)^{x_k}} \cdot |x_{0} \rangle \otimes \cdots \otimes |x_{N-1}\rangle$$

Tensor product is a linear operation, so we can bring each $(-1)^{x_k}$ scalar factor in next to the corresponding $|x_k\rangle$:

$$U_f |x\rangle = (-1)^{x_0} |x_{k}\rangle \otimes \dots \otimes (-1)^{x_{N-1}} |x_{N-1}\rangle = \bigotimes_{k=0}^{N-1} (-1)^{x_k} |x_{k}\rangle$$

As we've seen in the previous oracle, this can be achieved by applying a $Z$ gate to each qubit.

@[solution]({
    "id": "deutsch_jozsa__parity_oracle_solution",
    "codePath": "./Solution.qs"
})
