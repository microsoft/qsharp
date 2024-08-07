In this oracle the answer depends on all bits of the input. You can write $f(x)$ as follows (here $\bigoplus$ denotes sum modulo $2$):

$$f(x) = \bigoplus_{k=0}^{N-1} x_k$$ 

Let's substitute this expression in the expression for the oracle effect on the quantum state:

$$U_f \ket{x} = (-1)^{f(x)} \ket{x} = (-1)^{\bigoplus_{k=0}^{N-1} x_k} \ket{x}$$

Since $(-1)^2 = 1$, you can replace sum modulo $2$ with a regular sum in the exponent. Then you'll be able to rewrite it as a product of individual exponents for each bit:

$$U_f \ket{x} = (-1)^{\sum_{k=0}^{N-1} x_k} \ket{x} = \prod_{k=0}^{N-1} {(-1)^{x_k}} \ket{x}$$

Now let's spell out the system state as a tensor product of individual qubit states:

$$U_f \ket{x} = \prod_{k=0}^{N-1} {(-1)^{x_k}} \cdot \ket{x_{0} } \otimes \cdots \otimes \ket{x_{N-1}}$$

Tensor product is a linear operation, so you can bring each $(-1)^{x_k}$ scalar factor in next to the corresponding $\ket{x_k}$:

$$U_f \ket{x} = (-1)^{x_0} \ket{x_{k}} \otimes \dots \otimes (-1)^{x_{N-1}} \ket{x_{N-1}} = \bigotimes_{k=0}^{N-1} (-1)^{x_k} \ket{x_{k}}$$

As you've seen in the previous oracle, this can be achieved by applying a $Z$ gate to each qubit.

@[solution]({
    "id": "deutsch_jozsa__parity_oracle_solution",
    "codePath": "./Solution.qs"
})
