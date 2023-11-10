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
                Fact(MeasureInteger(ys) == (xsValue + ysValue) % (1 <<< n),
                    $"TestRippleCarryIncByLE (|xs|=|ys|): Incorrect ys for xs={xsValue}, ys={ysValue}");
                Fact(MeasureInteger(xs) == xsValue,
                    $"TestRippleCarryIncByLE (|xs|=|ys|): Incorrect xs for xs={xsValue}, ys={ysValue}");
                ResetAll(xs);
                ResetAll(ys);
            }
        }

        let ysL = n+1;
        use xs = Qubit[n];
        use ys = Qubit[ysL];

        for xsValue in 0..(1 <<< n) - 1 {
            for ysValue in 0..(1 <<< ysL) - 1 {
                ApplyXorInPlace(xsValue, xs);
                ApplyXorInPlace(ysValue, ys);
                RippleCarryIncByLE(xs, ys);
                Fact(MeasureInteger(ys) == (xsValue + ysValue) % (1 <<< ysL),
                    $"TestRippleCarryIncByLE (|xs|<|ys|): Incorrect ys for xs={xsValue}, ys={ysValue}");
                Fact(MeasureInteger(xs) == xsValue,
                    $"TestRippleCarryIncByLE (|xs|<|ys|): Incorrect xs for xs={xsValue}, ys={ysValue}");
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
                        Fact(MeasureInteger(ys) == (isCtl ? (xsValue + ysValue) % (1 <<< n) | ysValue),
                            $"TestRippleCarryIncByLECtl: Incorrect ys for xs={xsValue}, ys={ysValue}");
                        Fact(MeasureInteger(xs) == xsValue,
                            $"TestRippleCarryIncByLECtl: Incorrect xs for xs={xsValue}, ys={ysValue}");
                    }
                    ResetAll(xs);
                    ResetAll(ys);
                    Reset(ctl);
                }
            }
        }
    }
    
}
