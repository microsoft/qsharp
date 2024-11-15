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

    @EntryPoint()
    operation CheckSolution() : Bool {
        for i in 1..6 {
            let nVar = DrawRandomInt(3, 7);
            let clause = Generate_SAT_Clause(nVar, i);

            if not CheckOracleImplementsFunction(nVar, Kata.Oracle_SATClause(_, _, clause), F_SATClause(_, clause)) {
                Message($"Test failed for SAT clause {SATClauseAsString(clause)}");
                return false;
            }
        }

        Message("Correct!");
        true
    }
}
