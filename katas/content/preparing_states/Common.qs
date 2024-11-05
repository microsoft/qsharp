namespace Kata.Verification {

    operation ApplyToFirstCA(op : Qubit => Unit is Adj + Ctl, qs : Qubit[]) : Unit is Adj + Ctl {
        op(qs[0]);
    }
}