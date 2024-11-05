namespace Kata {
    function FindFirstDiff(bits1 : Bool[], bits2 : Bool[]) : Int {
        for i in 0 .. Length(bits1) - 1 {
            if bits1[i] != bits2[i] {
                return i;
            }
        }
        return -1;
    }

    operation TwoBitstringsMeasurement(qs : Qubit[], bits1 : Bool[], bits2 : Bool[]) : Int {
        let firstDiff = FindFirstDiff(bits1, bits2);
        let res = M(qs[firstDiff]) == One;

        return res == bits1[firstDiff] ? 0 | 1;
    }
}
