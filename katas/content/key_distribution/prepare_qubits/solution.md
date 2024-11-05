There are a total of 4 types of states that Alice can prepare before sending to Bob, each corresponds to the unique combination of bits and bases bool array. 

1. State $\ket{0}$ corresponds to `bases[i]` being in the computational basis (that is, `false`) and `bits[i]` being equal to 0 (that is, `false`).
2. State $\ket{1}$ corresponds to `bases[i]` being in the computational basis (that is, `false`) and `bits[i]` being equal to 1 (that is, `true`).
3. State $\ket{+}$ corresponds to `bases[i]` being in the Hadamard basis (that is, `true`) and `bits[i]` being equal to 0 (that is, `false`).
4. State $\ket{-}$ corresponds to `bases[i]` being in the Hadamard basis (that is, `true`) and `bits[i]` being equal to 1 (that is, `true`).

So, in case `bits[i]` is set to `true`, you need to apply the $X$ gate to the i-th qubit, and then if `bases[i]` is set to `true`, the $H$ gate needs to be applied to the i-th qubit. 

@[solution]({
    "id": "key_distribution__prepare_qubits_solution",
    "codePath": "./Solution.qs"
})
