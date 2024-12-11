namespace Kata {
    import Std.Arrays.*;
    import Std.Convert.*;

    operation BasisStateMeasurement(qs : Qubit[]) : Int {
        return ResultArrayAsInt(Reversed(MeasureEachZ(qs)));
    }
}
