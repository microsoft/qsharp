namespace Test {
    open Microsoft.Quantum.Unstable.Arithmetic;
    open Microsoft.Quantum.Convert;
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Measurement;

    internal operation TestAddLE3(
        name : String,
        adder : (Qubit[], Qubit[], Qubit[]) => Unit,
        xLen : Int,
        yLen : Int,
        zLen : Int) : Unit {

        use x = Qubit[xLen];
        use y = Qubit[yLen];
        use z = Qubit[zLen];

        for xValue in 0..(1 <<< xLen) - 1 {
            for yValue in 0..(1 <<< yLen) - 1 {
                ApplyXorInPlace(xValue, x);
                ApplyXorInPlace(yValue, y);
                adder(x, y, z);

                let xActual = MeasureInteger(x);
                let yActual = MeasureInteger(y);
                let zActual = MeasureInteger(z);
                let zExpected = (xValue + yValue) % (1 <<< zLen);

                Fact(xActual == xValue,
                   $"{name}: Incorrect x={xActual}, expected={xValue}. |x|={xLen}, |y|={yLen}, |z|={zLen}, x={xValue}, y={yValue}.");
                Fact(yActual == yValue,
                   $"{name}: Incorrect y={yActual}, expected={yValue}. |x|={xLen}, |y|={yLen}, |z|={zLen}, x={xValue}, y={yValue}.");
                Fact(zActual == zExpected,
                   $"{name}: Incorrect z={zActual}, expected={zExpected}. |x|={xLen}, |y|={yLen}, |z|={zLen}, x={xValue}, y={yValue}.");

                ResetAll(x);
                ResetAll(y);
                ResetAll(z);
            }
        }
    }

    internal operation TestAddLE(
        name : String,
        adder : (Qubit[], Qubit[], Qubit[]) => Unit,
        bitwidth : Int) : Unit {

        TestAddLE3(name, adder, bitwidth, bitwidth, bitwidth);
        TestAddLE3(name, adder, bitwidth, bitwidth, bitwidth+1);
        TestAddLE3(name, adder, bitwidth, bitwidth, bitwidth+2);
    }


}
