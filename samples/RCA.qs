namespace QuantumHelloWorld {

    open Microsoft.Quantum.Convert;
    @EntryPoint()
    operation RCA() : Result {
        Message("Hello world!");

        let staticInt = 10;
        let staticDouble = IntAsDouble(staticInt);
        let staticBigInt = IntAsBigInt(staticInt);

        use myQubit = Qubit();
        H(myQubit);
        let result = M(myQubit);
        let dynInt = result == One ? 0 | 1;
        //let dynDouble = IntAsDouble(dynInt);
        //let dynBigInt = IntAsBigInt(dynInt);

        //let staticFoo = Foo(42);
        //let dynFoo = Foo(dynInt);

        //let staticBar0 = Bar(0, 1);
        //let dynBar1 = Bar(0, dynInt);
        //let dynBar2 = Bar(dynInt, 1);
        //let dynBar3 = Bar(dynInt, dynInt);

        //let staticBaz0 = Baz(0, 1);
        //let staticBaz1 = Baz(0, dynInt);
        //let dynBaz2 = Baz(dynInt, 1);
        //let dynBaz3 = Baz(dynInt, dynInt);

        Reset(myQubit);
        return result;
    }

    function Foo(a : Int) : Double {
        let i = a * 10;
        IntAsDouble(i)
    }

    function Bar(a : Int, b : Int) : Double {
        let i = a + b;
        IntAsDouble(i)
    }

    function Baz(a : Int, b : Int) : Double {
        IntAsDouble(a / 1)
    }

    operation InstrinsicallyAdaptiveInt() : Unit {
        mutable b = false;
        use qubit = Qubit();
        H(qubit);
        let dynInt = M(qubit) == Zero ? 41 | 42;
    }

    operation InstrinsicallyAdaptiveDouble() : Unit {
        use qubit = Qubit();
        H(qubit);
        let dynDouble = M(qubit) == Zero ? 41.0 | 42.0;
        Reset(qubit);
    }
}