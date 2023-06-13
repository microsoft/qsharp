## <span style="color:blue">Demo: Implementing measurement in Q# using the M operation</span>

In this demo, we prepare a qubit in the state we've seen in Exercise 1, and then measure it in the computational basis. In Q#, single-qubit measurements in the computational basis can be implemented using the [M operation](https://docs.microsoft.com/qsharp/api/qsharp/microsoft.quantum.intrinsic.m). It will return the constant `Zero` if measurement result was $0$ or the constant `One` if the measurement result was $1$. `Zero` and `One` are constants of type `Result`.

> If you run this code multiple times, you will notice that whenever the measurement outcome is $1$, the post-measurement state of the qubit is $\ket 1$, and similarly for $0$. This is in line with our expectation that after the measurement the wave function 'collapses' to the corresponding state.
