import Init.PrepareFxP;
import Types.FixedPoint;
import Measurement.MeasureFxP;
import Std.Diagnostics.Fact;
import Std.Convert.IntAsDouble;

operation Main() : Unit {
    FxpOperationTests();
}

operation FxpOperationTests() : Unit {
    for i in 0..100 {
        let constant = 0.1 * IntAsDouble(i);
        TestConstantMeasurement(constant);
    }
}

operation TestConstantMeasurement(constant: Double) : Unit {
    use register = Qubit[4];
    let newFxp = new FixedPoint { IntegerBits = 10, Register = register };
    PrepareFxP(constant, newFxp);
    let measured = MeasureFxP(newFxp);
    Fact(constant == measured, $"Expected {constant}, got {measured}");
}