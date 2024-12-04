// # Sample
// Functions
//
// # Description
// A function's execution always produces the same output, given the same input.
// Q# allows you to explicitly split out such purely deterministic computations into functions.
// Functions in Q#, contrasted with _operations_, are pure, i.e. lacking side-effects.
// Thus, functions can only call other functions, while operations can call both functions and operations.
function Main() : Unit {
    return ();
}

function MyFunction(qubits : Qubit[]) : Int {
    // Functions can call other functions.
    let length = Length(qubits);

    // Functions cannot call operations.
    // Uncommenting the following line would cause a compilation error.
    // CNOT(qubits[0], qubits[1]);

    return length;
}
