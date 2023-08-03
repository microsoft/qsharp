/// # Quantum Hello World
///
/// This Q# program...
namespace QuantumHelloWorld {
    // 
    open Microsoft.Quantum.Intrinsic;

    // 
    @EntryPoint()
    operation RandomBit() : Result {
        // 
        Message("Generating a random bit");

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
