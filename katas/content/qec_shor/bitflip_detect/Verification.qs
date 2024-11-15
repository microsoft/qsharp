namespace Kata.Verification {
    import KatasUtils.*;

    operation BitflipEncode(qs : Qubit[]) : Unit is Adj + Ctl {
        CNOT(qs[0], qs[1]);
        CNOT(qs[0], qs[2]);
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        CheckErrorDetection(3, BitflipEncode, X, Kata.BitflipDetectError)
    }
}
