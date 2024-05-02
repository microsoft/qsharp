/// # Sample
/// Adaptive Quantum Program
///
/// # Description
/// Demonstrates code generation from Q# to QIR.
namespace Adaptive {
    open Microsoft.Quantum.Measurement;
    open Microsoft.Quantum.Math;
    open Microsoft.Quantum.Convert;
    @EntryPoint()
    operation Main() : Result {
        // A minimal adaptive (aka integrated hybrid) program.
        use (q0, q1) = (Qubit(), Qubit());
        H(q0);
        let r0 = MResetZ(q0);

        // Quantinuum supports dynamic bools and ints, which means
        // bools and ints that depend on measurement results.
        //let dynamicBool = r0 != Zero;
        //let dynamicBool = ResultAsBool(r0);
        //let dynamicInt = dynamicBool ? 0 | 1;

        // However, dynamic values cannot be used in certain situations.
        //let dynamicallySizedArray = [0, size = dynamicInt];
        //let staticallySizedArray = [0, size = 10];
        //ApplyNPiXRotations(dynamicInt, q1);
        //let staticInt = 3;
        //ApplyNPiXRotations(staticInt, q1);

        // Even though Quantinuum supports dynamic bools and ints, it does not
        // support dynamic doubles.
        //let dynamicDouble = r0 == One ? 1. | 0.;
        //let dynamicDouble = IntAsDouble(dynamicInt);
        //let dynamicRoot = Sqrt(dynamicDouble);
        //let staticRoot = Sqrt(4.0); // RELEVANT!

        // Even if we do some computations, we don't generate anything until qubits are used.
        // IMPROVEMENT: the classic QDK would generate QIR that was not supported by our
        //              hardware partners for most math functions in Q#'s standard library.
        let alpha = ArcCos(0.5);
        let beta = ArcSin(0.5);

        // If we apply gates to qubits we generate code.
        // Note that we only include the operations that are used.
        //Rx(alpha, q1);
        //Ry(beta, q1);

        // We evaluate anything that can be classically evaluated.
        // If expressions with classical conditions.
        //let i = 5;
        //if i > 0 {
        //    X(q1);
        //}
        //if i > 5 {
        //    Y(q1);
        //} else {
        //    Z(q1);
        //}

        // Loops with classical conditions.
        //for _ in 0..4 {
        //    T(q0);
        //}
        //for theta in [PI(), 2.0*PI(), 3.0*PI()] {
        //    Rx(theta, q1);
        //}

        // Also works when calling operations.
        //ApplyNPiXRotations(3, q1);

        // Loops with classical conditions and if expressions.
        //for idx in 0..3 {
        //    if idx % 2 == 0 {
        //        Rx(ArcSin(1.), q0);
        //        Rz(IntAsDouble(idx) * PI(), q1)
        //    } else {
        //        Ry(ArcCos(-1.), q1);
        //        Rz(IntAsDouble(idx) * PI(), q1)
        //    }
        //}

        // Branching based on measurement.
        //if r0 == One {
        //    X(q1);
        //}
        //let r1 = MResetZ(q1);
        //if r0 != r1 {
        //    let angle = PI() + PI() + PI()* Sin(PI()/2.0);
        //    Rxx(angle, q0, q1);
        //} else {
        //    Rxx(PI() + PI() + 2.0 * PI() * Sin(PI()/2.0), q1, q0);
        //}
        r0
    }

    operation ApplyNPiXRotations(n : Int, q : Qubit) : Unit {
        for _ in 0..n-1 {
            Rx(PI(), q);
        }
    }
}