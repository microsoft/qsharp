namespace Kata.Verification {
    open Microsoft.Quantum.Arrays;
    open Microsoft.Quantum.Convert;
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Random;

    operation RandomArray(N : Int) : Bool[] {
        ForEach(x => DrawRandomInt(0, 1) == 0, [0, size = N])
    }

    operation PrepareAlicesQubits_Reference(qs : Qubit[], bases : Bool[], bits : Bool[]) : Unit is Adj {
        for i in 0 .. Length(qs) - 1 {
            if bits[i] {
                X(qs[i]);
            }
            if bases[i] {
                H(qs[i]);
            }
        }
    }

    operation StateToString(base : Bool, bit : Bool) : String {
        if base {  // ∣+⟩ / ∣-⟩
            return bit ? "|-⟩" | "|+⟩";
        } else {  // ∣0⟩ / ∣1⟩
            return bit ? "|1⟩" | "|-⟩";
        }
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        for N in 2 .. 10 {
            let (bases, bits) = (RandomArray(N), RandomArray(N));
            use qs = Qubit[N];
            Kata.PrepareAlicesQubits(qs, bases, bits);
            Adjoint PrepareAlicesQubits_Reference(qs, bases, bits);
            for i in 0 .. N - 1 {
                if not CheckZero(qs[i]) {
                    Message($"Qubit qs[{i}] prepared in incorrect state.");
                    Message($"Expected state {StateToString(bases[i], bits[i])}; actual state");
                    PrepareAlicesQubits_Reference([qs[i]], [bases[i]], [bits[i]]);
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
