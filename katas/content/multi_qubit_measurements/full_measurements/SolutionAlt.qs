namespace Kata {
    open Microsoft.Quantum.Arrays;
    open Microsoft.Quantum.Convert;

    operation BasisStateMeasurement(qs : Qubit[]) : Int {
        return ResultArrayAsInt(Reversed(MeasureEachZ(qs)));
    }
}
