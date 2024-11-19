namespace Kata.Verification {
    import Std.Convert.*;
    import KatasUtils.*;
    import Std.Random.*;

    operation ApplyMarkingOracleAsPhaseOracle_Reference(
        markingOracle : ((Qubit[], Qubit) => Unit is Adj + Ctl),
        qubits : Qubit[]
    ) : Unit is Adj + Ctl {

        use minus = Qubit();
        within {
            X(minus);
            H(minus);
        } apply {
            markingOracle(qubits, minus);
        }
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        for N in 1..3 {
            for k in 0..2^N - 1 {
                let pattern = IntAsBoolArray(k, N);
                let marking = ApplyControlledOnBitString(pattern, X, _, _);
                let sol = Kata.ApplyMarkingOracleAsPhaseOracle(marking, _);
                let ref = ApplyMarkingOracleAsPhaseOracle_Reference(marking, _);

                let isCorrect = CheckOperationsAreEqualStrict(N, sol, ref);

                if not isCorrect {
                    Message("Incorrect.");
                    Message("Hint: examine how your solution transforms the given state and compare it with the expected " +
                        $"transformation for the {N}-bit oracle that marks the bit string {pattern}");
                    ShowQuantumStateComparison(N, PrepDemoState, sol, ref);
                    return false;
                }
            }
        }
        Message("Correct!");
        true
    }

}
