// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::semantic::tests::check_stmt_kinds as check;
use expect_test::expect;

#[test]
fn array_concatenation_has_the_right_type() {
    let source = "
    def f(readonly array[int, #dim = 1] a, readonly array[int, #dim = 1] b, mutable array[int, #dim = 1] c) {
        c = a ++ b;
    }
    ";

    check(
        source,
        &expect![[r#"
            DefStmt [5-136]:
                symbol_id: 8
                has_qubit_params: false
                parameters:
                    9
                    10
                    11
                return_type_span: [0-0]
                body: Block [109-136]:
                    Stmt [119-130]:
                        annotations: <empty>
                        kind: AssignStmt [119-130]:
                            lhs: Expr [119-120]:
                                ty: mutable array[int, #dim = 1]
                                kind: SymbolId(11)
                            rhs: Expr [123-129]:
                                ty: readonly array[int, #dim = 1]
                                kind: ConcatExpr [123-129]:
                                    operands:
                                        Expr [123-124]:
                                            ty: readonly array[int, #dim = 1]
                                            kind: SymbolId(9)
                                        Expr [128-129]:
                                            ty: readonly array[int, #dim = 1]
                                            kind: SymbolId(10)
        "#]],
    );
}

#[test]
fn array_can_be_concatenated_with_itself() {
    let source = "
    def f(readonly array[int, #dim = 1] a, mutable array[int, #dim = 1] c) {
        c = a ++ a;
    }
    ";

    check(
        source,
        &expect![[r#"
            DefStmt [5-103]:
                symbol_id: 8
                has_qubit_params: false
                parameters:
                    9
                    10
                return_type_span: [0-0]
                body: Block [76-103]:
                    Stmt [86-97]:
                        annotations: <empty>
                        kind: AssignStmt [86-97]:
                            lhs: Expr [86-87]:
                                ty: mutable array[int, #dim = 1]
                                kind: SymbolId(10)
                            rhs: Expr [90-96]:
                                ty: readonly array[int, #dim = 1]
                                kind: ConcatExpr [90-96]:
                                    operands:
                                        Expr [90-91]:
                                            ty: readonly array[int, #dim = 1]
                                            kind: SymbolId(9)
                                        Expr [95-96]:
                                            ty: readonly array[int, #dim = 1]
                                            kind: SymbolId(9)
        "#]],
    );
}

#[test]
fn array_concatenation_with_different_widths_errors() {
    let source = "
    def f(readonly array[int[8], #dim = 1] a, readonly array[int[16], #dim = 1] b, mutable array[int[8], #dim = 1] c) {
        c = a ++ b;
    }
    ";

    check(
        source,
        &expect![[r#"
            Program:
                version: <none>
                pragmas: <empty>
                statements:
                    Stmt [5-146]:
                        annotations: <empty>
                        kind: DefStmt [5-146]:
                            symbol_id: 8
                            has_qubit_params: false
                            parameters:
                                9
                                10
                                11
                            return_type_span: [0-0]
                            body: Block [119-146]:
                                Stmt [129-140]:
                                    annotations: <empty>
                                    kind: AssignStmt [129-140]:
                                        lhs: Expr [129-130]:
                                            ty: mutable array[int[8], #dim = 1]
                                            kind: SymbolId(11)
                                        rhs: Expr [133-139]:
                                            ty: unknown
                                            kind: ConcatExpr [133-139]:
                                                operands:
                                                    Expr [133-134]:
                                                        ty: readonly array[int[8], #dim = 1]
                                                        kind: SymbolId(9)
                                                    Expr [138-139]:
                                                        ty: readonly array[int[16], #dim = 1]
                                                        kind: SymbolId(10)

            [Qasm.Lowerer.InconsistentTypesInArrayConcatenation

              x inconsistent types in array concatenation expression: readonly
              | array[int[8], #dim = 1], readonly array[int[16], #dim = 1]
               ,-[test:3:13]
             2 |     def f(readonly array[int[8], #dim = 1] a, readonly array[int[16], #dim = 1] b, mutable array[int[8], #dim = 1] c) {
             3 |         c = a ++ b;
               :             ^^^^^^
             4 |     }
               `----
            ]"#]],
    );
}

#[test]
fn array_concatenation_with_different_types_errors() {
    let source = "
    def f(readonly array[int[8], #dim = 1] a, readonly array[uint[8], #dim = 1] b, mutable array[int[8], #dim = 1] c) {
        c = a ++ b;
    }
    ";

    check(
        source,
        &expect![[r#"
            Program:
                version: <none>
                pragmas: <empty>
                statements:
                    Stmt [5-146]:
                        annotations: <empty>
                        kind: DefStmt [5-146]:
                            symbol_id: 8
                            has_qubit_params: false
                            parameters:
                                9
                                10
                                11
                            return_type_span: [0-0]
                            body: Block [119-146]:
                                Stmt [129-140]:
                                    annotations: <empty>
                                    kind: AssignStmt [129-140]:
                                        lhs: Expr [129-130]:
                                            ty: mutable array[int[8], #dim = 1]
                                            kind: SymbolId(11)
                                        rhs: Expr [133-139]:
                                            ty: unknown
                                            kind: ConcatExpr [133-139]:
                                                operands:
                                                    Expr [133-134]:
                                                        ty: readonly array[int[8], #dim = 1]
                                                        kind: SymbolId(9)
                                                    Expr [138-139]:
                                                        ty: readonly array[uint[8], #dim = 1]
                                                        kind: SymbolId(10)

            [Qasm.Lowerer.InconsistentTypesInArrayConcatenation

              x inconsistent types in array concatenation expression: readonly
              | array[int[8], #dim = 1], readonly array[uint[8], #dim = 1]
               ,-[test:3:13]
             2 |     def f(readonly array[int[8], #dim = 1] a, readonly array[uint[8], #dim = 1] b, mutable array[int[8], #dim = 1] c) {
             3 |         c = a ++ b;
               :             ^^^^^^
             4 |     }
               `----
            ]"#]],
    );
}

#[test]
fn multidimensional_array_concatenation_has_the_right_type() {
    let source = "
    def f(readonly array[int, #dim = 2] a, readonly array[int, #dim = 2] b, mutable array[int, #dim = 2] c) {
        c = a ++ b;
    }
    ";

    check(
        source,
        &expect![[r#"
            DefStmt [5-136]:
                symbol_id: 8
                has_qubit_params: false
                parameters:
                    9
                    10
                    11
                return_type_span: [0-0]
                body: Block [109-136]:
                    Stmt [119-130]:
                        annotations: <empty>
                        kind: AssignStmt [119-130]:
                            lhs: Expr [119-120]:
                                ty: mutable array[int, #dim = 2]
                                kind: SymbolId(11)
                            rhs: Expr [123-129]:
                                ty: readonly array[int, #dim = 2]
                                kind: ConcatExpr [123-129]:
                                    operands:
                                        Expr [123-124]:
                                            ty: readonly array[int, #dim = 2]
                                            kind: SymbolId(9)
                                        Expr [128-129]:
                                            ty: readonly array[int, #dim = 2]
                                            kind: SymbolId(10)
        "#]],
    );
}

#[test]
fn multidimensional_array_can_be_concatenated_with_itself() {
    let source = "
    def f(readonly array[int, #dim = 2] a, mutable array[int, #dim = 2] c) {
        c = a ++ a;
    }
    ";

    check(
        source,
        &expect![[r#"
            DefStmt [5-103]:
                symbol_id: 8
                has_qubit_params: false
                parameters:
                    9
                    10
                return_type_span: [0-0]
                body: Block [76-103]:
                    Stmt [86-97]:
                        annotations: <empty>
                        kind: AssignStmt [86-97]:
                            lhs: Expr [86-87]:
                                ty: mutable array[int, #dim = 2]
                                kind: SymbolId(10)
                            rhs: Expr [90-96]:
                                ty: readonly array[int, #dim = 2]
                                kind: ConcatExpr [90-96]:
                                    operands:
                                        Expr [90-91]:
                                            ty: readonly array[int, #dim = 2]
                                            kind: SymbolId(9)
                                        Expr [95-96]:
                                            ty: readonly array[int, #dim = 2]
                                            kind: SymbolId(9)
        "#]],
    );
}

#[test]
fn multidimensional_array_concatenation_with_different_widths_errors() {
    let source = "
    def f(readonly array[int[8], #dim = 2] a, readonly array[int[16], #dim = 2] b, mutable array[int[8], #dim = 2] c) {
        c = a ++ b;
    }
    ";

    check(
        source,
        &expect![[r#"
            Program:
                version: <none>
                pragmas: <empty>
                statements:
                    Stmt [5-146]:
                        annotations: <empty>
                        kind: DefStmt [5-146]:
                            symbol_id: 8
                            has_qubit_params: false
                            parameters:
                                9
                                10
                                11
                            return_type_span: [0-0]
                            body: Block [119-146]:
                                Stmt [129-140]:
                                    annotations: <empty>
                                    kind: AssignStmt [129-140]:
                                        lhs: Expr [129-130]:
                                            ty: mutable array[int[8], #dim = 2]
                                            kind: SymbolId(11)
                                        rhs: Expr [133-139]:
                                            ty: unknown
                                            kind: ConcatExpr [133-139]:
                                                operands:
                                                    Expr [133-134]:
                                                        ty: readonly array[int[8], #dim = 2]
                                                        kind: SymbolId(9)
                                                    Expr [138-139]:
                                                        ty: readonly array[int[16], #dim = 2]
                                                        kind: SymbolId(10)

            [Qasm.Lowerer.InconsistentTypesInArrayConcatenation

              x inconsistent types in array concatenation expression: readonly
              | array[int[8], #dim = 2], readonly array[int[16], #dim = 2]
               ,-[test:3:13]
             2 |     def f(readonly array[int[8], #dim = 2] a, readonly array[int[16], #dim = 2] b, mutable array[int[8], #dim = 2] c) {
             3 |         c = a ++ b;
               :             ^^^^^^
             4 |     }
               `----
            ]"#]],
    );
}

#[test]
fn multidimensional_array_concatenation_with_different_types_errors() {
    let source = "
    def f(readonly array[int[8], #dim = 2] a, readonly array[uint[8], #dim = 2] b, mutable array[int[8], #dim = 2] c) {
        c = a ++ b;
    }
    ";

    check(
        source,
        &expect![[r#"
            Program:
                version: <none>
                pragmas: <empty>
                statements:
                    Stmt [5-146]:
                        annotations: <empty>
                        kind: DefStmt [5-146]:
                            symbol_id: 8
                            has_qubit_params: false
                            parameters:
                                9
                                10
                                11
                            return_type_span: [0-0]
                            body: Block [119-146]:
                                Stmt [129-140]:
                                    annotations: <empty>
                                    kind: AssignStmt [129-140]:
                                        lhs: Expr [129-130]:
                                            ty: mutable array[int[8], #dim = 2]
                                            kind: SymbolId(11)
                                        rhs: Expr [133-139]:
                                            ty: unknown
                                            kind: ConcatExpr [133-139]:
                                                operands:
                                                    Expr [133-134]:
                                                        ty: readonly array[int[8], #dim = 2]
                                                        kind: SymbolId(9)
                                                    Expr [138-139]:
                                                        ty: readonly array[uint[8], #dim = 2]
                                                        kind: SymbolId(10)

            [Qasm.Lowerer.InconsistentTypesInArrayConcatenation

              x inconsistent types in array concatenation expression: readonly
              | array[int[8], #dim = 2], readonly array[uint[8], #dim = 2]
               ,-[test:3:13]
             2 |     def f(readonly array[int[8], #dim = 2] a, readonly array[uint[8], #dim = 2] b, mutable array[int[8], #dim = 2] c) {
             3 |         c = a ++ b;
               :             ^^^^^^
             4 |     }
               `----
            ]"#]],
    );
}
