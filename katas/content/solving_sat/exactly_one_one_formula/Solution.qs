namespace Kata {
    import Std.Arrays.*;

    operation Oracle_Exactly13SATFormula(x : Qubit[], y : Qubit, formula : (Int, Bool)[][]) : Unit is Adj + Ctl {
        use aux = Qubit[Length(formula)];
        within {
            for i in 0..Length(formula) - 1 {
                Oracle_Exactly13SATClause(x, aux[i], formula[i]);
            }
        } apply {
            Oracle_And(aux, y);
        }
    }

    // You might want to implement this helper operation that evaluates a single clause and use it in your solution.
    operation Oracle_Exactly13SATClause(x : Qubit[], y : Qubit, clause : (Int, Bool)[]) : Unit is Adj + Ctl {
        let clauseQubits = Mapped((ind, _) -> x[ind], clause);
        within {
            for (ind, positive) in clause {
                if not positive {
                    X(x[ind]);
                }
            }
        } apply {
            Oracle_Exactly1One(clauseQubits, y);
        }
    }

    // You might find these helper operations from earlier tasks useful.
    operation Oracle_Exactly1One(x : Qubit[], y : Qubit) : Unit is Adj + Ctl {
        for i in 0..Length(x) - 1 {
            ApplyControlledOnInt(2^i, X, x, y);
        }
    }

    operation Oracle_And(x : Qubit[], y : Qubit) : Unit is Adj + Ctl {
        Controlled X(x, y);
    }
}
