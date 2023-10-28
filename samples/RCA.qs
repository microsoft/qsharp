namespace QuantumHelloWorld {

    @EntryPoint()
    operation RCA() : Result {
        Message("Hello world!");
        use qubit = Qubit();
        H(qubit);
        let result = M(qubit);
        Reset(qubit);
        return result;
    }

    operation InstrinsicallyAdaptive() : Bool {
        mutable b = false;
        use qubit = Qubit();
        H(qubit);
        let result = M(qubit);
        if (result == One) {
            set b = true;
        }

        b
    }

    function Foo(a : Int, b : Int) : Int {
        let c = a + b;
        return c * 10;
    }

    function Bar(a : Int, b : Int) : Int {
        let c = a + b;
        c * 10
    }
}