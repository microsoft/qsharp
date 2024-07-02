The exercise can be done easily using solutions to earlier exercises.
- First, reconstruct the state of qubit `qBob` using the `ReconstructMessage` operation from an earlier exercise. This would bring the `qBob` state into one of the basis states of the given basis.
- Now, we know from the "Measurements in Single-Qubit Systems" kata that the default measurement operation `M` performs measurement in the Pauli $Z$ basis. To get the correct result for the measurement of `qBob` qubit, you need to perform the measurement in the given basis using the operation `Measure`.

@[solution]({
    "id": "teleportation__reconstruct_and_measure_solution",
    "codePath": "./Solution.qs"
})