/// # Sample
/// Code generation
///
/// # Description
/// Demonstrates code generation from Q# to QIR.
namespace MyQuantumProgram {
    open Microsoft.Quantum.Math;
    open Microsoft.Quantum.Convert;
    open QIR.Intrinsic;
    @EntryPoint()
    operation Main() : Unit {
        // Even when qubits are allocated, no code is generated until they're actually used.
        use (q0, q1, q2) = (Qubit(), Qubit(), Qubit());

        // Even if you do some computations, we don't generate anything until qubits are used.
        let alpha = ArcCos(0.5);
        let beta = ArcSin(0.5);

        // If we apply gates to qubits we generate code.
        // Note that we only include the operations that are used.
        //__quantum__qis__rx__body(alpha, q0);
        //__quantum__qis__ry__body(beta, q1);

        // We evaluate anything that can be classically evaluated.

        // If expressions with classical conditions.
        //let i = 5;
        //if i > 0 {
        //    __quantum__qis__x__body(q0);
        //}
        //if i > 5 {
        //    __quantum__qis__y__body(q0);
        //} else {
        //    __quantum__qis__z__body(q0);
        //}

        // Loops with classical conditions.
        //for _ in 0..4 {
        //    __quantum__qis__h__body(q0);
        //}
        //for theta in [PI(), 2.0*PI(), 3.0*PI()] {
        //    __quantum__qis__rx__body(theta, q1);
        //}

        // Loops with classical conditions and if expressions.
        //for idx in 0..4 {
        //    if idx % 2 == 0 {
        //        __quantum__qis__ry__body(Sin(PI()/2.0), q0);
        //    } else {
        //        __quantum__qis__ry__body(Cos(PI()/2.0), q1);
        //    }
        //}

        // Branching based on measurement.
        //let r0 = __quantum__qis__mresetz__body(q0);
        //if r0 == One {
        //    __quantum__qis__x__body(q2);
        //}
        //let r1 = __quantum__qis__m__body(q1);
        //if r0 != r1 {
        //    let angle = PI() + PI() + PI()*Sin(PI()/2.0);
        //    __quantum__qis__ry__body(angle, q2);
        //} else {
        //    __quantum__qis__rz__body(PI() + PI() + 2.0*PI()*Sin(PI()/2.0), q2);
        //}
    }
}
