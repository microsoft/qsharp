namespace Kata.Verification {
    import Std.Diagnostics.*;

    @EntryPoint()
    operation CheckSolution() : Bool {
        for N in 2..10 {
            let (bases, bits) = (RandomArray(N), RandomArray(N));
            use qs = Qubit[N];
            PrepareQubits_Reference(qs, bases, bits);
            let res = Kata.MeasureQubits(qs, bases);
            ResetAll(qs);

            if Length(res) != N {
                Message($"The returned array should have length {N}, same as the inputs, and it had length {Length(res)}.");
                return false;
            }

            for i in 0..N - 1 {
                if res[i] != bits[i] {
                    Message($"Qubit qs[{i}] measured in incorrect basis.");
                    Message($"When measuring state {StateToString(bases[i], bits[i])} in the {BasisToString(bases[i])} basis, " +
                            $"expected result is {bits[i]}, got {res[i]}.");
                    return false;
                }
            }
        }

        Message("Correct!");
        true
    }
}
