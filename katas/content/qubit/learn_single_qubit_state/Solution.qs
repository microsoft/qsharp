namespace Kata { 
    open Microsoft.Quantum.Diagnostics;

    operation LearnSingleQubitState (q : Qubit) : (Double, Double) {
        DumpMachine(); // Only used to learn the amplitudes.
        return (0.9689, 0.2474);
    }
}
