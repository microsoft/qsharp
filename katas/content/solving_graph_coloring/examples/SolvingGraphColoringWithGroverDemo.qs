namespace Kata {
    import Std.Arrays.*;
    import Std.Convert.*;
    import Std.Diagnostics.*;
    import Std.Math.*;

    @EntryPoint()
    operation SolvingGraphColoringWithGroverDemo() : Unit {
        // Experiment with the parameters to explore algorithm behavior in different conditions!
        let V = 3;
        // The 0 -- 1 -- 2 graph from the examples
        let edges = [(0, 1), (1, 2)];
        let markingOracle = Oracle_VertexColoring(V, edges, _, _);
        for iterations in 0..9 {
            mutable success = 0;
            for _ in 1..100 {
                let res = GroversSearch(2 * V, markingOracle, iterations);
                // Convert measurement results to integers
                let colorPartitions = Chunks(2, res);
                let colors = Mapped(bits -> BoolArrayAsInt(Reversed(bits)), colorPartitions);
                if IsVertexColoringValid(V, edges, colors) {
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

    operation Oracle_VertexColoring(V : Int, edges : (Int, Int)[], x : Qubit[], y : Qubit) : Unit is Adj + Ctl {
        let edgesNumber = Length(edges);
        use conflicts = Qubit[edgesNumber];
        within {
            for i in 0..edgesNumber - 1 {
                let (v0, v1) = edges[i];
                Oracle_ColorEquality(
                    x[2 * v0 .. 2 * v0 + 1],
                    x[2 * v1 .. 2 * v1 + 1],
                    conflicts[i]
                );
            }
        } apply {
            ApplyControlledOnInt(0, X, conflicts, y);
        }
    }

    operation Oracle_ColorEquality(x0 : Qubit[], x1 : Qubit[], y : Qubit) : Unit is Adj + Ctl {
        within {
            for i in 0..Length(x0) - 1 {
                CNOT(x0[i], x1[i]);
            }
        } apply {
            ApplyControlledOnInt(0, X, x1, y);
        }
    }

    function IsVertexColoringValid(V : Int, edges : (Int, Int)[], colors : Int[]) : Bool {
        for (v0, v1) in edges {
            if colors[v0] == colors[v1] {
                return false;
            }
        }
        return true;
    }
}
