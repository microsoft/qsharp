namespace Kata.Verification {
    import KatasUtils.*;
    import Std.Random.*;

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

    @EntryPoint()
    operation CheckSolution() : Bool {
        for nVar in 2..6 {
            for _ in 1..3 {
                let formula = GenerateSATInstance(nVar, nVar - 1, -1);

                if not CheckOracleImplementsFunction(nVar, Kata.Oracle_SATFormula(_, _, formula), F_SATFormula(_, formula)) {
                    Message($"Test failed for SAT formula {SATFormulaAsString(formula)}");
                    return false;
                }
            }
        }

        Message("Correct!");
        true
    }
}
