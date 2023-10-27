// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

namespace Microsoft.Quantum.Arithmetic {
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Arrays;
    open Microsoft.Quantum.Math;

    // Computes ys += xs + carryIn using a ripple carry architecture
    @Config(Full)
    internal operation AddWithCarryIn(carryIn : Qubit, xs : Qubit[], ys : Qubit[]) : Unit is Adj + Ctl {
        // We assume that it has already been checked that xs and ys are of
        // equal size and non-empty in Add
        if Length(xs) == 1 {
            if Length(ys) == 1 {
                within {
                    CNOT(carryIn, xs[0]);
                } apply {
                    CNOT(xs[0], ys[0]);
                }
            } elif Length(ys) == 2 {
                FullAdderInc(carryIn, xs[0], ys[0], ys[1]);
            }
        } else {
            let (x0, xrest) = HeadAndRest(xs);
            let (y0, yrest) = HeadAndRest(ys);

            use carryOut = Qubit();
            Carry(carryIn, x0, y0, carryOut);
            AddWithCarryIn(carryOut, xrest, yrest);
            Uncarry(carryIn, x0, y0, carryOut);
        }
    }

    /// # Summary
    /// Implements Half-adder. Adds qubit x to qubit y and sets carryOut appropriately
    /// assuming carryOut is in |0> state.
    @Config(Full)
    internal operation HalfAdderInc(x : Qubit, y : Qubit, carryOut : Qubit) : Unit is Adj + Ctl {
        body (...) {
            ApplyAndWith0Target(x, y, carryOut);
            CNOT(x, y);
        }
        adjoint auto;

        controlled (ctls, ...) {
            Fact(Length(ctls) == 1, "HalfAdder should be controlled by exactly one control qubit.");

            let ctl = ctls[0];
            use helper = Qubit();

            within {
                ApplyAndWith0Target(x, y, helper);
            } apply {
                ApplyAndWith0Target(ctl, helper, carryOut);
            }
            CCNOT(ctl, x, y);
        }
        controlled adjoint auto;
    }

    /// # Summary
    /// Implements Full-adder. Adds qubit carryIn and x to qubit y and sets carryOut appropriately.
    @Config(Full)
    internal operation FullAdderInc(carryIn : Qubit, x : Qubit, y : Qubit, carryOut : Qubit) : Unit is Adj + Ctl {
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
            Fact(Length(ctls) == 1, "FullAdder should be controlled by exactly one control qubit.");

            let ctl = ctls[0];
            use helper = Qubit();

            Carry(carryIn, x, y, helper);
            CCNOT(ctl, helper, carryOut);
            Controlled Uncarry(ctls, (carryIn, x, y, helper));
        }
        controlled adjoint auto;
    }

    // Computes carryOut := carryIn + x + y
    @Config(Full)
    internal operation FullAdder(carryIn : Qubit, x : Qubit, y : Qubit, carryOut : Qubit) : Unit is Adj {
        CNOT(x, y);
        CNOT(x, carryIn);
        ApplyAndWith0Target(y, carryIn, carryOut);
        CNOT(x, y);
        CNOT(x, carryOut);
        CNOT(y, carryIn);
    }

    /// # Summary
    /// Compute carry bit for a full adder.
    @Config(Full)
    internal operation Carry(carryIn : Qubit, x : Qubit, y : Qubit, carryOut : Qubit) : Unit is Adj + Ctl {
        body (...) {
            CNOT(carryIn, x);
            CNOT(carryIn, y);
            ApplyAndWith0Target(x, y, carryOut);
            CNOT(carryIn, carryOut);
        }
        adjoint auto;
        controlled (ctls, ...) {
            // TODO: Is it not controlled actually?
            Carry(carryIn, x, y, carryOut);
        }
        controlled adjoint auto;
    }

    /// # Summary
    /// Uncompute carry bit for a full adder.
    @Config(Full)
    internal operation Uncarry(carryIn : Qubit, x : Qubit, y : Qubit, carryOut : Qubit) : Unit is Adj + Ctl {
        body (...) {
            CNOT(carryIn, carryOut);
            Adjoint ApplyAndWith0Target(x, y, carryOut);
            CNOT(carryIn, x);
            CNOT(x, y);
        }
        adjoint auto;
        controlled (ctls, ...) {
            Fact(Length(ctls) == 1, "Uncarry should be controlled by exactly one control qubit.");

            let ctl = ctls[0];

            CNOT(carryIn, carryOut);
            Adjoint ApplyAndWith0Target(x, y, carryOut);
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
    @Config(Full)
    internal operation ApplyAndWith0Target(control1 : Qubit, control2 : Qubit, target: Qubit) : Unit is Adj {
        body (...) {
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
