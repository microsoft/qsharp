The binary representation of $x$ is $x = (x_{0}, x_{1}, \dots, x_{N-1})$, with the most significant bit encoded in the first bit (stored in the first qubit of the input array). Then we can rewrite the function as

$$f(x) = x_0$$

and its effect on the quantum state as 

$$U_f |x\rangle = (-1)^{f(x)} |x\rangle = (-1)^{x_0} |x\rangle = (-1)^{x_0} |x_{0} \rangle \otimes |x_1\rangle \otimes \cdots \otimes |x_{N-1}\rangle$$

As we've seen in the previous oracle, this can be achieved by applying a $Z$ gate to the first qubit.

@[solution]({
    "id": "deutsch_jozsa__msb_oracle_solution",
    "codePath": "./Solution.qs"
})
