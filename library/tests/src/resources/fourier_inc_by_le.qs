namespace Test {
    open Microsoft.Quantum.Arithmetic;
    open Microsoft.Quantum.Convert;
    open Microsoft.Quantum.Diagnostics;

    internal operation TestFourierIncByLE(n : Int) : Unit {
        use xs = Qubit[n];
        use ys = Qubit[n];

        for xsValue in 0..(1 <<< n) - 1 {
            for ysValue in 0..(1 <<< n) - 1 {
                ApplyXorInPlace(xsValue, xs);
                ApplyXorInPlace(ysValue, ys);
                FourierIncByLE(xs, ys);
                Fact(MeasureInteger(ys) == (xsValue + ysValue) % (1 <<< n),
                    $"TestFourierIncByLE (|xs|=|ys|): Incorrect ys for xs={xsValue}, ys={ysValue}");
                Fact(MeasureInteger(xs) == xsValue,
                    $"TestFourierIncByLE (|xs|=|ys|): Incorrect xs for xs={xsValue}, ys={ysValue}");
                ResetAll(xs);
                ResetAll(ys);
            }
        }

        let ysL = n+2;
        use xs = Qubit[n];
        use ys = Qubit[ysL];

        for xsValue in 0..(1 <<< n) - 1 {
            for ysValue in 0..(1 <<< ysL) - 1 {
                ApplyXorInPlace(xsValue, xs);
                ApplyXorInPlace(ysValue, ys);
                FourierIncByLE(xs, ys);
                Fact(MeasureInteger(ys) == (xsValue + ysValue) % (1 <<< ysL),
                    $"TestFourierIncByLE (|xs|<|ys|): Incorrect ys for xs={xsValue}, ys={ysValue}");
                Fact(MeasureInteger(xs) == xsValue,
                    $"TestFourierIncByLE (|xs|<|ys|): Incorrect xs for xs={xsValue}, ys={ysValue}");
                ResetAll(xs);
                ResetAll(ys);
            }
        }
    }
    
}
