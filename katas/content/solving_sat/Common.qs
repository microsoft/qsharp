namespace Kata.Verification {
    import Std.Random.*;

    // Helper functions to pretty-print SAT formulas
    function SATVariableAsString(var : (Int, Bool)) : String {
        let (index, positive) = var;
        return (positive ? "" | "¬") + $"x{index}";
    }

    function SATClauseAsString(clause : (Int, Bool)[]) : String {
        mutable ret = SATVariableAsString(clause[0]);
        for ind in 1..Length(clause) - 1 {
            set ret = ret + " ∨ " + SATVariableAsString(clause[ind]);
        }
        return ret;
    }

    function SATFormulaAsString(formula : (Int, Bool)[][]) : String {
        mutable ret = "(" + SATClauseAsString(formula[0]) + ")";
        for ind in 1..Length(formula) - 1 {
            set ret = ret + " ∧ (" + SATClauseAsString(formula[ind]) + ")";
        }
        return ret;
    }

    // Helper operations to generate random SAT formulas
    operation Generate_SAT_Clause(nVar : Int, nTerms : Int) : (Int, Bool)[] {
        // number of terms in clause is either given or (if nTerms <= 0) chosen randomly
        mutable nVarInClause = (nTerms > 0) ? nTerms | DrawRandomInt(1, 4);
        if nVarInClause > nVar {
            set nVarInClause = nVar;
        }

        mutable clause = [(0, false), size = nVarInClause];
        mutable usedVariables = [false, size = nVar];
        // Make sure variables in the clause are distinct
        for k in 0..nVarInClause - 1 {
            mutable nextInd = -1;
            repeat {
                set nextInd = DrawRandomInt(0, nVar - 1);
            } until (not usedVariables[nextInd])
            fixup {}
            set clause w/= k <- (nextInd, DrawRandomBool(0.5));
            set usedVariables w/= nextInd <- true;
        }
        return clause;
    }

    operation GenerateSATInstance(nVar : Int, nClause : Int, nTerms : Int) : (Int, Bool)[][] {
        mutable problem = [[(0, false), size = 0], size = nClause];

        for j in 0..nClause - 1 {
            set problem w/= j <- Generate_SAT_Clause(nVar, nTerms);
        }
        return problem;
    }
}
