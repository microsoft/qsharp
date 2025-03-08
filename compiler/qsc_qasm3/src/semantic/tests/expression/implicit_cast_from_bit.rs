// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::expect;

use crate::semantic::tests::check_classical_decls;

use super::check_stmt_kinds;

#[test]
fn to_bool_and_back_implicitly() {
    let input = r#"
        input bit a;
        bool _bool0;
        bool _bool1;
        _bool0 = true;
        _bool1 = a;
        _bool0 = _bool1;
        _bool0 = _bool1;
        a = _bool1;
    "#;

    check_stmt_kinds(
        input,
        &expect![[r#"
            Program:
                version: <none>
                statements:
                    Stmt [30-42]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [30-42]:
                            symbol_id: 6
                            ty_span: [30-34]
                            init_expr: Expr [0-0]:
                                ty: Bool(true)
                                kind: Lit: Bool(false)
                    Stmt [51-63]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [51-63]:
                            symbol_id: 7
                            ty_span: [51-55]
                            init_expr: Expr [0-0]:
                                ty: Bool(true)
                                kind: Lit: Bool(false)

            [Qsc.Qasm3.Compile.Unimplemented

              x this statement is not yet handled during OpenQASM 3 import: io decl stmt
               ,-[test:2:9]
             1 |
             2 |         input bit a;
               :         ^^^^^^^^^^^^
             3 |         bool _bool0;
               `----
            , Qsc.Qasm3.Compile.Unimplemented

              x this statement is not yet handled during OpenQASM 3 import: assign expr
               ,-[test:5:9]
             4 |         bool _bool1;
             5 |         _bool0 = true;
               :         ^^^^^^^^^^^^^
             6 |         _bool1 = a;
               `----
            , Qsc.Qasm3.Compile.Unimplemented

              x this statement is not yet handled during OpenQASM 3 import: assign expr
               ,-[test:6:9]
             5 |         _bool0 = true;
             6 |         _bool1 = a;
               :         ^^^^^^^^^^
             7 |         _bool0 = _bool1;
               `----
            , Qsc.Qasm3.Compile.Unimplemented

              x this statement is not yet handled during OpenQASM 3 import: assign expr
               ,-[test:7:9]
             6 |         _bool1 = a;
             7 |         _bool0 = _bool1;
               :         ^^^^^^^^^^^^^^^
             8 |         _bool0 = _bool1;
               `----
            , Qsc.Qasm3.Compile.Unimplemented

              x this statement is not yet handled during OpenQASM 3 import: assign expr
               ,-[test:8:9]
             7 |         _bool0 = _bool1;
             8 |         _bool0 = _bool1;
               :         ^^^^^^^^^^^^^^^
             9 |         a = _bool1;
               `----
            , Qsc.Qasm3.Compile.Unimplemented

              x this statement is not yet handled during OpenQASM 3 import: assign expr
                ,-[test:9:9]
              8 |         _bool0 = _bool1;
              9 |         a = _bool1;
                :         ^^^^^^^^^^
             10 |
                `----
            ]"#]],
    );
}

#[test]
fn to_bool_implicitly() {
    let input = r#"
         bit x = 1;
         bool y = x;
    "#;

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [10-20]:
                symbol_id: 6
                ty_span: [10-13]
                init_expr: Expr [18-19]:
                    ty: Bit(true)
                    kind: Lit: Int(1)
            [6] Symbol [14-15]:
                name: x
                type: Bit(false)
                qsharp_type: Result
                io_kind: Default
            ClassicalDeclarationStmt [30-41]:
                symbol_id: 7
                ty_span: [30-34]
                init_expr: Expr [0-0]:
                    ty: Bool(false)
                    kind: Cast [0-0]:
                        type: Bool(false)
                        expr: Expr [39-40]:
                            ty: Bit(false)
                            kind: 6
            [7] Symbol [35-36]:
                name: y
                type: Bool(false)
                qsharp_type: bool
                io_kind: Default
        "#]],
    );
}

// #[test]
// fn to_bool_implicitly() -> miette::Result<(), Vec<Report>> {
//     let source = "
//         bit x = 1;
//         bool y = x;
//     ";

//     let qsharp = compile_qasm_to_qsharp(source)?;
//     expect![
//         r#"
//         function __ResultAsBool__(input : Result) : Bool {
//             Microsoft.Quantum.Convert.ResultAsBool(input)
//         }
//         mutable x = One;
//         mutable y = __ResultAsBool__(x);
//     "#
//     ]
//     .assert_eq(&qsharp);
//     Ok(())
// }

// #[test]
// fn to_implicit_int_implicitly() -> miette::Result<(), Vec<Report>> {
//     let source = "
//         bit x = 1;
//         int y = x;
//     ";

//     let qsharp = compile_qasm_to_qsharp(source)?;
//     expect![
//         r#"
//         function __ResultAsInt__(input : Result) : Int {
//             if Microsoft.Quantum.Convert.ResultAsBool(input) {
//                 1
//             } else {
//                 0
//             }
//         }
//         mutable x = One;
//         mutable y = __ResultAsInt__(x);
//     "#
//     ]
//     .assert_eq(&qsharp);
//     Ok(())
// }

// #[test]
// fn to_explicit_int_implicitly() -> miette::Result<(), Vec<Report>> {
//     let source = "
//         bit x = 1;
//         int[32] y = x;
//     ";

//     let qsharp = compile_qasm_to_qsharp(source)?;
//     expect![
//         r#"
//         function __ResultAsInt__(input : Result) : Int {
//             if Microsoft.Quantum.Convert.ResultAsBool(input) {
//                 1
//             } else {
//                 0
//             }
//         }
//         mutable x = One;
//         mutable y = __ResultAsInt__(x);
//     "#
//     ]
//     .assert_eq(&qsharp);
//     Ok(())
// }

// #[test]
// fn to_implicit_uint_implicitly() -> miette::Result<(), Vec<Report>> {
//     let source = "
//         bit x = 1;
//         uint y = x;
//     ";

//     let qsharp = compile_qasm_to_qsharp(source)?;
//     expect![
//         r#"
//         function __ResultAsInt__(input : Result) : Int {
//             if Microsoft.Quantum.Convert.ResultAsBool(input) {
//                 1
//             } else {
//                 0
//             }
//         }
//         mutable x = One;
//         mutable y = __ResultAsInt__(x);
//     "#
//     ]
//     .assert_eq(&qsharp);
//     Ok(())
// }

// #[test]
// fn to_explicit_uint_implicitly() -> miette::Result<(), Vec<Report>> {
//     let source = "
//         bit x = 1;
//         uint[32] y = x;
//     ";

//     let qsharp = compile_qasm_to_qsharp(source)?;
//     expect![
//         r#"
//         function __ResultAsInt__(input : Result) : Int {
//             if Microsoft.Quantum.Convert.ResultAsBool(input) {
//                 1
//             } else {
//                 0
//             }
//         }
//         mutable x = One;
//         mutable y = __ResultAsInt__(x);
//     "#
//     ]
//     .assert_eq(&qsharp);
//     Ok(())
// }

// #[test]
// fn to_explicit_bigint_implicitly() -> miette::Result<(), Vec<Report>> {
//     let source = "
//         bit x = 1;
//         int[65] y = x;
//     ";

//     let qsharp = compile_qasm_to_qsharp(source)?;
//     expect![
//         r#"
//         function __ResultAsBigInt__(input : Result) : BigInt {
//             if Microsoft.Quantum.Convert.ResultAsBool(input) {
//                 1L
//             } else {
//                 0L
//             }
//         }
//         mutable x = One;
//         mutable y = __ResultAsBigInt__(x);
//     "#
//     ]
//     .assert_eq(&qsharp);
//     Ok(())
// }

// #[test]
// fn to_implicit_float_implicitly_fails() {
//     let source = "
//         bit x = 1;
//         float y = x;
//     ";

//     let Err(error) = compile_qasm_to_qsharp(source) else {
//         panic!("Expected error")
//     };

//     expect![r#"Cannot cast expression of type Bit(False) to type Float(None, False)"#]
//         .assert_eq(&error[0].to_string());
// }
