Unlike in the previous task, this time measuring the first qubit won't give us any information on the second qubit, so we need to measure both qubits.

First, we measure both qubits in the input array and store the results in `m1` and `m2`. We can decode these results like this:

* m1 is $\ket{0}$ and m2 is $\ket{0}$: we return $0\cdot2+0=0$
* m1 is $\ket{0}$ and m2 is $\ket{1}$: we return $0\cdot2+1=1$
* m1 is $\ket{1}$ and m2 is $\ket{0}$: we return $1\cdot2+0=2$
* m1 is $\ket{1}$ and m2 is $\ket{1}$: we return $1\cdot2+1=3$

In other words, we treat the measurement results as the binary notation of the return value in big endian notation, with the most significant bit stored in `m1` and the least significant - in `m2`.

@[solution]({
    "id": "distinguishing_states__four_basis_states_solution",
    "codePath": "Solution.qs"
})
