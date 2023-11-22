namespace QuantumHelloWorld {

    @EntryPoint()
    operation RCA() : Result {
        Message("Hello world!");
        use qubit = Qubit();
        H(qubit);
        let result = M(qubit);
        Reset(qubit);
        let a = FFoo(11111);
        let fiveTwos = 22222;
        let b = FFoo(fiveTwos);
        let c = FBar(33333, 44444);
        let fiveFives = 55555;
        let fiveSixes = 66666;
        let d = FBar(fiveFives, fiveSixes);
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

    function FFoo(a : Int) : Int {
        return a * 10;
    }

    function FBar(a : Int, b : Int) : Int {
        let c = a + b;
        c * 10
    }

    function FBas(a : Double[]) : Double {
        a[0]
    }

    function FBat(a : Double[], i : Int) : Double {
        a[i]
    }

    function FBau(a : (Int, Int)) : (Int, Int) {
        let (a0, a1) = a;
        return (a0 + 1, a1 + 1);
    }
}