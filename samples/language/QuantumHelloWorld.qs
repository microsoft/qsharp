/// # Quantum Hello World!
///
/// This Q# code implements a "Hello world!" program for a quantum computer.
/// It generates a random bit by setting a qubit in a 50/50 superposition of
/// states |0〉 and |1〉, and the returning the result of measuring the qubit.
namespace QuantumHelloWorld {
    // 
    open Microsoft.Quantum.Intrinsic;

    // 
    @EntryPoint()
    operation RandomBit() : Result {
        // 
        Message("Hello world!");

        //
        use qubit = Qubit();

        //
        H(qubit);

        //
        let result = M(qubit);

        //
        Reset(qubit);
        return result;
    }
}
