namespace Kata {
    import Std.Convert.*;
    import Std.Diagnostics.*;
    import Std.Math.*;

    @EntryPoint()
    operation GroversSearchAlgorithmDemo() : Unit {
        // Experiment with the parameters to explore algorithm behavior in different conditions!
        let n = 3;
        let prefix = [false, true, false];
        let markingOracle = Oracle_StartsWith(_, _, prefix);
        for iterations in 0..9 {
            mutable success = 0;
            for _ in 1..100 {
                let res = GroversSearch(n, markingOracle, iterations);
                if BoolArrayAsInt(prefix) == BoolArrayAsInt(res) {
                    set success += 1;
                }
            }
            Message($"{iterations} iterations - {success}% success rate");
        }
    }

    operation GroversSearch(
        n : Int,
        markingOracle : (Qubit[], Qubit) => Unit is Adj + Ctl,
        iterations : Int
    ) : Bool[] {
        use qs = Qubit[n];

        // Operation that prepares the state |all⟩.
        let meanStatePrep = ApplyToEachCA(H, _);

        // The phase oracle.
        let phaseOracle = ApplyMarkingOracleAsPhaseOracle(markingOracle, _);

        // Prepare the system in the state |all⟩.
        meanStatePrep(qs);

        // Do Grover's iterations.
        for _ in 1..iterations {
            // Apply the phase oracle.
            phaseOracle(qs);

            // Apply "reflection about the mean".
            ReflectionAboutState(qs, meanStatePrep);
        }

        // Measure to get the result.
        return ResultArrayAsBoolArray(MResetEachZ(qs));
    }

    operation Oracle_StartsWith(x : Qubit[], y : Qubit, p : Bool[]) : Unit is Adj + Ctl {
        ApplyControlledOnBitString(p, X, x[...Length(p) - 1], y);
    }

    operation ApplyMarkingOracleAsPhaseOracle(
        markingOracle : (Qubit[], Qubit) => Unit is Adj + Ctl,
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

    operation ReflectionAboutState(
        qs : Qubit[],
        statePrep : Qubit[] => Unit is Adj + Ctl
    ) : Unit is Adj + Ctl {
        within {
            Adjoint statePrep(qs);
        } apply {
            ConditionalPhaseFlip(qs);
        }
    }

    operation ConditionalPhaseFlip(qs : Qubit[]) : Unit is Adj + Ctl {
        within {
            ApplyToEachA(X, qs);
        } apply {
            Controlled Z(qs[1...], qs[0]);
        }
        R(PauliI, 2.0 * PI(), qs[0]);
    }
}
