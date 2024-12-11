namespace Kata {
    import Std.Arrays.*;

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
}
