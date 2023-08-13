namespace Kata {
    operation QubitSwap (qs : Qubit[], index1 : Int, index2 : Int) : Unit is Adj + Ctl {
       SWAP(qs[index1], qs[index2]);
    }
}