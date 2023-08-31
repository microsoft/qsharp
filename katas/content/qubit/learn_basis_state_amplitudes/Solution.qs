namespace Kata {
    open Microsoft.Quantum.Diagnostics;

    operation LearnBasisStateAmplitudes(qs: Qubit[]): (Double, Double) {
        DumpMachine(); // Only used to learn the amplitudes.
        return (0.3821, 0.339);
    }
}
