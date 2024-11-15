namespace Kata.Verification {
    import KatasUtils.*;
    import Std.Random.*;

    function F_Exactly1SATClause(args : Bool[], clause : (Int, Bool)[]) : Bool {
        mutable nOnes = 0;
        for (index, isTrue) in clause {
            if isTrue == args[index] {
                set nOnes += 1;
            }
        }
        return nOnes == 1;
    }

    function F_Exactly1SATFormula(args : Bool[], formula : (Int, Bool)[][]) : Bool {
        for clause in formula {
            if not F_Exactly1SATClause(args, clause) {
                return false;
            }
        }
        return true;
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        for nVar in 3..6 {
            for _ in 1..3 {
                let formula = GenerateSATInstance(nVar, nVar - 1, 3);

                if not CheckOracleImplementsFunction(nVar, Kata.Oracle_Exactly13SATFormula(_, _, formula), F_Exactly1SATFormula(_, formula)) {
                    Message($"Test failed for SAT formula {SATFormulaAsString(formula)}");
                    return false;
                }
            }
        }

        Message("Correct!");
        true
    }
}
