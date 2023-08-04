/// # Quantum Hello World!
///
/// This Q# code implements the equivalent of a "Hello world!" program in
/// quantum computing.
///
/// It generates a random bit by setting a qubit in a superposition of the
/// computational basis states |0〉 and |1〉, and returning the result of measuring
/// the qubit.
namespace QuantumHelloWorld {
    // The following `open` directive is used to import all types and callables
    // declared in the Microsoft.Quantum.Intrinsic namespace, which includes
    // the `Message` function and the `H` operation used in the program.
    open Microsoft.Quantum.Intrinsic;

    // The `EntryPoint` attribute is used to mark an operation as the entry
    // point of the program.
    @EntryPoint()
    operation RandomBit() : Result {
        Message("Hello world!");

        // Allocate a qubit.
        use qubit = Qubit();

        // Set the qubit in superposition by applying a Hadamard transformation
        // using the `H` operation.
        H(qubit);

        // Measure the qubit using the `M` operation.
        // There is a 50% probability of measuring either `Zero` or `One`. 
        let result = M(qubit);

        // Reset the qubit so it can be safely released, and return the
        // measurement result as the output of the program.
        Reset(qubit);
        return result;
    }
}
