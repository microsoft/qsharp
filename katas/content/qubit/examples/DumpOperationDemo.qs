namespace Kata {
    open Microsoft.Quantum.Diagnostics;

    operation DumpOperationDemo () : Unit {
        DumpOperation(2, ApplyToFirstTwoQubitsCA(CNOT, _));
    }
}
