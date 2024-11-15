namespace Kata {
    import Std.Arrays.*;
    import Std.Convert.*;
    import Std.Diagnostics.*;
    import Std.Math.*;

    @EntryPoint()
    operation SolvingSATWithGroverDemo() : Unit {
        // Experiment with the parameters to explore algorithm behavior in different conditions!
        let n = 3;
        // (x₀ ∨ x₁) ∧ (¬x₀ ∨ ¬x₁) ∧ ¬x₂
        let formula = [[(0, true), (1, true)], [(0, false), (1, false)], [(2, false)]];
        let markingOracle = Oracle_SATFormula(_, _, formula);
        for iterations in 0..9 {
            mutable success = 0;
            for _ in 1..100 {
                let res = GroversSearch(n, markingOracle, iterations);
                if F_SATFormula(res, formula) {
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

    operation Oracle_SATFormula(x : Qubit[], y : Qubit, formula : (Int, Bool)[][]) : Unit is Adj + Ctl {
        use aux = Qubit[Length(formula)];
        within {
            for i in 0..Length(formula) - 1 {
                Oracle_SATClause(x, aux[i], formula[i]);
            }
        } apply {
            Oracle_And(aux, y);
        }
    }

    operation Oracle_SATClause(x : Qubit[], y : Qubit, clause : (Int, Bool)[]) : Unit is Adj + Ctl {
        let clauseQubits = Mapped((ind, _) -> x[ind], clause);
        within {
            for (ind, positive) in clause {
                if not positive {
                    X(x[ind]);
                }
            }
        } apply {
            Oracle_Or(clauseQubits, y);
        }
    }

    operation Oracle_Or(x : Qubit[], y : Qubit) : Unit is Adj + Ctl {
        ApplyControlledOnInt(0, X, x, y);
        X(y);
    }

    operation Oracle_And(x : Qubit[], y : Qubit) : Unit is Adj + Ctl {
        Controlled X(x, y);
    }

    function F_SATClause(args : Bool[], clause : (Int, Bool)[]) : Bool {
        for (index, positive) in clause {
            if positive == args[index] {
                // one true literal is sufficient for the clause to be true
                return true;
            }
        }
        // none of the literals is true - the whole clause is false
        return false;
    }

    function F_SATFormula(args : Bool[], formula : (Int, Bool)[][]) : Bool {
        for clause in formula {
            // one false clause invalidates the whole formula
            if not F_SATClause(args, clause) {
                return false
            }
        }
        return true;
    }
}
