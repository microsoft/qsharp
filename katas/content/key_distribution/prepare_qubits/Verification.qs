namespace Kata.Verification {
    import Std.Diagnostics.*;

    @EntryPoint()
    operation CheckSolution() : Bool {
        for N in 2..10 {
            let (bases, bits) = (RandomArray(N), RandomArray(N));
            use qs = Qubit[N];
            Kata.PrepareQubits(qs, bases, bits);
            Adjoint PrepareQubits_Reference(qs, bases, bits);
            for i in 0..N - 1 {
                if not CheckZero(qs[i]) {
                    Message($"Qubit qs[{i}] prepared in incorrect state.");
                    Message($"Expected state {StateToString(bases[i], bits[i])}; actual state");
                    PrepareQubits_Reference([qs[i]], [bases[i]], [bits[i]]);
                    DumpRegister([qs[i]]);
                    ResetAll(qs);
                    return false;
                }
            }
        }

        Message("Correct!");
        true
    }
}
