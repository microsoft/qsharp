namespace Kata {

    // Task 1.3.
    operation IsSeven_MarkingOracle(x: Qubit[], y: Qubit): Unit is Adj + Ctl {
        Controlled X(x, y);
    }

}
