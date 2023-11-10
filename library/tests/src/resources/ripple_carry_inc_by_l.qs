namespace Test {
    open Microsoft.Quantum.Arithmetic;
    open Microsoft.Quantum.Convert;
    open Microsoft.Quantum.Diagnostics;

    operation TestRippleCarryIncByL(n : Int) : Unit {
        use ys = Qubit[n];

        for c in 0..(1 <<< n) - 1 {
            for ysValue in 0..(1 <<< n) - 1 {
                ApplyXorInPlace(ysValue, ys);
                RippleCarryIncByL(IntAsBigInt(c), ys);
                Fact(MeasureInteger(ys) == (c + ysValue) % (1 <<< n), $"unexpected value for `ys` given c = {c} and ysValue = {ysValue}");
                ResetAll(ys);
            }
        }
    }

    operation TestRippleCarryIncByLCtl(n : Int) : Unit {
        use ctl = Qubit();
        use ys = Qubit[n];

        for isCtl in [false, true] {
            for c in 0..(1 <<< n) - 1 {
                for ysValue in 0..(1 <<< n) - 1 {
                    within {
                        if isCtl {
                            X(ctl);
                        }
                    } apply {
                        ApplyXorInPlace(ysValue, ys);
                        Controlled RippleCarryIncByL([ctl], (IntAsBigInt(c), ys));
                        Fact(MeasureInteger(ys) == (isCtl ? (c + ysValue) % (1 <<< n) | ysValue), $"unexpected value for `ys` given c = {c} and ysValue = {ysValue}");
                    }
                    ResetAll(ys);
                    Reset(ctl);
                }
            }
        }
    }

}
