// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

import Init.PrepareFxP;
import Types.FixedPoint;
import Measurement.MeasureFxP;
import Std.Diagnostics.Fact;
import Std.Convert.IntAsDouble;
import Std.Math.AbsD;
import Operations.*;

operation Main() : Unit {
    FxpMeasurementTest();
    FxpOperationTests();
}

operation FxpMeasurementTest() : Unit {
    for i in 0..100 {
        let constant = 0.1 * IntAsDouble(i);
        TestConstantMeasurement(constant);
    }
}

operation TestConstantMeasurement(constant : Double) : Unit {
    use register = Qubit[20];
    let newFxp = new FixedPoint { IntegerBits = 8, Register = register };
    PrepareFxP(constant, newFxp);
    let measured = MeasureFxP(newFxp);
    let difference = AbsD(constant - measured);
    Fact(difference < 0.001, $"Difference of {difference} is outside of the expected range. Input was {constant} and measured result was {measured}");
    ResetAll(register);
}

operation FxpOperationTests() : Unit {
    for i in 0..50 {
        let constant1 = 0.2 * IntAsDouble(i);
        let constant2 = 0.2 * IntAsDouble(100 - i);
        TestOperation(constant1, constant2, AddFxP, (a, b) -> a + b, "Add");
        TestOperation(constant1, constant2, SubtractFxP, (a, b) -> a - b, "Subtract");
        TestOperation3(constant1, constant2, (a, b, c) => MultiplyFxP(a, b, c), (a, b) -> a * b, "Multiply");
        // manually test square, since it requires higher precision to test well
        TestSquare(constant1);
    }
}
operation TestSquare(a : Double) : Unit {
    Message($"Testing Square({a})");
    use resultRegister = Qubit[30];
    let resultFxp = new FixedPoint { IntegerBits = 8, Register = resultRegister };
    PrepareFxP(0.0, resultFxp);

    use aRegister = Qubit[30];
    let aFxp = new FixedPoint { IntegerBits = 8, Register = aRegister };
    PrepareFxP(a, aFxp);

    SquareFxP(aFxp, resultFxp);
    let measured = MeasureFxP(resultFxp);
    Fact(AbsD(a * a - measured) < 0.001, $"Difference of {AbsD(a * a - measured)} is outside of the expected range. Expected {a * a} and measured result was {measured}. (Inputs were Square({a})");
    ResetAll(resultRegister);
    ResetAll(aRegister);
}

// assume the second register that `op` takes is the result register
operation TestOperation(a : Double, b : Double, op : (FixedPoint, FixedPoint) => (), reference : (Double, Double) -> Double, name : String) : Unit {
    Message($"Testing {name}({a}, {b})");
    use register1 = Qubit[20];
    let aFxp = new FixedPoint { IntegerBits = 8, Register = register1 };
    PrepareFxP(a, aFxp);

    use register2 = Qubit[20];
    let bFxp = new FixedPoint { IntegerBits = 8, Register = register2 };
    PrepareFxP(b, bFxp);

    op(aFxp, bFxp);
    let measured = MeasureFxP(bFxp);

    let expected = reference(a, b);
    let difference = expected - measured;
    Fact(difference < 0.001, $"Difference of {difference} is outside of the expected range. Expected {expected} and measured result was {measured}. (Inputs were {name}({a}, {b})");
    ResetAll(register1 + register2);
}


// assume the third register that `op` takes is the result register
operation TestOperation3(a : Double, b : Double, op : (FixedPoint, FixedPoint, FixedPoint) => (), reference : (Double, Double) -> Double, name : String) : Unit {
    Message($"Testing {name}({a}, {b})");
    use register1 = Qubit[24];
    let aFxp = new FixedPoint { IntegerBits = 8, Register = register1 };
    PrepareFxP(a, aFxp);

    use register2 = Qubit[24];
    let bFxp = new FixedPoint { IntegerBits = 8, Register = register2 };
    PrepareFxP(b, bFxp);

    use resultRegister = Qubit[24];
    let result = new FixedPoint { IntegerBits = 8, Register = resultRegister };

    op(aFxp, bFxp, result);
    let measured = MeasureFxP(result);

    let expected = reference(a, b);
    let difference = expected - measured;
    Fact(difference < 0.001, $"Difference of {difference} is outside of the expected range. Expected {expected} and measured result was {measured}. (Inputs were {name}({a}, {b})");
    ResetAll(register1 + register2);
}