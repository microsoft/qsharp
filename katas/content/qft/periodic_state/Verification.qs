namespace Kata.Verification {
    import Std.Arrays.*;
    import Std.Convert.*;
    import KatasUtils.*;
    import Std.Math.*;
    import Std.StatePreparation.*;

    operation PeriodicState_Reference(qs : Qubit[], F : Int) : Unit is Adj + Ctl {
        let n = Length(qs);
        let amps = MappedOverRange(
            k -> ComplexPolar(1.0, 2. * PI() * IntAsDouble(F * k) / IntAsDouble(2^n)),
            0..2^n - 1
        );
        ApproximatelyPreparePureStateCP(0.0, amps, qs);
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        for n in 1..3 {
            for F in 0..2^n - 1 {
                let solution = Kata.PeriodicState(_, F);
                let reference = PeriodicState_Reference(_, F);
                if not CheckOperationsEquivalenceOnZeroState(solution, reference, n) {
                    Message($"Incorrect for {n} qubit(s), F = {F}");
                    ShowQuantumStateComparison(n, qs => (), solution, reference);
                    return false;
                }
            }
        }

        Message("Correct!");
        return true;
    }
}
