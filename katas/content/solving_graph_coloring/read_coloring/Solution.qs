namespace Kata {
    open Microsoft.Quantum.Arrays;
    open Microsoft.Quantum.Convert;

    operation ReadColoring(nBits : Int, qs : Qubit[]) : Int[] {
        let colorPartitions = Chunks(nBits, qs);
        let measureColor = qs => ResultArrayAsInt(Reversed(MeasureEachZ(qs)));
        return ForEach(measureColor, colorPartitions);
    }
}
