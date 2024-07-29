In Q#, you can perform measurements in a specific basis using either the
[Measure operation](https://docs.microsoft.com/qsharp/api/qsharp/microsoft.quantum.intrinsic.measure)
or convenient shorthands for measure-and-reset-to-$\ket{0}$ sequence of operations
[MResetZ](https://docs.microsoft.com/qsharp/api/qsharp/microsoft.quantum.measurement.mresetz) and
[MResetX](https://docs.microsoft.com/qsharp/api/qsharp/microsoft.quantum.measurement.mresetx).

(See the the lesson below for details on why Alice should follow this strategy.)

@[solution]({
    "id": "nonlocal_games__chsh_quantum_alice_strategy_solution",
    "codePath": "Solution.qs"
})
