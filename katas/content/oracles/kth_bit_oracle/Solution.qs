namespace Kata {
    operation KthBit_Oracle(x : Qubit[], k : Int) : Unit is Adj + Ctl {
        Z(x[k]);
    }
}
