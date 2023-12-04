namespace QuantumHelloWorld {

    @EntryPoint()
    operation RCA() : Result {
        Message("Hello world!");
        use myQubit = Qubit();
        H(myQubit);
        let result = M(myQubit);
        let dynIntA = result == One ? 0 | 1;
        let conditionB = result == Zero;
        mutable dynIntB = 0;
        if conditionB {
            set dynIntB = 1;
        }

        mutable dynDoubleA = -1.0;
        if dynIntB == 0 {
            set dynDoubleA = 0.0;
        } elif dynIntB == 1 {
            set dynDoubleA = 1.0;
        } else {
            set dynDoubleA = 2.0;
        }
        Reset(myQubit);
        let a = FFoo(11111);
        let fiveTwos = 22222;
        let b = FFoo(fiveTwos);
        let c = FBar(33333, 44444);
        let fiveFives = 55555;
        let fiveSixes = 66666;
        let d = FBar(fiveFives, fiveSixes);
        let eTuple = (77777, One, Zero);
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