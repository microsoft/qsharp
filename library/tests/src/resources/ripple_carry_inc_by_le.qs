namespace Test {
    open Microsoft.Quantum.Arithmetic;
    open Microsoft.Quantum.Convert;
    open Microsoft.Quantum.Diagnostics;

    internal operation TestRippleCarryIncByLE(n : Int) : Unit {
        use xs = Qubit[n];
        use ys = Qubit[n];

        for xsValue in 0..(1 <<< n) - 1 {
            for ysValue in 0..(1 <<< n) - 1 {
                ApplyXorInPlace(xsValue, xs);
                ApplyXorInPlace(ysValue, ys);
                RippleCarryIncByLE(xs, ys);
                Fact(MeasureInteger(ys) == (xsValue + ysValue) % (1 <<< n), $"unexpected value for `ys` given xsValue = {xsValue} and ysValue = {ysValue}");
                Fact(MeasureInteger(xs) == xsValue, $"unexpected value for `xs` given xsValue = {xsValue} and ysValue = {ysValue}");
                ResetAll(xs);
                ResetAll(ys);
            }
        }

        use xs = Qubit[n];
        use ys = Qubit[n + 1];

        for xsValue in 0..(1 <<< n) - 1 {
            for ysValue in 0..(1 <<< (n + 1)) - 1 {
                ApplyXorInPlace(xsValue, xs);
                ApplyXorInPlace(ysValue, ys);
                RippleCarryIncByLE(xs, ys);
                Fact(MeasureInteger(ys) == (xsValue + ysValue) % (1 <<< (n + 1)), $"unexpected value for `ys` given xsValue = {xsValue} and ysValue = {ysValue}");
                Fact(MeasureInteger(xs) == xsValue, $"unexpected value for `xs` given xsValue = {xsValue} and ysValue = {ysValue}");
                ResetAll(xs);
                ResetAll(ys);
            }
        }
    }

    internal operation TestRippleCarryIncByLECtl(n : Int) : Unit {
        use ctl = Qubit();
        use xs = Qubit[n];
        use ys = Qubit[n];

        for isCtl in [false, true] {
            for xsValue in 0..(1 <<< n) - 1 {
                for ysValue in 0..(1 <<< n) - 1 {
                    within {
                        if isCtl {
                            X(ctl);
                        }
                    } apply {
                        ApplyXorInPlace(xsValue, xs);
                        ApplyXorInPlace(ysValue, ys);
                        Controlled RippleCarryIncByLE([ctl], (xs, ys));
                        Fact(MeasureInteger(ys) == (isCtl ? (xsValue + ysValue) % (1 <<< n) | ysValue), $"unexpected value for `ys` given xsValue = {xsValue} and ysValue = {ysValue}");
                        Fact(MeasureInteger(xs) == xsValue, $"unexpected value for `xs` given xsValue = {xsValue} and ysValue = {ysValue}");
                    }
                    ResetAll(xs);
                    ResetAll(ys);
                    Reset(ctl);
                }
            }
        }
    }
    
}
