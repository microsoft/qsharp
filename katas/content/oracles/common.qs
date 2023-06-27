    // ------------------------------------------------------
    // Helper functions
    operation ApplyOracle (qs : Qubit[], oracle : ((Qubit[], Qubit) => Unit is Adj + Ctl)) : Unit is Adj + Ctl {
        let N = Length(qs);
        oracle(qs[0 .. N - 2], qs[N - 1]);
    }

    operation AssertTwoOraclesAreEqual (nQubits : Range,
        oracle1 : ((Qubit[], Qubit) => Unit is Adj + Ctl),
        oracle2 : ((Qubit[], Qubit) => Unit is Adj + Ctl)) : Unit {
        let sol = ApplyOracle(_, oracle1);
        let refSol = ApplyOracle(_, oracle2);

        for i in nQubits {
            AssertOperationsEqualReferenced(i + 1, sol, refSol);
        }
    }
