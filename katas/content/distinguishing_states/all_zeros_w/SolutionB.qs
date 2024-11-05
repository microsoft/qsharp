namespace Kata {
    operation AllZerosOrWState(qs : Qubit[]) : Int {
        return MeasureInteger(qs) == 0 ? 0 | 1;
    }
}
