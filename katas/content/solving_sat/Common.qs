namespace Kata.Verification {
    open Microsoft.Quantum.Random;

    // A set of helper functions to pretty-print SAT formulas
    function SATVariableAsString (var : (Int, Bool)) : String {
        let (index, positive) = var;
        return (positive ? "" | "¬") + $"x{index}";
    }

    function SATClauseAsString (clause : (Int, Bool)[]) : String {
        mutable ret = SATVariableAsString(clause[0]);
        for ind in 1 .. Length(clause) - 1 {
            set ret = ret + " ∨ " + SATVariableAsString(clause[ind]);
        }
        return ret;
    }

    function SATInstanceAsString (instance : (Int, Bool)[][]) : String {
        mutable ret = "(" + SATClauseAsString(instance[0]) + ")";
        for ind in 1 .. Length(instance) - 1 {
            set ret = ret + " ∧ (" + SATClauseAsString(instance[ind]) + ")";
        }
        return ret;
    }

    operation Generate_SAT_Clause (nVar : Int, nTerms : Int) : (Int, Bool)[] {
        mutable nVarInClause = (nTerms > 0) ? nTerms | DrawRandomInt(1, 4);
        if nVarInClause > nVar {
            set nVarInClause = nVar;
        }
    
        mutable clause = [(0, false), size = nVarInClause];
        mutable usedVariables = [false, size = nVar];
        // Make sure variables in the clause are distinct
        for k in 0 .. nVarInClause - 1 {
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

}