// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::expect;

use crate::semantic::tests::check_classical_decls;

#[test]
fn to_int_decl_implicitly() {
    let input = r#"
        bit[5] reg;
        int b = reg;
    "#;

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-20]:
                symbol_id: 8
                ty_span: [9-15]
                init_expr: Expr [0-0]:
                    ty: BitArray(5, true)
                    kind: Lit: Bitstring("00000")
            [8] Symbol [16-19]:
                name: reg
                type: BitArray(5, false)
                qsharp_type: Result[]
                io_kind: Default
            ClassicalDeclarationStmt [29-41]:
                symbol_id: 9
                ty_span: [29-32]
                init_expr: Expr [37-40]:
                    ty: Int(None, false)
                    kind: Cast [0-0]:
                        ty: Int(None, false)
                        expr: Expr [37-40]:
                            ty: BitArray(5, false)
                            kind: SymbolId(8)
            [9] Symbol [33-34]:
                name: b
                type: Int(None, false)
                qsharp_type: Int
                io_kind: Default
        "#]],
    );
}

#[test]
fn to_int_assignment_implicitly() {
    let input = r#"
        bit[5] reg;
        int a;
        a = reg;
    "#;

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-20]:
                symbol_id: 8
                ty_span: [9-15]
                init_expr: Expr [0-0]:
                    ty: BitArray(5, true)
                    kind: Lit: Bitstring("00000")
            [8] Symbol [16-19]:
                name: reg
                type: BitArray(5, false)
                qsharp_type: Result[]
                io_kind: Default
            ClassicalDeclarationStmt [29-35]:
                symbol_id: 9
                ty_span: [29-32]
                init_expr: Expr [0-0]:
                    ty: Int(None, true)
                    kind: Lit: Int(0)
            [9] Symbol [33-34]:
                name: a
                type: Int(None, false)
                qsharp_type: Int
                io_kind: Default
            AssignStmt [44-52]:
                symbol_id: 9
                lhs_span: [44-45]
                rhs: Expr [48-51]:
                    ty: Int(None, false)
                    kind: Cast [0-0]:
                        ty: Int(None, false)
                        expr: Expr [48-51]:
                            ty: BitArray(5, false)
                            kind: SymbolId(8)
            [9] Symbol [33-34]:
                name: a
                type: Int(None, false)
                qsharp_type: Int
                io_kind: Default
        "#]],
    );
}

#[test]
fn to_int_with_equal_width_in_assignment_implicitly() {
    let input = r#"
        bit[5] reg;
        int[5] a;
        a = reg;
    "#;

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-20]:
                symbol_id: 8
                ty_span: [9-15]
                init_expr: Expr [0-0]:
                    ty: BitArray(5, true)
                    kind: Lit: Bitstring("00000")
            [8] Symbol [16-19]:
                name: reg
                type: BitArray(5, false)
                qsharp_type: Result[]
                io_kind: Default
            ClassicalDeclarationStmt [29-38]:
                symbol_id: 9
                ty_span: [29-35]
                init_expr: Expr [0-0]:
                    ty: Int(Some(5), true)
                    kind: Lit: Int(0)
            [9] Symbol [36-37]:
                name: a
                type: Int(Some(5), false)
                qsharp_type: Int
                io_kind: Default
            AssignStmt [47-55]:
                symbol_id: 9
                lhs_span: [47-48]
                rhs: Expr [51-54]:
                    ty: Int(Some(5), false)
                    kind: Cast [0-0]:
                        ty: Int(Some(5), false)
                        expr: Expr [51-54]:
                            ty: BitArray(5, false)
                            kind: SymbolId(8)
            [9] Symbol [36-37]:
                name: a
                type: Int(Some(5), false)
                qsharp_type: Int
                io_kind: Default
        "#]],
    );
}

#[test]
fn to_int_with_equal_width_in_decl_implicitly() {
    let input = r#"
        bit[5] reg;
        int[5] a = reg;
    "#;

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-20]:
                symbol_id: 8
                ty_span: [9-15]
                init_expr: Expr [0-0]:
                    ty: BitArray(5, true)
                    kind: Lit: Bitstring("00000")
            [8] Symbol [16-19]:
                name: reg
                type: BitArray(5, false)
                qsharp_type: Result[]
                io_kind: Default
            ClassicalDeclarationStmt [29-44]:
                symbol_id: 9
                ty_span: [29-35]
                init_expr: Expr [40-43]:
                    ty: Int(Some(5), false)
                    kind: Cast [0-0]:
                        ty: Int(Some(5), false)
                        expr: Expr [40-43]:
                            ty: BitArray(5, false)
                            kind: SymbolId(8)
            [9] Symbol [36-37]:
                name: a
                type: Int(Some(5), false)
                qsharp_type: Int
                io_kind: Default
        "#]],
    );
}

#[test]
fn to_int_with_higher_width_implicitly_fails() {
    let input = "
        int[6] a;
        bit[5] reg;
        a = reg;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            Program:
                version: <none>
                statements:
                    Stmt [9-18]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [9-18]:
                            symbol_id: 8
                            ty_span: [9-15]
                            init_expr: Expr [0-0]:
                                ty: Int(Some(6), true)
                                kind: Lit: Int(0)
                    Stmt [27-38]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [27-38]:
                            symbol_id: 9
                            ty_span: [27-33]
                            init_expr: Expr [0-0]:
                                ty: BitArray(5, true)
                                kind: Lit: Bitstring("00000")
                    Stmt [47-55]:
                        annotations: <empty>
                        kind: AssignStmt [47-55]:
                            symbol_id: 8
                            lhs_span: [47-48]
                            rhs: Expr [51-54]:
                                ty: BitArray(5, false)
                                kind: SymbolId(9)

            [Qasm.Lowerer.CannotCast

              x cannot cast expression of type BitArray(5, false) to type Int(Some(6),
              | false)
               ,-[test:4:13]
             3 |         bit[5] reg;
             4 |         a = reg;
               :             ^^^
             5 |     
               `----
            ]"#]],
    );
}

#[test]
fn to_int_with_higher_width_decl_implicitly_fails() {
    let input = "
        bit[5] reg;
        int[6] a = reg;
    ";
    check_classical_decls(
        input,
        &expect![[r#"
            Program:
                version: <none>
                statements:
                    Stmt [9-20]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [9-20]:
                            symbol_id: 8
                            ty_span: [9-15]
                            init_expr: Expr [0-0]:
                                ty: BitArray(5, true)
                                kind: Lit: Bitstring("00000")
                    Stmt [29-44]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [29-44]:
                            symbol_id: 9
                            ty_span: [29-35]
                            init_expr: Expr [40-43]:
                                ty: BitArray(5, false)
                                kind: SymbolId(8)

            [Qasm.Lowerer.CannotCast

              x cannot cast expression of type BitArray(5, false) to type Int(Some(6),
              | false)
               ,-[test:3:20]
             2 |         bit[5] reg;
             3 |         int[6] a = reg;
               :                    ^^^
             4 |     
               `----
            ]"#]],
    );
}

#[test]
fn to_int_with_lower_width_implicitly_fails() {
    let input = "
        input int[4] a;
        bit[5] reg;
        a = reg;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            Program:
                version: <none>
                statements:
                    Stmt [9-24]:
                        annotations: <empty>
                        kind: InputDeclaration [9-24]:
                            symbol_id: 8
                    Stmt [33-44]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [33-44]:
                            symbol_id: 9
                            ty_span: [33-39]
                            init_expr: Expr [0-0]:
                                ty: BitArray(5, true)
                                kind: Lit: Bitstring("00000")
                    Stmt [53-61]:
                        annotations: <empty>
                        kind: AssignStmt [53-61]:
                            symbol_id: 8
                            lhs_span: [53-54]
                            rhs: Expr [57-60]:
                                ty: BitArray(5, false)
                                kind: SymbolId(9)

            [Qasm.Lowerer.CannotCast

              x cannot cast expression of type BitArray(5, false) to type Int(Some(4),
              | false)
               ,-[test:4:13]
             3 |         bit[5] reg;
             4 |         a = reg;
               :             ^^^
             5 |     
               `----
            ]"#]],
    );
}

#[test]
fn to_int_with_lower_width_decl_implicitly_fails() {
    let input = "
        bit[5] reg;
        int[4] a = reg;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            Program:
                version: <none>
                statements:
                    Stmt [9-20]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [9-20]:
                            symbol_id: 8
                            ty_span: [9-15]
                            init_expr: Expr [0-0]:
                                ty: BitArray(5, true)
                                kind: Lit: Bitstring("00000")
                    Stmt [29-44]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [29-44]:
                            symbol_id: 9
                            ty_span: [29-35]
                            init_expr: Expr [40-43]:
                                ty: BitArray(5, false)
                                kind: SymbolId(8)

            [Qasm.Lowerer.CannotCast

              x cannot cast expression of type BitArray(5, false) to type Int(Some(4),
              | false)
               ,-[test:3:20]
             2 |         bit[5] reg;
             3 |         int[4] a = reg;
               :                    ^^^
             4 |     
               `----
            ]"#]],
    );
}
