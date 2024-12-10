namespace Kata.Verification {
    import Std.Arrays.*;
    import KatasUtils.*;

    operation LibraryQFT(qs : Qubit[]) : Unit is Adj + Ctl {
        ApplyQFT(Reversed(qs));
        SwapReverseRegister(qs);
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        for n in 1..5 {
            if not CheckOperationsAreEqualStrict(n, Kata.QuantumFourierTransform, LibraryQFT) {
                Message($"Incorrect for n = {n}.");
                return false;
            }
        }

        Message("Correct!");
        true
    }
}
