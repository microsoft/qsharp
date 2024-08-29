// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

import Facts.AssertFormatsAreIdenticalFxP, Facts.AssertAllZeroFxP;
import Types.FixedPoint;
import Init.PrepareFxP;
import Operations.MultiplyFxP, Operations.SquareFxP, Operations.AddConstantFxP;

/// # Summary
/// Evaluates a polynomial in a fixed-point representation.
///
/// # Input
/// ## coefficients
/// Coefficients of the polynomial as a double array, i.e., the array
/// $[a_0, a_1, ..., a_d]$ for the polynomial
/// $P(x) = a_0 + a_1 x + \cdots + a_d x^d$.
/// ## fpx
/// Input fixed-point number for which to evaluate the polynomial.
/// ## result
/// Output fixed-point number which will hold $P(x)$. Must be in state
/// $\ket{0}$ initially.
operation EvaluatePolynomialFxP(coefficients : Double[], fpx : FixedPoint, result : FixedPoint) : Unit is Adj {
    body (...) {
        Controlled EvaluatePolynomialFxP([], (coefficients, fpx, result));
    }
    controlled (controls, ...) {
        AssertFormatsAreIdenticalFxP([fpx, result]);
        let degree = Length(coefficients) - 1;
        let p = fpx::IntegerBits;
        let n = Length(fpx::Register);
        if degree == 0 {
            Controlled PrepareFxP(controls, (coefficients[0], result));
        } elif degree > 0 {
            // initialize ancillary register to a_d
            use qubits = Qubit[n * degree];
            within {
                let firstIterate = FixedPoint(p, qubits[(degree - 1) * n..degree * n - 1]);
                PrepareFxP(coefficients[degree], firstIterate);
                for d in degree..(-1)..2 {
                    let currentIterate = FixedPoint(p, qubits[(d - 1) * n..d * n - 1]);
                    let nextIterate = FixedPoint(p, qubits[(d - 2) * n..(d - 1) * n - 1]);
                    // multiply by x and then add current coefficient
                    MultiplyFxP(currentIterate, fpx, nextIterate);
                    AddConstantFxP(coefficients[d-1], nextIterate);
                }
            } apply {
                let finalIterate = FixedPoint(p, qubits[0..n-1]);
                // final multiplication into the result register
                Controlled MultiplyFxP(controls, (finalIterate, fpx, result));
                // add a_0 to complete polynomial evaluation and
                Controlled AddConstantFxP(controls, (coefficients[0], result));
            }
        }
    }
}

/// # Summary
/// Evaluates an even polynomial in a fixed-point representation.
///
/// # Input
/// ## coefficients
/// Coefficients of the even polynomial as a double array, i.e., the array
/// $[a_0, a_1, ..., a_k]$ for the even polynomial
/// $P(x) = a_0 + a_1 x^2 + \cdots + a_k x^{2k}$.
/// ## fpx
/// Input fixed-point number for which to evaluate the polynomial.
/// ## result
/// Output fixed-point number which will hold $P(x)$. Must be in state
/// $\ket{0}$ initially.
operation EvaluateEvenPolynomialFxP(coefficients : Double[], fpx : FixedPoint, result : FixedPoint) : Unit is Adj {
    body (...) {
        Controlled EvaluateEvenPolynomialFxP([], (coefficients, fpx, result));
    }
    controlled (controls, ...) {
        AssertFormatsAreIdenticalFxP([fpx, result]);
        let halfDegree = Length(coefficients) - 1;
        let n = Length(fpx::Register);

        if halfDegree == 0 {
            Controlled PrepareFxP(controls, (coefficients[0], result));
        } elif halfDegree > 0 {
            // initialize auxiliary register to a_d
            use xsSquared = Qubit[n];
            let fpxSquared = FixedPoint(fpx::IntegerBits, xsSquared);
            within {
                SquareFxP(fpx, fpxSquared);
            } apply {
                Controlled EvaluatePolynomialFxP(controls, (coefficients, fpxSquared, result));
            }
        }
    }
}

/// # Summary
/// Evaluates an odd polynomial in a fixed-point representation.
///
/// # Input
/// ## coefficients
/// Coefficients of the odd polynomial as a double array, i.e., the array
/// $[a_0, a_1, ..., a_k]$ for the odd polynomial
/// $P(x) = a_0 x + a_1 x^3 + \cdots + a_k x^{2k+1}$.
/// ## fpx
/// Input fixed-point number for which to evaluate the polynomial.
/// ## result
/// Output fixed-point number which will hold P(x). Must be in state
/// $\ket{0}$ initially.
operation EvaluateOddPolynomialFxP(coefficients : Double[], fpx : FixedPoint, result : FixedPoint) : Unit is Adj {
    body (...) {
        Controlled EvaluateOddPolynomialFxP([], (coefficients, fpx, result));
    }
    controlled (controls, ...) {
        AssertFormatsAreIdenticalFxP([fpx, result]);
        let halfDegree = Length(coefficients) - 1;
        let n = Length(fpx::Register);
        if halfDegree >= 0 {
            use tmpResult = Qubit[n];
            let tmpResultFp = FixedPoint(fpx::IntegerBits, tmpResult);
            within {
                EvaluateEvenPolynomialFxP(coefficients, fpx, tmpResultFp);
            } apply {
                Controlled MultiplyFxP(controls, (fpx, tmpResultFp, result));
            }
        }
    }
}

export
    EvaluatePolynomialFxP,
    EvaluateEvenPolynomialFxP,
    EvaluateOddPolynomialFxP;