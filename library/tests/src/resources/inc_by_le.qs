namespace Test {
    open Microsoft.Quantum.Unstable.Arithmetic;
    open Microsoft.Quantum.Convert;
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Measurement;

    internal operation TestIncByLE2(
        name : String,
        adder : (Qubit[], Qubit[]) => Unit,
        xLen : Int,
        yLen : Int) : Unit {

        use x = Qubit[xLen];
        use y = Qubit[yLen];

        for xValue in 0..(1 <<< xLen) - 1 {
            for yValue in 0..(1 <<< yLen) - 1 {
                ApplyXorInPlace(xValue, x);
                ApplyXorInPlace(yValue, y);
                adder(x, y);

                let yActual = MeasureInteger(y);
                let xActual = MeasureInteger(x);
                let yExpected = (xValue + yValue) % (1 <<< yLen);

                Fact(yActual == yExpected,
                    $"{name}: Incorrect sum={yActual}, expected={yExpected}. |x|={xLen}, |y|={yLen}, x={xValue}, y={yValue}.");
                Fact(xActual == xValue,
                    $"{name}: Incorrect x={xActual}, expected={xValue}. |x|={xLen}, |y|={yLen}, x={xValue}, y={yValue}.");

                ResetAll(x);
                ResetAll(y);
            }
        }
    }

    internal operation TestIncByLECtl2(
        name : String,
        adder : (Qubit[], Qubit[]) => Unit is Ctl,
        xLen : Int,
        yLen : Int) : Unit {

        use ctl = Qubit();
        use x = Qubit[xLen];
        use y = Qubit[yLen];

        for isCtl in [false, true] {
            for xValue in 0..(1 <<< xLen) - 1 {
                for yValue in 0..(1 <<< yLen) - 1 {
                    if isCtl {
                        X(ctl);
                    }
                    ApplyXorInPlace(xValue, x);
                    ApplyXorInPlace(yValue, y);
                    Controlled adder([ctl], (x, y));

                    let yActual = MeasureInteger(y);
                    let xActual = MeasureInteger(x);
                    let yExpected = isCtl ? (xValue + yValue) % (1 <<< yLen) | yValue;

                    Fact(yActual == yExpected,
                        $"{name}: Incorrect sum={yActual}, expected={yExpected}. ctl={isCtl}, |x|={xLen}, |y|={yLen}, x={xValue}, y={yValue}.");
                    Fact(xActual == xValue,
                        $"{name}: Incorrect x={xActual}, expected={xValue}. ctl={isCtl}, |x|={xLen}, |y|={yLen}, x={xValue}, y={yValue}.");
                    
                    ResetAll(x);
                    ResetAll(y);
                    Reset(ctl);
                }
            }
        }
    }

    internal operation TestIncByLE(
        name : String,
        adder : (Qubit[], Qubit[]) => Unit,
        bitwidth : Int) : Unit {

        TestIncByLE2(name, adder, bitwidth, bitwidth);
        TestIncByLE2(name, adder, bitwidth, bitwidth+1);
        TestIncByLE2(name, adder, bitwidth, bitwidth+2);
    }

    internal operation TestIncByLECtl(
        name : String,
        adder : (Qubit[], Qubit[]) => Unit is Ctl,
        bitwidth : Int) : Unit {

        TestIncByLECtl2(name, adder, bitwidth, bitwidth);
    }

}
