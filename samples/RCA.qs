namespace QuantumHelloWorld {

    @EntryPoint()
    operation RandomBit() : Result {
        Message("Hello world!");
        use qubit = Qubit();
        H(qubit);
        let result = M(qubit);
        Reset(qubit);
        return result;
    }
}