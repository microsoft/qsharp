namespace Kata {
    import Std.Convert.*;

    operation SuperpositionMeasurement(qs : Qubit[], bits1 : Bool[][], bits2 : Bool[][]) : Int {
        let measuredState = MeasureInteger(qs);
        for s in bits1 {
            if BoolArrayAsInt(s) == measuredState {
                return 0;
            }
        }

        return 1;
    }
}
