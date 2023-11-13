namespace Test {
    open Microsoft.Quantum.Arithmetic;
    open Microsoft.Quantum.Convert;
    open Microsoft.Quantum.Diagnostics;

    operation TestLookAheadAddLE(n : Int) : Unit {
        use xs = Qubit[n];
        use ys = Qubit[n];
        use zs = Qubit[n];

        for xsValue in 0..(1 <<< n) - 1 {
            for ysValue in 0..(1 <<< n) - 1 {
                ApplyXorInPlace(xsValue, xs);
                ApplyXorInPlace(ysValue, ys);
                LookAheadAddLE(xs, ys, zs);
                Fact(MeasureInteger(xs) == xsValue,
                   $"TestLookAheadAddLE (|xs|=|zs|): Incorrect xs for xs={xsValue}, ys={ysValue}");
                Fact(MeasureInteger(ys) == ysValue,
                   $"TestLookAheadAddLE (|xs|=|zs|): Incorrect ys for xs={xsValue}, ys={ysValue}");
                Fact(MeasureInteger(zs) == (xsValue + ysValue) % (1 <<< n),
                   $"TestLookAheadAddLE (|xs|=|zs|): Incorrect zs for xs={xsValue}, ys={ysValue}");
                ResetAll(xs);
                ResetAll(ys);
                ResetAll(zs);
            }
        }

        let zsL = n+1;
        use xs = Qubit[n];
        use ys = Qubit[n];
        use zs = Qubit[zsL];

        for xsValue in 0..(1 <<< n) - 1 {
            for ysValue in 0..(1 <<< n) - 1 {
                ApplyXorInPlace(xsValue, xs);
                ApplyXorInPlace(ysValue, ys);
                LookAheadAddLE(xs, ys, zs);
                Fact(MeasureInteger(xs) == xsValue,
                    $"TestLookAheadAddLE (|xs|<|zs|): Incorrect xs for xs={xsValue}, ys={ysValue}");
                Fact(MeasureInteger(ys) == ysValue,
                    $"TestLookAheadAddLE (|xs|<|zs|): Incorrect ys for xs={xsValue}, ys={ysValue}");
                Fact(MeasureInteger(zs) == (xsValue + ysValue) % (1 <<< zsL),
                    $"TestLookAheadAddLE (|xs|<|zs|): Incorrect zs for xs={xsValue}, ys={ysValue}");
                ResetAll(xs);
                ResetAll(ys);
                ResetAll(zs);
            }
        }
    }

}
