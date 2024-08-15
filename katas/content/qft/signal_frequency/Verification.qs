namespace Kata.Verification {
    open Microsoft.Quantum.Arrays;
    open Microsoft.Quantum.Convert;
    open Microsoft.Quantum.Katas;

    operation PreparePeriodicState(qs : Qubit[], F : Int) : Unit is Adj + Ctl {
        let bitsBE = Reversed(IntAsBoolArray(F, Length(qs)));
        ApplyPauliFromBitString(PauliX, true, bitsBE, qs);
        ApplyQFT(Reversed(qs));
        SwapReverseRegister(qs);
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        for n in 2 .. 4 {
            use qs = Qubit[n];
            for F in 0 .. 2 ^ n - 1 {
                PreparePeriodicState(qs, F);
                let fRes = Kata.SignalFrequency(qs);
                ResetAll(qs);
                if fRes != F {
                    Message($"Incorrect frequency for n = {n}, F = {F}: got {fRes}");
                    return false;
                }
            }
        }

        Message("Correct!");
        return true;
    }
}
