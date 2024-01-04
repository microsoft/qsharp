namespace Test {
    open Microsoft.Quantum.Measurement;
    open Microsoft.Quantum.Unstable.Arithmetic;
    open Microsoft.Quantum.Convert;
    open Microsoft.Quantum.Diagnostics;

    internal operation CompareWithBigInt(
        name: String,
        bitwidth : Int,
        quantumComparator : (BigInt, Qubit[], Qubit) => Unit,
        classicalComparator : (Int, Int) -> Bool) : Unit {
        
        for n in 1..bitwidth {
            use qs = Qubit[n];
            use t = Qubit();

            for a in 0 .. 2^n+1 { // We want b to have more bits sometimes...
                for b in 0 .. 2^n-1 {
                    ApplyXorInPlace(b, qs);
                    quantumComparator(IntAsBigInt(a), qs, t);
                    let actual = MResetZ(t) == One;
                    let expected = classicalComparator(a, b);
                    Fact(actual == expected,
                        $"{name}: Wrong result {actual}, expected {expected}. bitwidth={n}, a={a}, b={b}.");
                    ResetAll(qs);
                }
            }
        }
    }

    internal operation CompareWithLE(
        name: String,
        bitwidth : Int,
        quantumComparator : (Qubit[], Qubit[], Qubit) => Unit,
        classicalComparator : (Int, Int) -> Bool) : Unit {
        
        for n in 1..bitwidth {
            use x = Qubit[n];
            use y = Qubit[n];
            use t = Qubit();

            for a in 0 .. 2^n-1 {
                for b in 0 .. 2^n-1 {
                    ApplyXorInPlace(a, x);
                    ApplyXorInPlace(b, y);
                    quantumComparator(x, y, t);
                    let actual = MResetZ(t) == One;
                    let expected = classicalComparator(a, b);
                    Fact(actual == expected,
                        $"{name}: Wrong result {actual}, expected {expected}. bitwidth={n}, a={a}, b={b}.");
                    ResetAll(x);
                    ResetAll(y);
                }
            }
        }
    }

}
