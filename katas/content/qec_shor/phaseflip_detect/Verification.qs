namespace Kata.Verification {
    import KatasUtils.*;

    operation PhaseflipEncode(qs : Qubit[]) : Unit is Adj + Ctl {
        CNOT(qs[0], qs[1]);
        CNOT(qs[0], qs[2]);
        ApplyToEachCA(H, qs);
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        CheckErrorDetection(3, PhaseflipEncode, Z, Kata.PhaseflipDetectError)
    }
}
