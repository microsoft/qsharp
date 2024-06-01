The exercise can be done easily using previous solutions with slight modification.
- Reconstruct `qBob` qubit using the ReconstructMessage operation from exercise. This would bring the qBob state into one of the eigenstates depending to the basis mentioned.
- We know from Single Qubit Measurement Kata, by default all measurements are made in PauliZ basis unless specified. To get the correct result for the measurement of `qBob` qubit, perform measurement in respective basis.

@[solution]({
    "id": "teleportation__reconstruct_and_measure_solution",
    "codePath": "./Solution.qs"
})