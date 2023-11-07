// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

namespace Microsoft.Quantum.Arithmetic {
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Arrays;
    open Microsoft.Quantum.Math;
    open Microsoft.Quantum.Convert;


    // Computes ys += xs + carryIn using a ripple carry architecture
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
    /// in `target` assuming `target` is in |0> state.
    ///
    /// # Description
    /// Inverts `target` if and only if both controls are 1, but assumes that
    /// `target` is in state 0. The operation has T-count 4, T-depth 2 and
    /// requires no helper qubit, and may therefore be preferable to a CCNOT
    /// operation, if `target` is known to be 0.
    /// The adjoint of this operation is measurement based and requires no T
    /// gates (but requires target to support branching on measurements).
    /// Although the Toffoli gate (CCNOT) will perform faster in in simulations,
    /// this version has lower T gate requirements.
    /// # References
    /// - Cody Jones: "Novel constructions for the fault-tolerant Toffoli gate",
    ///   Phys. Rev. A 87, 022328, 2013
    ///   [arXiv:1212.5069](https://arxiv.org/abs/1212.5069)
    ///   doi:10.1103/PhysRevA.87.022328
    @Config(Full)
    internal operation ApplyAndAssuming0Target(control1 : Qubit, control2 : Qubit, target: Qubit)
    : Unit is Adj { // NOTE: Eventually this operation will be public and intrinsic.
        body (...) {
            if not CheckZero(target) {
                fail "ApplyAndAssuming0Target expects `target` to be in |0> state.";
            }
            H(target);
            T(target);
            CNOT(control1, target);
            CNOT(control2, target);
            within {
                CNOT(target, control1);
                CNOT(target, control2);
            }
            apply {
                Adjoint T(control1);
                Adjoint T(control2);
                T(target);
            }
            H(target);
            S(target);            
        }
        adjoint (...) {
            H(target);
            if M(target) == One {
                Reset(target);
                CZ(control1, control2);
            }
        }
    }

    /// # Summary
    /// Applies AND gate between `control1` and `control2` and stores the result
    /// in `target` assuming `target` is in |0> state.
    ///
    /// # Description
    /// Inverts `target` if and only if both controls are 1, but assumes that
    /// `target` is in state 0. The operation has T-count 4, T-depth 2 and
    /// requires no helper qubit, and may therefore be preferable to a CCNOT
    /// operation, if `target` is known to be 0.
    /// This version is suitable for Base profile.
    /// Although the Toffoli gate (CCNOT) will perform faster in in simulations,
    /// this version has lower T gate requirements.
    /// # References
    /// - Cody Jones: "Novel constructions for the fault-tolerant Toffoli gate",
    ///   Phys. Rev. A 87, 022328, 2013
    ///   [arXiv:1212.5069](https://arxiv.org/abs/1212.5069)
    ///   doi:10.1103/PhysRevA.87.022328
    @Config(Base)
    internal operation ApplyAndAssuming0Target(control1 : Qubit, control2 : Qubit, target: Qubit)
    : Unit is Adj {
        H(target);
        T(target);
        CNOT(control1, target);
        CNOT(control2, target);
        within {
            CNOT(target, control1);
            CNOT(target, control2);
        }
        apply {
            Adjoint T(control1);
            Adjoint T(control2);
            T(target);
        }
        H(target);
        S(target);
    }

    /// # Summary
    /// Computes carries for the look-ahead adder
    internal operation ComputeCarries(ps : Qubit[], gs : Qubit[]) : Unit is Adj {
        let n = Length(gs);
        Fact(Length(ps)+1 == n, "Register gs must be one qubit longer than register gs.");

        let T = Floor(Lg(IntAsDouble(n)));
        use qs = Qubit[n - HammingWeightI(n) - T];

        let registerPartition = MappedOverRange(t -> Floor(IntAsDouble(n) / IntAsDouble(2^t)) - 1, 1..T - 1);
        let pWorkspace = [ps] + Partitioned(registerPartition, qs);

        within {
            PRounds(pWorkspace);
        } apply {
            // U_G
            GRounds(pWorkspace, gs);

            // U_C
            CRounds(pWorkspace, gs);
        }
    }

    /// # Summary
    /// Computes all p[i, j] values in workspace for the look-ahead adder.
    ///
    /// The register array `pWorkspace` has T entries, where T = ⌊log₂ n⌋.
    ///
    /// The first entry `pWorkspace[0]` is initialized with `P_0` which is
    /// computed before `ComputeCarries` is called.  The other registers are
    /// 0-initialized and will be computed in successive rounds t = 1, ..., T - 1.
    ///
    /// In each round t we compute
    ///
    /// p[i, j] = p[2ᵗ × m, 2ᵗ × (m + 1)] = p[i, k] ∧ p[k, j]
    ///
    /// in `pWorkspace[t][m - 1]` and use that for k = 2ᵗ × m + 2ᵗ⁻¹, p[i, k] and p[k, j]
    /// have already been computed in round t - 1 in `pWorkspace[t - 1][2 * m - 1]` and
    /// `pWorkspace[t - 1][2 * m]`, respectively.
    internal operation PRounds(pWorkspace : Qubit[][]) : Unit is Adj {
        for ws in Windows(2, pWorkspace) {
            // note that we are using Rest, since pWorkspace[t - 1][0] is never
            // accessed in round t.
            let (current, next) = (Rest(ws[0]), ws[1]);

            for (m, target) in Enumerated(next) {
                ApplyAndAssuming0Target(current[2 * m], current[2 * m + 1], target);
            }
        }
    }

    /// # Summary
    /// Computes g[i ∧ (i + 1), i + 1] into gs[i] for the look-ahead adder.
    ///
    /// The register gs has n entries initialized to gs[i] = g[i, i + 1].
    ///
    /// After successive rounds t = 1, ..., T, the register is updated to
    /// gs[i] = g[i ∧ (i + 1), i + 1], from which we can compute the carries
    /// in the C-rounds.
    internal operation GRounds(pWorkspace : Qubit[][], gs : Qubit[]) : Unit is Adj {
        let T = Length(pWorkspace);
        let n = Length(gs);

        for t in 1..T {
            let length = Floor(IntAsDouble(n) / IntAsDouble(2^t)) - 1;
            let ps = pWorkspace[t - 1][0..2...];

            for m in 0..length {
                CCNOT(gs[2^t * m + 2^(t - 1) - 1], ps[m], gs[2^t * m + 2^t - 1]);
            }
        }
    }

    /// # Summary
    /// Computes carries into gs for the look-ahead adder.
    internal operation CRounds(pWorkspace : Qubit[][], gs : Qubit[]) : Unit is Adj {
        let n = Length(gs);

        let start = Floor(Lg(IntAsDouble(2 * n) / 3.0));
        for t in start..-1..1 {
            let length = Floor(IntAsDouble(n - 2^(t - 1)) / IntAsDouble(2^t));
            let ps = pWorkspace[t - 1][1..2...];

            for m in 1..length {
                CCNOT(gs[2^t * m - 1], ps[m - 1], gs[2^t * m + 2^(t - 1) - 1]);
            }
        }
    }

    internal operation PhaseGradient (qs : Qubit[]) : Unit is Adj + Ctl {
        for (i, q) in Enumerated(qs) {
            R1Frac(1, i, q);
        }
    }

}
