The binary representation of $x$ is $x = (x_{0}, x_{1}, \dots, x_{N-1})$, with the most significant bit encoded in the first bit (stored in the first qubit of the input array). Then, you can rewrite the function as

$$f(x) = x_0$$

and its effect on the quantum state as 

$$U_f \ket{x} = (-1)^{f(x)} \ket{x} = (-1)^{x_0} \ket{x} = (-1)^{x_0} \ket{x_{0} } \otimes \ket{x_1} \otimes \cdots \otimes \ket{x_{N-1}}$$

As you've seen in the previous exercise, this can be achieved by applying a $Z$ gate to the first qubit.

@[solution]({
    "id": "deutsch_jozsa__msb_oracle_solution",
    "codePath": "./Solution.qs"
})
