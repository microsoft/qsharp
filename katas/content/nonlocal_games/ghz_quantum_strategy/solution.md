In Q#, you can perform measurements in a specific basis using either the
[Measure operation](https://learn.microsoft.com/qsharp/api/qsharp-lang/microsoft.quantum.intrinsic/measure)
or convenient shorthands for measure-and-reset-to-$\ket{0}$ sequence of operations
[MResetZ](https://learn.microsoft.com/qsharp/api/qsharp-lang/microsoft.quantum.measurement/mresetz) and
[MResetX](https://learn.microsoft.com/qsharp/api/qsharp-lang/microsoft.quantum.measurement/mresetx).

Alternatively, you can recall that measuring the qubit in the X basis is equivalent to applying an H gate to it and measuring it in the Z basis.

@[solution]({
    "id": "nonlocal_games__ghz_quantum_strategy_solution",
    "codePath": "Solution.qs"
})
