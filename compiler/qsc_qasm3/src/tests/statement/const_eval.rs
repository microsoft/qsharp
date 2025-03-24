// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! The tests in this file need to check that const exprs are
//! evaluatable at lowering time. To do that we use them in
//! contexts where they need to be const-evaluated, like array
//! sizes or type widths.

use crate::tests::compile_qasm_to_qsharp;
use expect_test::expect;
use miette::Report;

#[test]
fn const_exprs_work_in_bitarray_size_position() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const int a = 1;
        const int b = 2 + a;
        const int c = a + 3;
        bit[b] r1;
        bit[c] r2;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 1;
        let b = 2 + a;
        let c = a + 3;
        mutable r1 = [Zero, Zero, Zero];
        mutable r2 = [Zero, Zero, Zero, Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn const_exprs_implicit_cast_work_in_bitarray_size_position() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const int a = 1;
        const float b = 2.0 + a;
        const float c = a + 3.0;
        bit[b] r1;
        bit[c] r2;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 1;
        let b = 2. + Microsoft.Quantum.Convert.IntAsDouble(a);
        let c = Microsoft.Quantum.Convert.IntAsDouble(a) + 3.;
        mutable r1 = [Zero, Zero, Zero];
        mutable r2 = [Zero, Zero, Zero, Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn non_const_exprs_fail_in_bitarray_size_position() {
    let source = r#"
        const int a = 1;
        int b = 2 + a;
        int c = a + 3;
        bit[b] r1;
        bit[c] r2;
    "#;

    let Err(errs) = compile_qasm_to_qsharp(source) else {
        panic!("should have generated an error");
    };
    let errs: Vec<_> = errs.iter().map(|e| format!("{e:?}")).collect();
    let errs_string = errs.join("\n");
    expect![[r#"
        Qsc.Qasm3.Compile.ExprMustBeConst

          x designator must be a const expression
           ,-[Test.qasm:5:13]
         4 |         int c = a + 3;
         5 |         bit[b] r1;
           :             ^
         6 |         bit[c] r2;
           `----

        Qsc.Qasm3.Compile.ExprMustBeConst

          x designator must be a const expression
           ,-[Test.qasm:6:13]
         5 |         bit[b] r1;
         6 |         bit[c] r2;
           :             ^
         7 |     
           `----
    "#]]
    .assert_eq(&errs_string);
}

#[test]
fn can_assign_const_expr_to_non_const_decl() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const int a = 1;
        const int b = 2;
        int c = a + b;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 1;
        let b = 2;
        mutable c = a + b;
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn ident_const() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const uint a = 1;
        bit[a] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 1;
        mutable r = [Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
#[ignore = "indexed ident is not yet supported"]
fn indexed_ident() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const uint[2] a = {1, 2};
        bit[a[1]] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 1;
        let b = 2;
        mutable c = a + b;
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

// UnaryOp Float

#[test]
fn unary_op_neg_float() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const float a = -1.0;
        const float b = -a;
        bit[b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = -1.;
        let b = -a;
        mutable r = [Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn unary_op_neg_int() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const int a = -1;
        const int b = -a;
        bit[b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = -1;
        let b = -a;
        mutable r = [Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
#[ignore = "casting float to angle is not yet supported"]
fn unary_op_neg_angle() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const angle a = -1.0;
        const bool b = a;
        bit[b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#""#]].assert_eq(&qsharp);
    Ok(())
}

#[test]
fn unary_op_negb_uint() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const uint[3] a = 5;
        const uint[3] b = ~a;
        bit[b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 5;
        let b = ~~~a;
        mutable r = [Zero, Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
#[ignore = "angles are not yet supported"]
fn unary_op_negb_angle() {
    let source = r#"
        const angle a = 1.0;
        const bool b = ~a;
        bit[b] r;
    "#;

    let Err(errs) = compile_qasm_to_qsharp(source) else {
        panic!("should have generated an error");
    };
    let errs: Vec<_> = errs.iter().map(|e| format!("{e:?}")).collect();
    let errs_string = errs.join("\n");
    expect![[r#""#]].assert_eq(&errs_string);
}

#[test]
fn unary_op_negb_bit() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const bit a = 0;
        const bit b = ~a;
        bit[b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = Zero;
        let b = ~~~a;
        mutable r = [Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn unary_op_negb_bitarray() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const bit[3] a = "101";
        const uint[3] b = ~a;
        bit[b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        function __ResultArrayAsIntBE__(results : Result[]) : Int {
            Microsoft.Quantum.Convert.ResultArrayAsInt(Microsoft.Quantum.Arrays.Reversed(results))
        }
        let a = [One, Zero, One];
        let b = __ResultArrayAsIntBE__(~~~a);
        mutable r = [Zero, Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

// BinaryOp

#[test]
fn lhs_ty_equals_rhs_ty_assumption_holds() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const int a = 1;
        const float b = 2.0;
        const uint c = a + b;
        bit[c] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 1;
        let b = 2.;
        let c = Microsoft.Quantum.Math.Truncate(Microsoft.Quantum.Convert.IntAsDouble(a) + b);
        mutable r = [Zero, Zero, Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

// BinaryOp: Bit Shifts

// Shl

#[test]
fn binary_op_shl_uint() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const uint a = 1;
        const uint b = a << 2;
        bit[b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 1;
        let b = a <<< 2;
        mutable r = [Zero, Zero, Zero, Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
#[ignore = "angles are not yet supported"]
fn binary_op_shl_angle() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const angle a = 1;
        const angle b = a << 2;
        const bool c = b;
        bit[c] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#""#]].assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_shl_bit() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const bit a = 1;
        const bit b = a << 2;
        bit[b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        function __ResultAsInt__(input : Result) : Int {
            if Microsoft.Quantum.Convert.ResultAsBool(input) {
                1
            } else {
                0
            }
        }
        let a = One;
        let b = if __ResultAsInt__(a) <<< 2 == 0 {
            One
        } else {
            Zero
        };
        mutable r = [];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_shl_bitarray() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const bit[3] a = "101";
        const bit[3] b = a << 2;
        bit[b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        function __BoolAsResult__(input : Bool) : Result {
            Microsoft.Quantum.Convert.BoolAsResult(input)
        }
        function __IntAsResultArrayBE__(number : Int, bits : Int) : Result[] {
            mutable runningValue = number;
            mutable result = [];
            for _ in 1..bits {
                set result += [__BoolAsResult__((runningValue &&& 1) != 0)];
                set runningValue >>>= 1;
            }
            Microsoft.Quantum.Arrays.Reversed(result)
        }
        function __ResultArrayAsIntBE__(results : Result[]) : Int {
            Microsoft.Quantum.Convert.ResultArrayAsInt(Microsoft.Quantum.Arrays.Reversed(results))
        }
        let a = [One, Zero, One];
        let b = __IntAsResultArrayBE__(__ResultArrayAsIntBE__(a) <<< 2, 3);
        mutable r = [Zero, Zero, Zero, Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_shl_creg_fails() {
    let source = r#"
        const creg a[3] = "101";
        const creg b[3] = a << 2;
        bit[b] r;
    "#;

    let Err(errs) = compile_qasm_to_qsharp(source) else {
        panic!("should have generated an error");
    };
    let errs: Vec<_> = errs.iter().map(|e| format!("{e:?}")).collect();
    let errs_string = errs.join("\n");
    expect![[r#"
        Qasm3.Parse.Rule

          x expected scalar or array type, found keyword `creg`
           ,-[Test.qasm:2:15]
         1 | 
         2 |         const creg a[3] = "101";
           :               ^^^^
         3 |         const creg b[3] = a << 2;
           `----

        Qasm3.Parse.Rule

          x expected scalar or array type, found keyword `creg`
           ,-[Test.qasm:3:15]
         2 |         const creg a[3] = "101";
         3 |         const creg b[3] = a << 2;
           :               ^^^^
         4 |         bit[b] r;
           `----

        Qsc.Qasm3.Compile.UndefinedSymbol

          x Undefined symbol: b.
           ,-[Test.qasm:4:13]
         3 |         const creg b[3] = a << 2;
         4 |         bit[b] r;
           :             ^
         5 |     
           `----

        Qsc.Qasm3.Compile.CannotCast

          x Cannot cast expression of type Err to type UInt(None, true)
           ,-[Test.qasm:4:13]
         3 |         const creg b[3] = a << 2;
         4 |         bit[b] r;
           :             ^
         5 |     
           `----

        Qsc.Qasm3.Compile.ExprMustBeConst

          x designator must be a const expression
           ,-[Test.qasm:4:13]
         3 |         const creg b[3] = a << 2;
         4 |         bit[b] r;
           :             ^
         5 |     
           `----
    "#]]
    .assert_eq(&errs_string);
}

// Shr

#[test]
fn binary_op_shr_uint() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const uint a = 5;
        const uint b = a >> 2;
        bit[b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 5;
        let b = a >>> 2;
        mutable r = [Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
#[ignore = "angles are not yet supported"]
fn binary_op_shr_angle() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const angle a = 1;
        const angle b = a >> 2;
        const bool c = b;
        bit[c] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#""#]].assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_shr_bit() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const bit a = 1;
        const bit b = a >> 2;
        bit[b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        function __ResultAsInt__(input : Result) : Int {
            if Microsoft.Quantum.Convert.ResultAsBool(input) {
                1
            } else {
                0
            }
        }
        let a = One;
        let b = if __ResultAsInt__(a) >>> 2 == 0 {
            One
        } else {
            Zero
        };
        mutable r = [];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_shr_bitarray() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const bit[4] a = "1011";
        const bit[4] b = a >> 2;
        bit[b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        function __BoolAsResult__(input : Bool) : Result {
            Microsoft.Quantum.Convert.BoolAsResult(input)
        }
        function __IntAsResultArrayBE__(number : Int, bits : Int) : Result[] {
            mutable runningValue = number;
            mutable result = [];
            for _ in 1..bits {
                set result += [__BoolAsResult__((runningValue &&& 1) != 0)];
                set runningValue >>>= 1;
            }
            Microsoft.Quantum.Arrays.Reversed(result)
        }
        function __ResultArrayAsIntBE__(results : Result[]) : Int {
            Microsoft.Quantum.Convert.ResultArrayAsInt(Microsoft.Quantum.Arrays.Reversed(results))
        }
        let a = [One, Zero, One, One];
        let b = __IntAsResultArrayBE__(__ResultArrayAsIntBE__(a) >>> 2, 4);
        mutable r = [Zero, Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_shr_creg_fails() {
    let source = r#"
        const creg a[4] = "1011";
        const creg b[4] = a >> 2;
        bit[b] r;
    "#;

    let Err(errs) = compile_qasm_to_qsharp(source) else {
        panic!("should have generated an error");
    };
    let errs: Vec<_> = errs.iter().map(|e| format!("{e:?}")).collect();
    let errs_string = errs.join("\n");
    expect![[r#"
        Qasm3.Parse.Rule

          x expected scalar or array type, found keyword `creg`
           ,-[Test.qasm:2:15]
         1 | 
         2 |         const creg a[4] = "1011";
           :               ^^^^
         3 |         const creg b[4] = a >> 2;
           `----

        Qasm3.Parse.Rule

          x expected scalar or array type, found keyword `creg`
           ,-[Test.qasm:3:15]
         2 |         const creg a[4] = "1011";
         3 |         const creg b[4] = a >> 2;
           :               ^^^^
         4 |         bit[b] r;
           `----

        Qsc.Qasm3.Compile.UndefinedSymbol

          x Undefined symbol: b.
           ,-[Test.qasm:4:13]
         3 |         const creg b[4] = a >> 2;
         4 |         bit[b] r;
           :             ^
         5 |     
           `----

        Qsc.Qasm3.Compile.CannotCast

          x Cannot cast expression of type Err to type UInt(None, true)
           ,-[Test.qasm:4:13]
         3 |         const creg b[4] = a >> 2;
         4 |         bit[b] r;
           :             ^
         5 |     
           `----

        Qsc.Qasm3.Compile.ExprMustBeConst

          x designator must be a const expression
           ,-[Test.qasm:4:13]
         3 |         const creg b[4] = a >> 2;
         4 |         bit[b] r;
           :             ^
         5 |     
           `----
    "#]]
    .assert_eq(&errs_string);
}

// BinaryOp: Bitwise

// AndB

#[test]
fn binary_op_andb_uint() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const uint a = 5;
        const uint b = a & 6;
        bit[b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 5;
        let b = a &&& 6;
        mutable r = [Zero, Zero, Zero, Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
#[ignore = "angles are not yet supported"]
fn binary_op_andb_angle() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const angle a = 1;
        const angle b = a & 2;
        const bool c = b;
        bit[c] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#""#]].assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_andb_bit() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const bit a = 1;
        const bit b = a & 0;
        bit[b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        function __ResultAsInt__(input : Result) : Int {
            if Microsoft.Quantum.Convert.ResultAsBool(input) {
                1
            } else {
                0
            }
        }
        let a = One;
        let b = if __ResultAsInt__(a) &&& 0 == 0 {
            One
        } else {
            Zero
        };
        mutable r = [];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_andb_bitarray() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const bit[4] a = "1011";
        const bit[4] b = a & "0110";
        bit[b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        function __BoolAsResult__(input : Bool) : Result {
            Microsoft.Quantum.Convert.BoolAsResult(input)
        }
        function __IntAsResultArrayBE__(number : Int, bits : Int) : Result[] {
            mutable runningValue = number;
            mutable result = [];
            for _ in 1..bits {
                set result += [__BoolAsResult__((runningValue &&& 1) != 0)];
                set runningValue >>>= 1;
            }
            Microsoft.Quantum.Arrays.Reversed(result)
        }
        function __ResultArrayAsIntBE__(results : Result[]) : Int {
            Microsoft.Quantum.Convert.ResultArrayAsInt(Microsoft.Quantum.Arrays.Reversed(results))
        }
        let a = [One, Zero, One, One];
        let b = __IntAsResultArrayBE__(__ResultArrayAsIntBE__(a) &&& __ResultArrayAsIntBE__([Zero, One, One, Zero]), 4);
        mutable r = [Zero, Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

// OrB

#[test]
fn binary_op_orb_uint() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const uint a = 5;
        const uint b = a | 6;
        bit[b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 5;
        let b = a ||| 6;
        mutable r = [Zero, Zero, Zero, Zero, Zero, Zero, Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
#[ignore = "angles are not yet supported"]
fn binary_op_orb_angle() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const angle a = 1.0;
        const angle b = a | 2.0;
        const bool c = b;
        bit[c] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#""#]].assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_orb_bit() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const bit a = 1;
        const bit b = a | 0;
        bit[b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        function __ResultAsInt__(input : Result) : Int {
            if Microsoft.Quantum.Convert.ResultAsBool(input) {
                1
            } else {
                0
            }
        }
        let a = One;
        let b = if __ResultAsInt__(a) ||| 0 == 0 {
            One
        } else {
            Zero
        };
        mutable r = [Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_orb_bitarray() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const bit[3] a = "001";
        const bit[3] b = a | "100";
        bit[b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        function __BoolAsResult__(input : Bool) : Result {
            Microsoft.Quantum.Convert.BoolAsResult(input)
        }
        function __IntAsResultArrayBE__(number : Int, bits : Int) : Result[] {
            mutable runningValue = number;
            mutable result = [];
            for _ in 1..bits {
                set result += [__BoolAsResult__((runningValue &&& 1) != 0)];
                set runningValue >>>= 1;
            }
            Microsoft.Quantum.Arrays.Reversed(result)
        }
        function __ResultArrayAsIntBE__(results : Result[]) : Int {
            Microsoft.Quantum.Convert.ResultArrayAsInt(Microsoft.Quantum.Arrays.Reversed(results))
        }
        let a = [Zero, Zero, One];
        let b = __IntAsResultArrayBE__(__ResultArrayAsIntBE__(a) ||| __ResultArrayAsIntBE__([One, Zero, Zero]), 3);
        mutable r = [Zero, Zero, Zero, Zero, Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

// XorB

#[test]
fn binary_op_xorb_uint() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const uint a = 5;
        const uint b = a ^ 6;
        bit[b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 5;
        let b = a ^^^ 6;
        mutable r = [Zero, Zero, Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
#[ignore = "angles are not yet supported"]
fn binary_op_xorb_angle() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const angle a = 1;
        const angle b = a ^ 2;
        const bool c = b;
        bit[c] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#""#]].assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_xorb_bit() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const bit a = 1;
        const bit b = a ^ 1;
        bit[b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        function __ResultAsInt__(input : Result) : Int {
            if Microsoft.Quantum.Convert.ResultAsBool(input) {
                1
            } else {
                0
            }
        }
        let a = One;
        let b = if __ResultAsInt__(a) ^^^ 1 == 0 {
            One
        } else {
            Zero
        };
        mutable r = [];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_xorb_bitarray() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const bit[4] a = "1011";
        const bit[4] b = a ^ "1110";
        bit[b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        function __BoolAsResult__(input : Bool) : Result {
            Microsoft.Quantum.Convert.BoolAsResult(input)
        }
        function __IntAsResultArrayBE__(number : Int, bits : Int) : Result[] {
            mutable runningValue = number;
            mutable result = [];
            for _ in 1..bits {
                set result += [__BoolAsResult__((runningValue &&& 1) != 0)];
                set runningValue >>>= 1;
            }
            Microsoft.Quantum.Arrays.Reversed(result)
        }
        function __ResultArrayAsIntBE__(results : Result[]) : Int {
            Microsoft.Quantum.Convert.ResultArrayAsInt(Microsoft.Quantum.Arrays.Reversed(results))
        }
        let a = [One, Zero, One, One];
        let b = __IntAsResultArrayBE__(__ResultArrayAsIntBE__(a) ^^^ __ResultArrayAsIntBE__([One, One, One, Zero]), 4);
        mutable r = [Zero, Zero, Zero, Zero, Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

// Binary Logical

#[test]
fn binary_op_andl_bool() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const bool f = false;
        const bool t = true;
        bit[f && f] r1;
        bit[f && t] r2;
        bit[t && f] r3;
        bit[t && t] r4;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let f = false;
        let t = true;
        mutable r1 = [];
        mutable r2 = [];
        mutable r3 = [];
        mutable r4 = [Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_orl_bool() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const bool f = false;
        const bool t = true;
        bit[f || f] r1;
        bit[f || t] r2;
        bit[t || f] r3;
        bit[t || t] r4;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let f = false;
        let t = true;
        mutable r1 = [];
        mutable r2 = [Zero];
        mutable r3 = [Zero];
        mutable r4 = [Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

// BinaryOp: Comparison

// Eq

#[test]
fn binary_op_comparison_int() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const int a = 2;
        bit[a == a] r1;
        bit[a != a] r2;
        bit[a > a] r3;
        bit[a >= a] r4;
        bit[a < a] r5;
        bit[a <= a] r6;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 2;
        mutable r1 = [Zero];
        mutable r2 = [];
        mutable r3 = [];
        mutable r4 = [Zero];
        mutable r5 = [];
        mutable r6 = [Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_comparison_uint() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const uint a = 2;
        bit[a == a] r1;
        bit[a != a] r2;
        bit[a > a] r3;
        bit[a >= a] r4;
        bit[a < a] r5;
        bit[a <= a] r6;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 2;
        mutable r1 = [Zero];
        mutable r2 = [];
        mutable r3 = [];
        mutable r4 = [Zero];
        mutable r5 = [];
        mutable r6 = [Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
#[ignore = "angles are not yet supported"]
fn binary_op_comparison_angle() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const angle a = 2.0;
        bit[a == a] r1;
        bit[a != a] r2;
        bit[a > a] r3;
        bit[a >= a] r4;
        bit[a < a] r5;
        bit[a <= a] r6;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#""#]].assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_comparison_bit() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const bit a = 1;
        bit[a == a] r1;
        bit[a != a] r2;
        bit[a > a] r3;
        bit[a >= a] r4;
        bit[a < a] r5;
        bit[a <= a] r6;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = One;
        mutable r1 = [Zero];
        mutable r2 = [];
        mutable r3 = [];
        mutable r4 = [Zero];
        mutable r5 = [];
        mutable r6 = [Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_comparison_bitarray() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const bit[2] a = "10";
        bit[a == a] r1;
        bit[a != a] r2;
        bit[a > a] r3;
        bit[a >= a] r4;
        bit[a < a] r5;
        bit[a <= a] r6;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = [One, Zero];
        mutable r1 = [Zero];
        mutable r2 = [];
        mutable r3 = [];
        mutable r4 = [Zero];
        mutable r5 = [];
        mutable r6 = [Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

// BinaryOp: Arithmetic

// Add

#[test]
fn binary_op_add_int() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const int a = 1;
        const int b = 2;
        bit[a + b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 1;
        let b = 2;
        mutable r = [Zero, Zero, Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_add_uint() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const uint a = 1;
        const uint b = 2;
        bit[a + b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 1;
        let b = 2;
        mutable r = [Zero, Zero, Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_add_float() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const float a = 1.0;
        const float b = 2.0;
        bit[a + b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 1.;
        let b = 2.;
        mutable r = [Zero, Zero, Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
#[ignore = "angles are not yet supported"]
fn binary_op_add_angle() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const angle a = 1.0;
        const angle b = 2.0;
        bit[a + b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#""#]].assert_eq(&qsharp);
    Ok(())
}

// Sub

#[test]
fn binary_op_sub_int() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const int a = 3;
        const int b = 2;
        bit[a - b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 3;
        let b = 2;
        mutable r = [Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_sub_uint() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const uint a = 3;
        const uint b = 2;
        bit[a - b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 3;
        let b = 2;
        mutable r = [Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_sub_float() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const float a = 3.0;
        const float b = 2.0;
        bit[a - b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 3.;
        let b = 2.;
        mutable r = [Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
#[ignore = "angles are not yet supported"]
fn binary_op_sub_angle() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const float a = 3.0;
        const float b = 2.0;
        bit[a - b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#""#]].assert_eq(&qsharp);
    Ok(())
}

// Mul

#[test]
fn binary_op_mul_int() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const int a = 3;
        const int b = 2;
        bit[a * b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 3;
        let b = 2;
        mutable r = [Zero, Zero, Zero, Zero, Zero, Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_mul_uint() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const uint a = 3;
        const uint b = 2;
        bit[a * b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 3;
        let b = 2;
        mutable r = [Zero, Zero, Zero, Zero, Zero, Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_mul_float() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const float a = 3.0;
        const float b = 2.0;
        bit[a * b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 3.;
        let b = 2.;
        mutable r = [Zero, Zero, Zero, Zero, Zero, Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
#[ignore = "angles are not yet supported"]
fn binary_op_mul_angle() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const float a = 3.0;
        const uint b = 2;
        bit[a * b] r1;
        bit[b * a] r2;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#""#]].assert_eq(&qsharp);
    Ok(())
}

// Div

#[test]
fn binary_op_div_int() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const int a = 6;
        const int b = 2;
        bit[a / b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 6;
        let b = 2;
        mutable r = [Zero, Zero, Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_div_uint() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const uint a = 6;
        const uint b = 2;
        bit[a / b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 6;
        let b = 2;
        mutable r = [Zero, Zero, Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_div_float() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const float a = 6.0;
        const float b = 2.0;
        bit[a / b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 6.;
        let b = 2.;
        mutable r = [Zero, Zero, Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
#[ignore = "angles are not yet supported"]
fn binary_op_div_angle() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const angle a = 6.0;
        const uint b = 2;
        bit[a / b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#""#]].assert_eq(&qsharp);
    Ok(())
}

// Mod

#[test]
fn binary_op_mod_int() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const int a = 8;
        bit[a % 3] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 8;
        mutable r = [Zero, Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_mod_uint() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const uint a = 8;
        bit[a % 3] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 8;
        mutable r = [Zero, Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

// Pow

#[test]
fn binary_op_pow_int() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const int a = 2;
        const int b = 3;
        bit[a ** b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 2;
        let b = 3;
        mutable r = [Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_pow_uint() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const uint a = 2;
        const uint b = 3;
        bit[a ** b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 2;
        let b = 3;
        mutable r = [Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_pow_float() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const float a = 2.0;
        const float b = 3.0;
        bit[a ** b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 2.;
        let b = 3.;
        mutable r = [Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

// Cast

#[test]
fn cast_to_bool() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const int a = 0;
        const uint b = 1;
        const float c = 2.0;
        // const angle d = 2.0;
        const bit e = 1;

        const bool s1 = a;
        const bool s2 = b;
        const bool s3 = c;
        // const bool s4 = d;
        const bool s5 = e;

        bit[s1] r1;
        bit[s2] r2;
        bit[s3] r3;
        // bit[s4] r4;
        bit[s5] r5;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        function __ResultAsBool__(input : Result) : Bool {
            Microsoft.Quantum.Convert.ResultAsBool(input)
        }
        let a = 0;
        let b = 1;
        let c = 2.;
        let e = One;
        let s1 = if a == 0 {
            false
        } else {
            true
        };
        let s2 = if b == 0 {
            false
        } else {
            true
        };
        let s3 = if Microsoft.Quantum.Math.Truncate(c) == 0 {
            false
        } else {
            true
        };
        let s5 = __ResultAsBool__(e);
        mutable r1 = [];
        mutable r2 = [Zero];
        mutable r3 = [Zero];
        mutable r5 = [Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn cast_to_int() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const bool a = true;
        const uint b = 2;
        const float c = 3.0;
        const bit d = 0;

        const int s1 = a;
        const int s2 = b;
        const int s3 = c;
        const int s4 = d;

        bit[s1] r1;
        bit[s2] r2;
        bit[s3] r3;
        bit[s4] r4;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        function __BoolAsInt__(value : Bool) : Int {
            if value {
                1
            } else {
                0
            }
        }
        function __ResultAsInt__(input : Result) : Int {
            if Microsoft.Quantum.Convert.ResultAsBool(input) {
                1
            } else {
                0
            }
        }
        let a = true;
        let b = 2;
        let c = 3.;
        let d = Zero;
        let s1 = __BoolAsInt__(a);
        let s2 = b;
        let s3 = Microsoft.Quantum.Math.Truncate(c);
        let s4 = __ResultAsInt__(d);
        mutable r1 = [Zero];
        mutable r2 = [Zero, Zero];
        mutable r3 = [Zero, Zero, Zero];
        mutable r4 = [];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn cast_to_uint() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const bool a = true;
        const uint b = 2;
        const float c = 3.0;
        const bit d = 0;

        const uint s1 = a;
        const uint s2 = b;
        const uint s3 = c;
        const uint s4 = d;

        bit[s1] r1;
        bit[s2] r2;
        bit[s3] r3;
        bit[s4] r4;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        function __BoolAsInt__(value : Bool) : Int {
            if value {
                1
            } else {
                0
            }
        }
        function __ResultAsInt__(input : Result) : Int {
            if Microsoft.Quantum.Convert.ResultAsBool(input) {
                1
            } else {
                0
            }
        }
        let a = true;
        let b = 2;
        let c = 3.;
        let d = Zero;
        let s1 = __BoolAsInt__(a);
        let s2 = b;
        let s3 = Microsoft.Quantum.Math.Truncate(c);
        let s4 = __ResultAsInt__(d);
        mutable r1 = [Zero];
        mutable r2 = [Zero, Zero];
        mutable r3 = [Zero, Zero, Zero];
        mutable r4 = [];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn cast_to_float() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const bool a = true;
        const int b = 2;
        const uint c = 3;

        const float s1 = a;
        const float s2 = b;
        const float s3 = c;

        bit[s1] r1;
        bit[s2] r2;
        bit[s3] r3;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        function __BoolAsDouble__(value : Bool) : Double {
            if value {
                1.
            } else {
                0.
            }
        }
        let a = true;
        let b = 2;
        let c = 3;
        let s1 = __BoolAsDouble__(a);
        let s2 = Microsoft.Quantum.Convert.IntAsDouble(b);
        let s3 = Microsoft.Quantum.Convert.IntAsDouble(c);
        mutable r1 = [Zero];
        mutable r2 = [Zero, Zero];
        mutable r3 = [Zero, Zero, Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
#[ignore = "angles are not yet supported"]
fn cast_to_angle() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const float a = 2.0;
        const bit b = 1;

        const angle s1 = a;
        const angle s2 = b;

        bit[s1] r1;
        bit[s2] r2;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#""#]].assert_eq(&qsharp);
    Ok(())
}

#[test]
fn cast_to_bit() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const bool a = false;
        const int b = 1;
        const uint c = 2;
        // const angle d = 3.0;

        const bit s1 = a;
        const bit s2 = b;
        const bit s3 = c;
        // const bit s4 = d;

        bit[s1] r1;
        bit[s2] r2;
        bit[s3] r3;
        // bit[s4] r4;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        function __BoolAsResult__(input : Bool) : Result {
            Microsoft.Quantum.Convert.BoolAsResult(input)
        }
        let a = false;
        let b = 1;
        let c = 2;
        let s1 = __BoolAsResult__(a);
        let s2 = if b == 0 {
            One
        } else {
            Zero
        };
        let s3 = if c == 0 {
            One
        } else {
            Zero
        };
        mutable r1 = [];
        mutable r2 = [Zero];
        mutable r3 = [Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}
