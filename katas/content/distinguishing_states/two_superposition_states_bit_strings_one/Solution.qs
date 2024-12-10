namespace Kata {
    import Std.Convert.*;

    function FindFirstSuperpositionDiff(bits1 : Bool[][], bits2 : Bool[][], nQubits : Int) : Int {
        for i in 0..nQubits - 1 {
            // count the number of 1s in i-th position in bit strings of both arrays
            mutable val1 = 0;
            mutable val2 = 0;
            for j in 0..Length(bits1) - 1 {
                if bits1[j][i] {
                    set val1 += 1;
                }
            }
            for k in 0..Length(bits2) - 1 {
                if bits2[k][i] {
                    set val2 += 1;
                }
            }
            if (val1 == Length(bits1) and val2 == 0) or (val1 == 0 and val2 == Length(bits2)) {
                return i;
            }
        }

        return -1;
    }

    operation SuperpositionOneMeasurement(qs : Qubit[], bits1 : Bool[][], bits2 : Bool[][]) : Int {
        let diff = FindFirstSuperpositionDiff(bits1, bits2, Length(qs));

        let res = ResultAsBool(M(qs[diff]));

        if res == bits1[0][diff] {
            return 0;
        } else {
            return 1;
        }
    }
}
