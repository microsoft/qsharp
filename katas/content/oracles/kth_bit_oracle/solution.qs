namespace Kata.Verification {

    // Task 3.2.
    operation KthBit_Oracle_Reference (x : Qubit[], k : Int) : Unit is Adj + Ctl {
        Z(x[k]);
    }

}
