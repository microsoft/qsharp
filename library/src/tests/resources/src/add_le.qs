namespace Test {
    import Std.Arithmetic.*;
    import Std.Convert.*;
    import Std.Diagnostics.*;

    internal operation TestAddLE3(
        name : String,
        adder : (Qubit[], Qubit[], Qubit[]) => Unit,
        xLen : Int,
        yLen : Int,
        zLen : Int
    ) : Unit {

        use x = Qubit[xLen];
        use y = Qubit[yLen];
        use z = Qubit[zLen];

        for carryIn in 0..1 {
            for xValue in 0..(1 <<< xLen) - 1 {
                for yValue in 0..(1 <<< yLen) - 1 {
                    ApplyXorInPlace(xValue, x);
                    ApplyXorInPlace(yValue, y);
                    ApplyXorInPlace(carryIn, z);
                    adder(x, y, z);

                    let xActual = MeasureInteger(x);
                    let yActual = MeasureInteger(y);
                    let zActual = MeasureInteger(z);
                    let zExpected = (xValue + yValue + carryIn) % (1 <<< zLen);

                    Fact(
                        xActual == xValue,
                        $"{name}: Incorrect x={xActual}, expected={xValue}. |x|={xLen}, |y|={yLen}, |z|={zLen}, x={xValue}, y={yValue}, z={carryIn}."
                    );
                    Fact(
                        yActual == yValue,
                        $"{name}: Incorrect y={yActual}, expected={yValue}. |x|={xLen}, |y|={yLen}, |z|={zLen}, x={xValue}, y={yValue}, z={carryIn}."
                    );
                    Fact(
                        zActual == zExpected,
                        $"{name}: Incorrect z={zActual}, expected={zExpected}. |x|={xLen}, |y|={yLen}, |z|={zLen}, x={xValue}, y={yValue}, z={carryIn}."
                    );

                    ResetAll(x);
                    ResetAll(y);
                    ResetAll(z);
                }
            }
        }
    }

    internal operation TestAddLE(
        name : String,
        adder : (Qubit[], Qubit[], Qubit[]) => Unit,
        bitwidth : Int
    ) : Unit {

        TestAddLE3(name, adder, bitwidth, bitwidth, bitwidth);
        TestAddLE3(name, adder, bitwidth, bitwidth, bitwidth + 1);
        TestAddLE3(name, adder, bitwidth, bitwidth, bitwidth + 2);
    }


}
