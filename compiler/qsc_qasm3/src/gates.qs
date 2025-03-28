// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

struct __Angle__ {
    Value : Int,
    Size : Int
}

function __AngleAsBoolArrayBE__(angle : __Angle__) : Bool[] {
    Microsoft.Quantum.Arrays.Reversed(Std.Convert.IntAsBoolArray(angle.Value, angle.Size))
}

function __AngleAsDouble__(angle : __Angle__) : Double {
    let F64_MANTISSA_DIGITS = 53;
    let (value, size) = if angle.Size > F64_MANTISSA_DIGITS {
        __ConvertAngleToWidth__(angle, F64_MANTISSA_DIGITS, false)!
    } else {
        angle!
    };
    let denom = Std.Convert.IntAsDouble(1 <<< size);
    let value = Std.Convert.IntAsDouble(value);
    let factor = (2.0 * Std.Math.PI()) / denom;
    value * factor
}

function __AngleAsBool__(angle : __Angle__) : Bool {
    return angle.Value != 0;
}

function __IntAsAngle__(value : Int, size : Int) : __Angle__ {
    Std.Diagnostics.Fact(value >= 0, "Value must be >= 0");
    Std.Diagnostics.Fact(size > 0, "Size must be > 0");
    new __Angle__ { Value = value, Size = size }
}

function __DoubleAsAngle__(value : Double, size : Int) : __Angle__ {
    let tau: Double = 2. * Std.Math.PI();

    mutable value = value % tau;
    if value < 0. {
        value = value + tau;
    }

    Std.Diagnostics.Fact(value >= 0., "Value must be >= 0.");
    Std.Diagnostics.Fact(value < tau, "Value must be < tau.");
    Std.Diagnostics.Fact(size > 0, "Size must be > 0");
    function RoundHalfAwayFromZero(value : Double) : Int {
        let roundedValue = Microsoft.Quantum.Math.Round(value);
        let EPSILON = 2.2204460492503131e-16;
        let diff = Std.Math.AbsD(value - Std.Convert.IntAsDouble(roundedValue));
        if (Std.Math.AbsD(diff - 0.5) < EPSILON) {
            if (value > 0.0) {
                return roundedValue + 1;
            } else {
                return roundedValue - 1;
            }
        } else {
            return roundedValue;
        }
    }

    let factor = tau / Std.Convert.IntAsDouble(1 <<< size);
    let value = RoundHalfAwayFromZero(value / factor);
    new __Angle__ { Value = value, Size = size }
}

function __ConvertAngleToWidth__(angle : __Angle__, new_size : Int, truncate : Bool) : __Angle__ {
    let (value, size) = angle!;
    if new_size < size {
        let value = if truncate {
            let shift_amount = size - new_size;
            value >>> shift_amount
        } else {
            // Rounding
            let shift_amount = size - new_size;
            let half = 1 <<< (shift_amount - 1);
            let mask = (1 <<< shift_amount) - 1;
            let lower_bits = value &&& mask;
            let upper_bits = value >>> shift_amount;
            if lower_bits > half or (lower_bits == half and (upper_bits &&& 1) == 1) {
                upper_bits + 1
            } else {
                upper_bits
            }
        };
        new __Angle__ { Value = value, Size = size }
    } elif new_size == size {
        // Same size, no change
        angle
    } else {
        // Padding with zeros
        let value = value <<< (new_size - size);
        new __Angle__ { Value = value, Size = size }
    }
}

function __AddAngles__(lhs : __Angle__, rhs : __Angle__) : __Angle__ {
    let (lhs_value, lhs_size) = lhs!;
    let (rhs_value, rhs_size) = rhs!;
    Std.Diagnostics.Fact(lhs_size == rhs_size, "Angle sizes must be the same");
    let value = (lhs_value + rhs_value) % (1 <<< lhs_size);
    new __Angle__ { Value = value, Size = lhs_size }
}

function __SubtractAngles__(lhs : __Angle__, rhs : __Angle__) : __Angle__ {
    let (lhs_value, lhs_size) = lhs!;
    let (rhs_value, rhs_size) = rhs!;
    Std.Diagnostics.Fact(lhs_size == rhs_size, "Angle sizes must be the same");
    let value = (lhs_value + ((1 <<< lhs_size) - rhs_value)) % (1 <<< lhs_size);
    new __Angle__ { Value = value, Size = lhs_size }
}

function __MultiplyAngleByInt__(angle : __Angle__, factor : Int) : __Angle__ {
    let (value, size) = angle!;
    let value = (value * factor) % (1 <<< size);
    new __Angle__ { Value = value, Size = size }
}

function __MultiplyAngleByBigInt__(angle : __Angle__, factor : BigInt) : __Angle__ {
    let (value, size) = angle!;
    let value : BigInt = Std.Convert.IntAsBigInt(value);
    let value = (value * factor) % Std.Convert.IntAsBigInt(1 <<< size);
    let value = Std.Convert.BoolArrayAsInt(Std.Convert.BigIntAsBoolArray(value, size));
    new __Angle__ { Value = value, Size = size }
}

function __DivideAngleByInt__(angle : __Angle__, divisor : Int) : __Angle__ {
    let (value, size) = angle!;
    let value = value / divisor;
    new __Angle__ { Value = value, Size = size }
}

function __NegAngle__(angle : __Angle__) : __Angle__ {
    let (value, size) = angle!;
    let value = (1 <<< value) - value;
    new __Angle__ { Value = value, Size = size }
}


// Standard gates.

operation gphase(theta : __Angle__) : Unit is Adj + Ctl {
    body ... {
        Exp([], __AngleAsDouble__(theta), [])
    }
    adjoint auto;
    controlled auto;
    controlled adjoint auto;
}

operation U(theta : __Angle__, phi : __Angle__, lambda : __Angle__, qubit : Qubit) : Unit is Adj + Ctl {
    body ... {
        let theta = __AngleAsDouble__(theta);
        let phi = __AngleAsDouble__(phi);
        let lambda = __AngleAsDouble__(lambda);

        Rz(lambda, qubit);
        Ry(theta, qubit);
        Rz(phi, qubit);
        R(PauliI, -lambda - phi - theta, qubit);
    }
    adjoint auto;
    controlled auto;
    controlled adjoint auto;
}

operation p(lambda: __Angle__, qubit: Qubit) : Unit is Adj + Ctl {
    Controlled gphase([qubit], lambda);
}

operation x(qubit: Qubit) : Unit is Adj + Ctl {
    X(qubit);
}

operation y(qubit: Qubit) : Unit is Adj + Ctl {
    Y(qubit);
}

operation z(qubit: Qubit) : Unit is Adj + Ctl {
    Z(qubit);
}

operation h(qubit: Qubit) : Unit is Adj + Ctl {
    H(qubit);
}

operation s(qubit: Qubit) : Unit is Adj + Ctl {
    S(qubit);
}

operation sdg(qubit: Qubit) : Unit is Adj + Ctl {
    Adjoint S(qubit);
}

operation t(qubit: Qubit) : Unit is Adj + Ctl {
    T(qubit);
}

operation tdg(qubit: Qubit) : Unit is Adj + Ctl {
    Adjoint T(qubit);
}

operation sx(qubit: Qubit) : Unit is Adj + Ctl {
    Rx(Std.Math.PI() / 2., qubit);
}

operation rx(theta: __Angle__, qubit: Qubit) : Unit is Adj + Ctl {
    let theta = __AngleAsDouble__(theta);
    Rx(theta, qubit);
}

operation ry(theta: __Angle__, qubit: Qubit) : Unit is Adj + Ctl {
    let theta = __AngleAsDouble__(theta);
    Ry(theta, qubit);
}

operation rz(theta: __Angle__, qubit: Qubit) : Unit is Adj + Ctl {
    let theta = __AngleAsDouble__(theta);
    Rz(theta, qubit);
}

operation cx(ctrl: Qubit, qubit: Qubit) : Unit is Adj + Ctl {
    CNOT(ctrl, qubit);
}

operation cp(lambda: __Angle__, ctrl: Qubit, qubit: Qubit) : Unit is Adj + Ctl {
    Controlled p([ctrl], (lambda, qubit));
}

operation swap(qubit1: Qubit, qubit2: Qubit) : Unit is Adj + Ctl {
    SWAP(qubit1, qubit2);
}

operation ccx(ctrl1: Qubit, ctrl2: Qubit, target: Qubit) : Unit is Adj + Ctl {
    CCNOT(ctrl1, ctrl2, target);
}

operation cu(theta: __Angle__, phi: __Angle__, lambda: __Angle__, gamma: __Angle__, qubit1: Qubit, qubit2: Qubit) : Unit is Adj + Ctl {
    p(__SubtractAngles__(gamma,  __DivideAngleByInt__(theta, 2)), qubit1);
    Controlled U([qubit2], (theta, phi, lambda, qubit1));
}

// Gates for OpenQASM 2 backwards compatibility
operation phase(lambda: __Angle__, qubit: Qubit) : Unit is Adj + Ctl {
    U(__DoubleAsAngle__(0., 1), __DoubleAsAngle__(0., 1), lambda, qubit);
}

operation id(qubit: Qubit) : Unit is Adj + Ctl {
    I(qubit)
}

operation u1(lambda: __Angle__, qubit: Qubit) : Unit is Adj + Ctl {
    U(__DoubleAsAngle__(0., 1), __DoubleAsAngle__(0., 1), lambda, qubit);
}

operation u2(phi: __Angle__, lambda: __Angle__, qubit: Qubit) : Unit is Adj + Ctl {
    let half_pi = __DivideAngleByInt__(__DoubleAsAngle__(Std.Math.PI(), 53), 2);

    gphase(__NegAngle__(__DivideAngleByInt__(__AddAngles__(
        phi,
        __AddAngles__(
            lambda,
            half_pi
        )
    ), 2)));

    U(half_pi, phi, lambda, qubit);
}

operation u3(theta: __Angle__, phi: __Angle__, lambda: __Angle__, qubit: Qubit) : Unit is Adj + Ctl {
    gphase(__NegAngle__(__DivideAngleByInt__(__AddAngles__(
        phi,
        __AddAngles__(
            lambda,
            theta
        )
    ), 2)));

    U(theta, phi, lambda, qubit);
}
