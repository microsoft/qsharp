// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// stdgates.inc, gates with implied modifiers are omitted as they are mapped
// to the base gate with modifiers in the lowerer.
export gphase, U, p, x, y, z, h, s, sdg, t, tdg, sx, rx, ry, rz, cx, cp, swap, ccx, cu, phase, id, u1, u2, u3;

// gates that qiskit won't emit qasm defs for that are NOT part of stgates.inc
// but are have the _standard_gate property in Qiskit:

// QIR intrinsics missing from qasm std library, that Qiskit won't emit qasm defs for
export rxx, ryy, rzz;

// Remaining gates that are not in the qasm std library, but are standard gates in Qiskit
// that Qiskit wont emit correctly.
export dcx, ecr, r, rzx, cs, csdg, sxdg, csx, cu1, cu3, rccx, c3sqrtx, c3x, rc3x, xx_minus_yy, xx_plus_yy, ccz;

import Angle.*;

import Std.Intrinsic.*;

function ZERO_ANGLE() : __Angle__ {
    return __DoubleAsAngle__(0., 1);
}

function PI_OVER_2() : __Angle__ {
    return __DoubleAsAngle__(Std.Math.PI() / 2., 53);
}

function PI_OVER_4() : __Angle__ {
    return __DoubleAsAngle__(Std.Math.PI() / 4., 53);
}

function PI_OVER_8() : __Angle__ {
    return __DoubleAsAngle__(Std.Math.PI() / 8., 53);
}

function PI_ANGLE() : __Angle__ {
    return __DoubleAsAngle__(Std.Math.PI(), 53);
}

function NEG_PI_OVER_2() : __Angle__ {
    return __DoubleAsAngle__(-Std.Math.PI() / 2., 53);
}

function NEG_PI_OVER_4() : __Angle__ {
    return __DoubleAsAngle__(-Std.Math.PI() / 4., 53);
}

function NEG_PI_OVER_8() : __Angle__ {
    return __DoubleAsAngle__(-Std.Math.PI() / 8., 53);
}

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
    Adjoint R(PauliI, Std.Math.PI() / 2., qubit);
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
    U(ZERO_ANGLE(), ZERO_ANGLE(), lambda, qubit);
}

operation id(qubit : Qubit) : Unit is Adj + Ctl {
    I(qubit)
}

operation u1(lambda : __Angle__, qubit : Qubit) : Unit is Adj + Ctl {
    U(ZERO_ANGLE(), ZERO_ANGLE(), lambda, qubit);
}

operation u2(phi : __Angle__, lambda : __Angle__, qubit : Qubit) : Unit is Adj + Ctl {
    gphase(__NegAngle__(__DivideAngleByInt__(__AddAngles__(
        phi,
        __AddAngles__(
            lambda,
            PI_OVER_2()
        )
    ), 2)));

    U(PI_OVER_2(), phi, lambda, qubit);
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


/// rxx: gate rxx(theta) a, b { h a; h b; cx a, b; rz(theta) b; cx a, b; h b; h a; }
operation rxx(theta : __Angle__, qubit0 : Qubit, qubit1 : Qubit) : Unit is Adj + Ctl {
    Rxx(__AngleAsDouble__(theta), qubit0, qubit1);
}

/// ryy: gate ryy(theta) a, b { rx(pi/2) a; rx(pi/2) b; cx a, b; rz(theta) b; cx a, b; rx(-pi/2) a; rx(-pi/2) b; }
operation ryy(theta : __Angle__, qubit0 : Qubit, qubit1 : Qubit) : Unit is Adj + Ctl {
    Ryy(__AngleAsDouble__(theta), qubit0, qubit1);
}

/// rzz: gate rzz(theta) a, b { cx a, b; u1(theta) b; cx a, b; }
operation rzz(theta : __Angle__, qubit0 : Qubit, qubit1 : Qubit) : Unit is Adj + Ctl {
    Rzz(__AngleAsDouble__(theta), qubit0, qubit1);
}

/// Double-CNOT gate.
/// ```
/// gate dcx a, b {
///     cx a, b;
///     cx b, a;
/// }
/// ```
operation dcx(qubit0 : Qubit, qubit1 : Qubit) : Unit is Adj + Ctl {
    cx(qubit0, qubit1);
    cx(qubit1, qubit0);
}

/// An echoed cross-resonance gate.
/// `gate ecr a, b { rzx(pi/4) a, b; x a; rzx(-pi/4) a, b; }`
operation ecr(qubit0 : Qubit, qubit1 : Qubit) : Unit is Adj + Ctl {
    rzx(PI_OVER_4(), qubit0, qubit1);
    x(qubit0);
    rzx(NEG_PI_OVER_4(), qubit0, qubit1);
}

/// Rotation θ around the cos(φ)x + sin(φ)y axis.
/// `gate r(θ, φ) a {u3(θ, φ - π/2, -φ + π/2) a;}`
operation r(theta : __Angle__, phi : __Angle__, qubit : Qubit) : Unit is Adj + Ctl {
    u3(theta, PI_OVER_4(), __SubtractAngles__(
        phi,
        NEG_PI_OVER_2()
    ), qubit);
}

/// A parametric 2-qubit `Z ⊗ X` interaction (rotation about ZX).
/// `gate rzx(theta) a, b { h b; cx a, b; u1(theta) b; cx a, b; h b; }`
operation rzx(theta : __Angle__, qubit0 : Qubit, qubit1 : Qubit) : Unit is Adj + Ctl {
    h(qubit1);
    cx(qubit0, qubit1);
    u1(theta, qubit1);
    cx(qubit0, qubit1);
    h(qubit1);
}


/// Controlled-S gate.
/// `gate cs a,b { h b; cp(pi/2) a,b; h b; }`
operation cs(qubit0 : Qubit, qubit1 : Qubit) : Unit is Adj + Ctl {
    Controlled s([qubit1], qubit0);
}

/// Controlled-S† gate.
/// csdg: gate csdg a,b { h b; cp(-pi/2) a,b; h b; }
operation csdg(qubit0 : Qubit, qubit1 : Qubit) : Unit is Adj + Ctl {
    Controlled Adjoint S([qubit1], qubit0);
}

/// The inverse single-qubit Sqrt(X) gate.
/// `gate sxdg a { rz(pi/2) a; h a; rz(pi/2); }`
operation sxdg(qubit : Qubit) : Unit is Adj + Ctl {
    Adjoint sx(qubit);
}

// Controlled-√X gate.
/// `gate csx a,b { h b; cu1(pi/2) a,b; h b; }`
operation csx(qubit0 : Qubit, qubit1 : Qubit) : Unit is Adj + Ctl {
    Controlled sx([qubit1], qubit0);
}

/// Controlled-U1 gate.
/// `ctrl @ u1(lambda) a, b` or:
/// ```
/// gate cu1(lambda) a,b {
///     u1(lambda/2) a;
///     cx a,b;
///     u1(-lambda/2) b;
///     cx a,b;
///     u1(lambda/2) b;
/// }
/// ```
operation cu1(lambda : __Angle__, ctrl : Qubit, target : Qubit) : Unit is Adj + Ctl {
    Controlled u1([ctrl], (lambda, target));
}

/// Controlled-U3 gate (3-parameter two-qubit gate).
/// `ctrl @ u3(theta, phi, lambda) a, b` or:
/// ```
/// gate cu3(theta,phi,lambda) c, t {
///     u1((lambda+phi)/2) c;
///     u1((lambda-phi)/2) t;
///     cx c,t;
///     u3(-theta/2,0,-(phi+lambda)/2) t;
///     cx c,t;
///     u3(theta/2,phi,0) t;
/// }
/// ```
operation cu3(theta : __Angle__, phi : __Angle__, lambda : __Angle__, ctrl : Qubit, target : Qubit) : Unit is Adj + Ctl {
    Controlled u3([ctrl], (theta, phi, lambda, target));
}

/// The simplified Toffoli gate, also referred to as Margolus gate.
/// `gate rccx a,b,c { u2(0,pi) c; u1(pi/4) c; cx b, c; u1(-pi/4) c; cx a, c; u1(pi/4) c; cx b, c; u1(-pi/4) c; u2(0,pi) c; }`
operation rccx(ctrl1 : Qubit, ctrl2 : Qubit, target : Qubit) : Unit is Adj + Ctl {
    u2(ZERO_ANGLE(), PI_ANGLE(), target);
    u1(PI_OVER_4(), target);
    cx(ctrl2, target);
    u1(NEG_PI_OVER_4(), target);
    cx(ctrl1, target);
    u1(PI_OVER_4(), target);
    cx(ctrl2, target);
    u1(NEG_PI_OVER_4(), target);
    u2(ZERO_ANGLE(), PI_ANGLE(), target);
}

/// c3sx/c3sqrtx: The 3-qubit controlled sqrt-X gate.
/// ```
/// gate c3sqrtx a,b,c,d {
///     h d;
///     cu1(pi/8) a,d;
///     h d; cx a,b;
///     h d;
///     cu1(-pi/8) b,d;
///     h d; cx a,b; h d;
///     cu1(pi/8) b,d;
///     h d;
///     cx b,c;
///     h d;
///     cu1(-pi/8) c,d;
///     h d;
///     cx a,c;
///     h d;
///     cu1(pi/8) c,d;
///     h d;
///     cx b,c;
///     h d;
///     cu1(-pi/8) c,d;
///     h d;
///     cx a,c;
///     h d;
///     cu1(pi/8) c,d;
///     h d;
/// }
/// ```
operation c3sqrtx(a : Qubit, b : Qubit, c : Qubit, target : Qubit) : Unit is Adj + Ctl {
    h(target);
    Controlled u1([a], (PI_OVER_8(), target));
    h(target);
    cx(a, b);
    h(target);
    Controlled u1([b], (NEG_PI_OVER_8(), target));
    h(target);
    cx(a, b);
    h(target);
    Controlled u1([b], (PI_OVER_8(), target));
    h(target);
    cx(b, c);
    h(target);
    Controlled u1([c], (NEG_PI_OVER_8(), target));
    h(target);
    cx(a, c);
    h(target);
    Controlled u1([c], (PI_OVER_8(), target));
    h(target);
    cx(b, c);
    h(target);
    Controlled u1([c], (NEG_PI_OVER_8(), target));
    h(target);
    cx(a, c);
    h(target);
    Controlled u1([c], (PI_OVER_8(), target));
    h(target);
}

/// The X gate controlled on 3 qubits.
/// ```
/// gate c3x a,b,c,d
/// {
///     h d;
///     p(pi/8) a;
///     p(pi/8) b;
///     p(pi/8) c;
///     p(pi/8) d;
///     cx a, b;
///     p(-pi/8) b;
///     cx a, b;
///     cx b, c;
///     p(-pi/8) c;
///     cx a, c;
///     p(pi/8) c;
///     cx b, c;
///     p(-pi/8) c;
///     cx a, c;
///     cx c, d;
///     p(-pi/8) d;
///     cx b, d;
///     p(pi/8) d;
///     cx c, d;
///     p(-pi/8) d;
///     cx a, d;
///     p(pi/8) d;
///     cx c, d;
///     p(-pi/8) d;
///     cx b, d;
///     p(pi/8) d;
///     cx c, d;
///     p(-pi/8) d;
///     cx a, d;
///     h d;
/// }
/// ```
operation c3x(a : Qubit, b : Qubit, c : Qubit, d : Qubit) : Unit is Adj + Ctl {
    h(d);
    p(PI_OVER_8(), a);
    p(PI_OVER_8(), b);
    p(PI_OVER_8(), c);
    p(PI_OVER_8(), d);
    cx(a, b);
    p(NEG_PI_OVER_8(), b);
    cx(a, b);
    cx(b, c);
    p(NEG_PI_OVER_8(), c);
    cx(a, c);
    p(PI_OVER_8(), c);
    cx(b, c);
    p(NEG_PI_OVER_8(), c);
    cx(a, c);
    cx(c, d);
    p(NEG_PI_OVER_8(), d);
    cx(b, d);
    p(PI_OVER_8(), d);
    cx(c, d);
    p(NEG_PI_OVER_8(), d);
    cx(a, d);
    p(PI_OVER_8(), d);
    cx(c, d);
    p(NEG_PI_OVER_8(), d);
    cx(b, d);
    p(PI_OVER_8(), d);
    cx(c, d);
    p(NEG_PI_OVER_8(), d);
    cx(a, d);
    h(d);
}

/// Simplified 3-controlled Toffoli gate.
/// ```
/// gate rc3x a,b,c,d{
///     u2(0,pi) d;
///     u1(pi/4) d;
///     cx c,d;
///     u1(-pi/4) d;
///     u2(0,pi) d;
///     cx a,d;
///     u1(pi/4) d;
///     cx b,d;
///     u1(-pi/4) d;
///     cx a,d;
///     u1(pi/4) d;
///     cx b,d;
///     u1(-pi/4) d;
///     u2(0,pi) d;
///     u1(pi/4) d;
///     cx c,d;
///     u1(-pi/4) d;
///     u2(0,pi) d;
/// }
/// ```
operation rc3x(a : Qubit, b : Qubit, c : Qubit, d : Qubit) : Unit is Adj + Ctl {
    u2(ZERO_ANGLE(), PI_ANGLE(), d);
    u1(PI_OVER_4(), d);
    cx(c, d);
    u1(NEG_PI_OVER_4(), d);
    u2(ZERO_ANGLE(), PI_ANGLE(), d);
    cx(a, d);
    u1(PI_OVER_4(), d);
    cx(b, d);
    u1(NEG_PI_OVER_4(), d);
    cx(a, d);
    u1(PI_OVER_4(), d);
    cx(b, d);
    u1(NEG_PI_OVER_4(), d);
    u2(ZERO_ANGLE(), PI_ANGLE(), d);
    u1(PI_OVER_4(), d);
    cx(c, d);
    u1(NEG_PI_OVER_4(), d);
    u2(ZERO_ANGLE(), PI_ANGLE(), d);
}

/// XX-YY gate.
/// ```
/// gate xx_minus_yy(theta, beta) a, b {
///     rz(-beta) b;
///     rz(-pi/2) a;
///     sx a;
///     rz(pi/2) a;
///     s b;
///     cx a, b;
///     ry(theta/2) a;
///     ry(-theta/2) b;
///     cx a, b;
///     sdg b;
///     rz(-pi/2) a;
///     sxdg a;
///     rz(pi/2) a;
///     rz(beta) b;
/// }
/// ```
operation xx_minus_yy(theta : __Angle__, beta : __Angle__, qubit0 : Qubit, qubit1 : Qubit) : Unit is Adj + Ctl {
    rz(__NegAngle__(beta), qubit1);
    rz(NEG_PI_OVER_2(), qubit0);
    sx(qubit0);
    rz(PI_OVER_2(), qubit0);
    s(qubit1);
    cx(qubit0, qubit1);
    ry(__DivideAngleByInt__(theta, 2), qubit0);
    ry(__NegAngle__(__DivideAngleByInt__(theta, 2)), qubit1);
    cx(qubit0, qubit1);
    sdg(qubit1);
    rz(NEG_PI_OVER_2(), qubit0);
    sxdg(qubit0);
    rz(PI_OVER_2(), qubit0);
    rz(beta, qubit1);
}

/// XX+YY gate.
/// ```
/// gate xx_plus_yy(theta, beta) a, b {
///     rz(beta) b;
///     rz(-pi/2) a;
///     sx a;
///     rz(pi/2) a;
///     s b;
///     cx a, b;
///     ry(theta/2) a;
///     ry(theta/2) b;
///     cx a, b;
///     sdg b;
///     rz(-pi/2) a;
///     sxdg a;
///     rz(pi/2) a;
///     rz(-beta) b;
/// }
/// ```
operation xx_plus_yy(theta : __Angle__, beta : __Angle__, qubit0 : Qubit, qubit1 : Qubit) : Unit is Adj + Ctl {
    rz(beta, qubit1);
    rz(NEG_PI_OVER_2(), qubit0);
    sx(qubit0);
    rz(PI_OVER_2(), qubit0);
    s(qubit1);
    cx(qubit0, qubit1);
    ry(__DivideAngleByInt__(theta, 2), qubit0);
    ry(__DivideAngleByInt__(theta, 2), qubit1);
    cx(qubit0, qubit1);
    sdg(qubit1);
    rz(NEG_PI_OVER_2(), qubit0);
    sxdg(qubit0);
    rz(PI_OVER_2(), qubit0);
    rz(__NegAngle__(beta), qubit1);
}

/// CCZ gate.
/// `gate ccz a,b,c { h c; ccx a,b,c; h c; }`
operation ccz(ctrl1 : Qubit, ctrl2 : Qubit, target : Qubit) : Unit is Adj + Ctl {
    h(target);
    ccx(ctrl1, ctrl2, target);
    h(target);
}
