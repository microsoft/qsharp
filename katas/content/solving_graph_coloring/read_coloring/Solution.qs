namespace Kata {
    import Std.Arrays.*;
    import Std.Convert.*;

    operation ReadColoring(nBits : Int, qs : Qubit[]) : Int[] {
        let colorPartitions = Chunks(nBits, qs);
        let measureColor = qs => ResultArrayAsInt(Reversed(MeasureEachZ(qs)));
        return ForEach(measureColor, colorPartitions);
    }
}
