namespace Test {
    open Microsoft.Quantum.Arithmetic;
    open Microsoft.Quantum.Convert;
    open Microsoft.Quantum.Diagnostics;

    operation TestRippleCarryAddLE(n : Int) : Unit {

        use xs = Qubit[n];
        use ys = Qubit[n];
        use zs = Qubit[n];

        for xsValue in 0..(1 <<< n) - 1 {
            for ysValue in 0..(1 <<< n) - 1 {
                ApplyXorInPlace(xsValue, xs);
                ApplyXorInPlace(ysValue, ys);
                RippleCarryAddLE(xs, ys, zs);
                Fact(MeasureInteger(xs) == xsValue, $"unexpected value for `xs` given xsValue = {xsValue} and ysValue = {ysValue}");
                Fact(MeasureInteger(ys) == ysValue, $"unexpected value for `ys` given xsValue = {xsValue} and ysValue = {ysValue}");
                Fact(MeasureInteger(zs) == (xsValue + ysValue) % (1 <<< n), $"unexpected value for `zs` given xsValue = {xsValue} and ysValue = {ysValue}");
                ResetAll(xs);
                ResetAll(ys);
                ResetAll(zs);
            }
        }

        use xs = Qubit[n];
        use ys = Qubit[n];
        use zs = Qubit[n + 1];

        for xsValue in 0..(1 <<< n) - 1 {
            for ysValue in 0..(1 <<< n) - 1 {
                ApplyXorInPlace(xsValue, xs);
                ApplyXorInPlace(ysValue, ys);
                RippleCarryAddLE(xs, ys, zs);
                Fact(MeasureInteger(xs) == xsValue, $"unexpected value for `xs` given xsValue = {xsValue} and ysValue = {ysValue}");
                Fact(MeasureInteger(ys) == ysValue, $"unexpected value for `ys` given xsValue = {xsValue} and ysValue = {ysValue}");
                Fact(MeasureInteger(zs) == (xsValue + ysValue) % (1 <<< (n + 1)), $"unexpected value for `zs` given xsValue = {xsValue} and ysValue = {ysValue}");
                ResetAll(xs);
                ResetAll(ys);
                ResetAll(zs);
            }
        }
    }

}
