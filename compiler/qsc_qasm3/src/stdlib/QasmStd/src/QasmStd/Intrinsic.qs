// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

export gphase, U, p, x, y, z, h, s, sdg, t, tdg, sx, rx, ry, rz, cx, cp, swap, ccx, cu, phase, id, u1, u2, u3;

import Angle.*;

import Std.Intrinsic.*;


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

operation p(lambda : __Angle__, qubit : Qubit) : Unit is Adj + Ctl {
    Controlled gphase([qubit], lambda);
}

operation x(qubit : Qubit) : Unit is Adj + Ctl {
    X(qubit);
}

operation y(qubit : Qubit) : Unit is Adj + Ctl {
    Y(qubit);
}

operation z(qubit : Qubit) : Unit is Adj + Ctl {
    Z(qubit);
}

operation h(qubit : Qubit) : Unit is Adj + Ctl {
    H(qubit);
}

operation s(qubit : Qubit) : Unit is Adj + Ctl {
    S(qubit);
}

operation sdg(qubit : Qubit) : Unit is Adj + Ctl {
    Adjoint S(qubit);
}

operation t(qubit : Qubit) : Unit is Adj + Ctl {
    T(qubit);
}

operation tdg(qubit : Qubit) : Unit is Adj + Ctl {
    Adjoint T(qubit);
}

operation sx(qubit : Qubit) : Unit is Adj + Ctl {
    Rx(Std.Math.PI() / 2., qubit);
}

operation rx(theta : __Angle__, qubit : Qubit) : Unit is Adj + Ctl {
    let theta = __AngleAsDouble__(theta);
    Rx(theta, qubit);
}

operation ry(theta : __Angle__, qubit : Qubit) : Unit is Adj + Ctl {
    let theta = __AngleAsDouble__(theta);
    Ry(theta, qubit);
}

operation rz(theta : __Angle__, qubit : Qubit) : Unit is Adj + Ctl {
    let theta = __AngleAsDouble__(theta);
    Rz(theta, qubit);
}

operation cx(ctrl : Qubit, qubit : Qubit) : Unit is Adj + Ctl {
    CNOT(ctrl, qubit);
}

operation cp(lambda : __Angle__, ctrl : Qubit, qubit : Qubit) : Unit is Adj + Ctl {
    Controlled p([ctrl], (lambda, qubit));
}

operation swap(qubit1 : Qubit, qubit2 : Qubit) : Unit is Adj + Ctl {
    SWAP(qubit1, qubit2);
}

operation ccx(ctrl1 : Qubit, ctrl2 : Qubit, target : Qubit) : Unit is Adj + Ctl {
    CCNOT(ctrl1, ctrl2, target);
}

operation cu(theta : __Angle__, phi : __Angle__, lambda : __Angle__, gamma : __Angle__, qubit1 : Qubit, qubit2 : Qubit) : Unit is Adj + Ctl {
    p(__SubtractAngles__(gamma, __DivideAngleByInt__(theta, 2)), qubit1);
    Controlled U([qubit2], (theta, phi, lambda, qubit1));
}

// Gates for OpenQASM 2 backwards compatibility
operation phase(lambda : __Angle__, qubit : Qubit) : Unit is Adj + Ctl {
    U(__DoubleAsAngle__(0., 1), __DoubleAsAngle__(0., 1), lambda, qubit);
}

operation id(qubit : Qubit) : Unit is Adj + Ctl {
    I(qubit)
}

operation u1(lambda : __Angle__, qubit : Qubit) : Unit is Adj + Ctl {
    U(__DoubleAsAngle__(0., 1), __DoubleAsAngle__(0., 1), lambda, qubit);
}

operation u2(phi : __Angle__, lambda : __Angle__, qubit : Qubit) : Unit is Adj + Ctl {
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

operation u3(theta : __Angle__, phi : __Angle__, lambda : __Angle__, qubit : Qubit) : Unit is Adj + Ctl {
    gphase(__NegAngle__(__DivideAngleByInt__(__AddAngles__(
        phi,
        __AddAngles__(
            lambda,
            theta
        )
    ), 2)));

    U(theta, phi, lambda, qubit);
}

