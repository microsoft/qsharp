namespace GenerateRandom {
    open Microsoft.Quantum.Arrays;
    open Microsoft.Quantum.Convert;
    open Microsoft.Quantum.Math;
    open Microsoft.Quantum.Random;

    operation GenerateRandomNumbers(min: Int, max: Int) : (Result[], Int) {

        let nQubits = BitSizeI(max);
        use qubits = Qubit[nQubits];

        ApplyToEach(H, qubits);

        let result = MeasureEachZ(qubits);

        MResetEachZ(qubits);

        let number = ResultArrayAsInt(Reversed(result));
        if(number > max){
            return GenerateRandomNumbers(0, max);
        }

        return (result, number);
    }
}
