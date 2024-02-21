namespace Kata {
    open Microsoft.Quantum.Arrays;
    open Microsoft.Quantum.Convert;
    open Microsoft.Quantum.Measurement;

    operation BasisStateMeasurement(qs : Qubit[]) : Int {
        return ResultArrayAsInt(Reversed(MeasureEachZ(qs)));
    }
    
}
