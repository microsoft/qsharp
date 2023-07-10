namespace Kata.Verification {

    // Task 3.1.
    operation Or_Oracle_Reference (x : Qubit[], y : Qubit) : Unit is Adj + Ctl {
        X(y);
        (ControlledOnInt(0, X))(x, y);
    }

}
