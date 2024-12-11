namespace Kata {
    import Std.Convert.*;

    operation MeasureQubits(qs : Qubit[], bases : Bool[]) : Bool[] {
        for i in 0..Length(qs) - 1 {
            if bases[i] {
                H(qs[i]);
            }
        }
        return ResultArrayAsBoolArray(MeasureEachZ(qs));
    }
}
