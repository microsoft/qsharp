import Std.Diagnostics.Fact;
import Std.Arrays.*;
import Std.Random.*;
import ClassicalFunction.Square;
import Measurement.MeasureBasisState;

/// # Summary
/// Test code that verifies the classical values returned by the rest of the code.
/// Throw exceptions if the test fails.

function TestSquare() : Unit {
    for i in -10..10 {
        let (actual, expected) = (Square(i), i * i);
        Fact(actual == expected, $"Incorrect function value for {i}: expected {expected}, got {actual}");
    }
}

operation TestMeasurement() : Unit {
    for _ in 1..10 {
        let n = DrawRandomInt(2, 10);
        let bits = ForEach(x => DrawRandomBool(0.5), [0, size = n]);
        let res = MeasureBasisState(bits);
        for (bit, resBit) in Zipped(bits, res) {
            Fact(bit == (resBit == One), $"Incorrect measurement result for {bit}");
        }
    }
}