// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

namespace Microsoft.Quantum.Arithmetic {
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Arrays;
    open Microsoft.Quantum.Math;

    // Computes ys += xs + carryIn using a ripple carry architecture
    @Config(Unrestricted)
    internal operation IncWithCarryIn(carryIn : Qubit, xs : Qubit[], ys : Qubit[])
    : Unit is Adj + Ctl {
        // We assume that it has already been checked that xs and ys are of
        // equal size and non-empty in RippleCarryIncByLE
        if Length(xs) == 1 {
            if Length(ys) == 1 {
                within {
                    CNOT(carryIn, xs[0]);
                } apply {
                    CNOT(xs[0], ys[0]);
                }
            } elif Length(ys) == 2 {
                FullAdderForInc(carryIn, xs[0], ys[0], ys[1]);
            }
        } else {
            let (x0, xrest) = HeadAndRest(xs);
            let (y0, yrest) = HeadAndRest(ys);

            use carryOut = Qubit();
            CarryForInc(carryIn, x0, y0, carryOut);
            IncWithCarryIn(carryOut, xrest, yrest);
            UncarryForInc(carryIn, x0, y0, carryOut);
        }
    }

    /// # Summary
    /// Implements Half-adder. Adds qubit x to qubit y and sets carryOut appropriately
    @Config(Unrestricted)
    internal operation HalfAdderForInc(x : Qubit, y : Qubit, carryOut : Qubit)
    : Unit is Adj + Ctl {
        body (...) {
            CCNOT(x, y, carryOut);
            CNOT(x, y);
        }
        adjoint auto;

        controlled (ctls, ...) {
            Fact(Length(ctls) == 1, "HalfAdderForInc should be controlled by exactly one control qubit.");

            let ctl = ctls[0];
            use helper = Qubit();

            within {
                ApplyAndAssuming0Target(x, y, helper);
            } apply {
                ApplyAndAssuming0Target(ctl, helper, carryOut);
            }
            CCNOT(ctl, x, y);
        }
        controlled adjoint auto;
    }

    /// # Summary
    /// Implements Full-adder. Adds qubit carryIn and x to qubit y and sets carryOut appropriately.
    @Config(Unrestricted)
    internal operation FullAdderForInc(carryIn : Qubit, x : Qubit, y : Qubit, carryOut : Qubit)
    : Unit is Adj + Ctl {
        body (...) {
            // TODO: cannot use `Carry` operation here
            CNOT(carryIn, x);
            CNOT(carryIn, y);
            CCNOT(x, y, carryOut);
            CNOT(carryIn, carryOut);
            CNOT(carryIn, x);
            CNOT(x, y);
        }
        adjoint auto;

        controlled (ctls, ...) {
            Fact(Length(ctls) == 1, "FullAdderForInc should be controlled by exactly one control qubit.");

            let ctl = ctls[0];
            use helper = Qubit();

            CarryForInc(carryIn, x, y, helper);
            CCNOT(ctl, helper, carryOut);
            Controlled UncarryForInc(ctls, (carryIn, x, y, helper));
        }
        controlled adjoint auto;
    }

    // Computes carryOut := carryIn + x + y
    @Config(Unrestricted)
    internal operation FullAdder(carryIn : Qubit, x : Qubit, y : Qubit, carryOut : Qubit)
    : Unit is Adj {
        CNOT(x, y);
        CNOT(x, carryIn);
        ApplyAndAssuming0Target(y, carryIn, carryOut);
        CNOT(x, y);
        CNOT(x, carryOut);
        CNOT(y, carryIn);
    }

    /// # Summary
    /// Computes carry bit for a full adder.
    @Config(Unrestricted)
    internal operation CarryForInc(carryIn : Qubit, x : Qubit, y : Qubit, carryOut : Qubit)
    : Unit is Adj + Ctl {
        body (...) {
            CNOT(carryIn, x);
            CNOT(carryIn, y);
            ApplyAndAssuming0Target(x, y, carryOut);
            CNOT(carryIn, carryOut);
        }
        adjoint auto;
        controlled (ctls, ...) {
            // This CarryForInc is intended to be used only in an in-place
            // ripple-carry implementation. Only such particular use case allows
            // for this simple implementation where controlled verions
            // is the same as uncontrolled body.
            CarryForInc(carryIn, x, y, carryOut);
        }
        controlled adjoint auto;
    }

    /// # Summary
    /// Uncomputes carry bit for a full adder.
    @Config(Unrestricted)
    internal operation UncarryForInc(carryIn : Qubit, x : Qubit, y : Qubit, carryOut : Qubit)
    : Unit is Adj + Ctl {
        body (...) {
            CNOT(carryIn, carryOut);
            Adjoint ApplyAndAssuming0Target(x, y, carryOut);
            CNOT(carryIn, x);
            CNOT(x, y);
        }
        adjoint auto;
        controlled (ctls, ...) {
            Fact(Length(ctls) == 1, "UncarryForInc should be controlled by exactly one control qubit.");

            let ctl = ctls[0];

            CNOT(carryIn, carryOut);
            Adjoint ApplyAndAssuming0Target(x, y, carryOut);
            CCNOT(ctl, x, y); // Controlled X(ctls + [x], y);
            CNOT(carryIn, x);
            CNOT(carryIn, y);
        }
        controlled adjoint auto;
    }

    /// # Summary
    /// Applies AND gate between `control1` and `control2` and stores the result
    /// in `target` assuming `target` is in |0> state. This allows for an
    /// otimized adjoint implementation.
    @Config(Unrestricted)
    internal operation ApplyAndAssuming0Target(control1 : Qubit, control2 : Qubit, target: Qubit)
    : Unit is Adj {
        body (...) {
            if not CheckZero(target) {
                fail "ApplyAndAssuming0Target expects `target` to be in |0> state.";
            }
            CCNOT(control1, control2, target);
        }

        adjoint (...) {
            H(target);
            if M(target) == One {
                Reset(target);
                CZ(control1, control2);
            }
        }
    }

}
