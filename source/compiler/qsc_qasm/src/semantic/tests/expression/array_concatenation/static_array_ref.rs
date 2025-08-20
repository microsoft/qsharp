// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::semantic::tests::check_stmt_kinds as check;
use expect_test::expect;

#[test]
fn array_concatenation_has_the_right_type() {
    let source = "
    def f(readonly array[int, 3] a, readonly array[int, 4] b, mutable array[int, 7] c) {
        c = a ++ b;
    }
    ";

    check(
        source,
        &expect![[r#"
            DefStmt [5-115]:
                symbol_id: 8
                has_qubit_params: false
                parameters:
                    9
                    10
                    11
                return_type_span: [0-0]
                body: Block [88-115]:
                    Stmt [98-109]:
                        annotations: <empty>
                        kind: AssignStmt [98-109]:
                            lhs: Expr [98-99]:
                                ty: mutable array[int, 7]
                                kind: SymbolId(11)
                            rhs: Expr [102-108]:
                                ty: readonly array[int, 7]
                                kind: ConcatExpr [102-108]:
                                    operands:
                                        Expr [102-103]:
                                            ty: readonly array[int, 3]
                                            kind: SymbolId(9)
                                        Expr [107-108]:
                                            ty: readonly array[int, 4]
                                            kind: SymbolId(10)
        "#]],
    );
}

#[test]
fn array_can_be_concatenated_with_itself() {
    let source = "
    def f(readonly array[int, 3] a, mutable array[int, 6] c) {
        c = a ++ a;
    }
    ";

    check(
        source,
        &expect![[r#"
            DefStmt [5-89]:
                symbol_id: 8
                has_qubit_params: false
                parameters:
                    9
                    10
                return_type_span: [0-0]
                body: Block [62-89]:
                    Stmt [72-83]:
                        annotations: <empty>
                        kind: AssignStmt [72-83]:
                            lhs: Expr [72-73]:
                                ty: mutable array[int, 6]
                                kind: SymbolId(10)
                            rhs: Expr [76-82]:
                                ty: readonly array[int, 6]
                                kind: ConcatExpr [76-82]:
                                    operands:
                                        Expr [76-77]:
                                            ty: readonly array[int, 3]
                                            kind: SymbolId(9)
                                        Expr [81-82]:
                                            ty: readonly array[int, 3]
                                            kind: SymbolId(9)
        "#]],
    );
}

#[test]
fn array_concatenation_with_different_widths_errors() {
    let source = "
    def f(readonly array[int[8], 3] a, readonly array[int[16], 4] b, mutable array[int[8], 7] c) {
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
                    Stmt [5-125]:
                        annotations: <empty>
                        kind: DefStmt [5-125]:
                            symbol_id: 8
                            has_qubit_params: false
                            parameters:
                                9
                                10
                                11
                            return_type_span: [0-0]
                            body: Block [98-125]:
                                Stmt [108-119]:
                                    annotations: <empty>
                                    kind: AssignStmt [108-119]:
                                        lhs: Expr [108-109]:
                                            ty: mutable array[int[8], 7]
                                            kind: SymbolId(11)
                                        rhs: Expr [112-118]:
                                            ty: unknown
                                            kind: ConcatExpr [112-118]:
                                                operands:
                                                    Expr [112-113]:
                                                        ty: readonly array[int[8], 3]
                                                        kind: SymbolId(9)
                                                    Expr [117-118]:
                                                        ty: readonly array[int[16], 4]
                                                        kind: SymbolId(10)

            [Qasm.Lowerer.InconsistentTypesInArrayConcatenation

              x inconsistent types in array concatenation expression: readonly
              | array[int[8], 3], readonly array[int[16], 4]
               ,-[test:3:13]
             2 |     def f(readonly array[int[8], 3] a, readonly array[int[16], 4] b, mutable array[int[8], 7] c) {
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
    def f(readonly array[int[8], 3] a, readonly array[uint[8], 4] b, mutable array[int[8], 7] c) {
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
                    Stmt [5-125]:
                        annotations: <empty>
                        kind: DefStmt [5-125]:
                            symbol_id: 8
                            has_qubit_params: false
                            parameters:
                                9
                                10
                                11
                            return_type_span: [0-0]
                            body: Block [98-125]:
                                Stmt [108-119]:
                                    annotations: <empty>
                                    kind: AssignStmt [108-119]:
                                        lhs: Expr [108-109]:
                                            ty: mutable array[int[8], 7]
                                            kind: SymbolId(11)
                                        rhs: Expr [112-118]:
                                            ty: unknown
                                            kind: ConcatExpr [112-118]:
                                                operands:
                                                    Expr [112-113]:
                                                        ty: readonly array[int[8], 3]
                                                        kind: SymbolId(9)
                                                    Expr [117-118]:
                                                        ty: readonly array[uint[8], 4]
                                                        kind: SymbolId(10)

            [Qasm.Lowerer.InconsistentTypesInArrayConcatenation

              x inconsistent types in array concatenation expression: readonly
              | array[int[8], 3], readonly array[uint[8], 4]
               ,-[test:3:13]
             2 |     def f(readonly array[int[8], 3] a, readonly array[uint[8], 4] b, mutable array[int[8], 7] c) {
             3 |         c = a ++ b;
               :             ^^^^^^
             4 |     }
               `----
            ]"#]],
    );
}

#[allow(clippy::too_many_lines)]
#[test]
fn multidimensional_array_concatenation_has_the_right_type() {
    let source = "
    def f(readonly array[int, 4, 2] a, readonly array[int, 5, 2] b, mutable array[int, 9, 2] c) {
        c = a ++ b;
    }
    ";

    check(
        source,
        &expect![[r#"
            DefStmt [5-124]:
                symbol_id: 8
                has_qubit_params: false
                parameters:
                    9
                    10
                    11
                return_type_span: [0-0]
                body: Block [97-124]:
                    Stmt [107-118]:
                        annotations: <empty>
                        kind: AssignStmt [107-118]:
                            lhs: Expr [107-108]:
                                ty: mutable array[int, 9, 2]
                                kind: SymbolId(11)
                            rhs: Expr [111-117]:
                                ty: readonly array[int, 9, 2]
                                kind: ConcatExpr [111-117]:
                                    operands:
                                        Expr [111-112]:
                                            ty: readonly array[int, 4, 2]
                                            kind: SymbolId(9)
                                        Expr [116-117]:
                                            ty: readonly array[int, 5, 2]
                                            kind: SymbolId(10)
        "#]],
    );
}

#[test]
fn multidimensional_array_can_be_concatenated_with_itself() {
    let source = "
    def f(readonly array[int, 4, 2] a, mutable array[int, 8, 2] c) {
        c = a ++ a;
    }
    ";

    check(
        source,
        &expect![[r#"
            DefStmt [5-95]:
                symbol_id: 8
                has_qubit_params: false
                parameters:
                    9
                    10
                return_type_span: [0-0]
                body: Block [68-95]:
                    Stmt [78-89]:
                        annotations: <empty>
                        kind: AssignStmt [78-89]:
                            lhs: Expr [78-79]:
                                ty: mutable array[int, 8, 2]
                                kind: SymbolId(10)
                            rhs: Expr [82-88]:
                                ty: readonly array[int, 8, 2]
                                kind: ConcatExpr [82-88]:
                                    operands:
                                        Expr [82-83]:
                                            ty: readonly array[int, 4, 2]
                                            kind: SymbolId(9)
                                        Expr [87-88]:
                                            ty: readonly array[int, 4, 2]
                                            kind: SymbolId(9)
        "#]],
    );
}

#[allow(clippy::too_many_lines)]
#[test]
fn multidimensional_array_concatenation_with_different_widths_errors() {
    let source = "
    def f(readonly array[int[8], 4, 2] a, readonly array[int[16], 5, 2] b, mutable array[int[8], 9, 2] c) {
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
                    Stmt [5-134]:
                        annotations: <empty>
                        kind: DefStmt [5-134]:
                            symbol_id: 8
                            has_qubit_params: false
                            parameters:
                                9
                                10
                                11
                            return_type_span: [0-0]
                            body: Block [107-134]:
                                Stmt [117-128]:
                                    annotations: <empty>
                                    kind: AssignStmt [117-128]:
                                        lhs: Expr [117-118]:
                                            ty: mutable array[int[8], 9, 2]
                                            kind: SymbolId(11)
                                        rhs: Expr [121-127]:
                                            ty: unknown
                                            kind: ConcatExpr [121-127]:
                                                operands:
                                                    Expr [121-122]:
                                                        ty: readonly array[int[8], 4, 2]
                                                        kind: SymbolId(9)
                                                    Expr [126-127]:
                                                        ty: readonly array[int[16], 5, 2]
                                                        kind: SymbolId(10)

            [Qasm.Lowerer.InconsistentTypesInArrayConcatenation

              x inconsistent types in array concatenation expression: readonly
              | array[int[8], 4, 2], readonly array[int[16], 5, 2]
               ,-[test:3:13]
             2 |     def f(readonly array[int[8], 4, 2] a, readonly array[int[16], 5, 2] b, mutable array[int[8], 9, 2] c) {
             3 |         c = a ++ b;
               :             ^^^^^^
             4 |     }
               `----
            ]"#]],
    );
}

#[allow(clippy::too_many_lines)]
#[test]
fn multidimensional_array_concatenation_with_different_types_errors() {
    let source = "
    def f(readonly array[int[8], 4, 2] a, readonly array[uint[8], 5, 2] b, mutable array[int[8], 9, 2] c) {
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
                    Stmt [5-134]:
                        annotations: <empty>
                        kind: DefStmt [5-134]:
                            symbol_id: 8
                            has_qubit_params: false
                            parameters:
                                9
                                10
                                11
                            return_type_span: [0-0]
                            body: Block [107-134]:
                                Stmt [117-128]:
                                    annotations: <empty>
                                    kind: AssignStmt [117-128]:
                                        lhs: Expr [117-118]:
                                            ty: mutable array[int[8], 9, 2]
                                            kind: SymbolId(11)
                                        rhs: Expr [121-127]:
                                            ty: unknown
                                            kind: ConcatExpr [121-127]:
                                                operands:
                                                    Expr [121-122]:
                                                        ty: readonly array[int[8], 4, 2]
                                                        kind: SymbolId(9)
                                                    Expr [126-127]:
                                                        ty: readonly array[uint[8], 5, 2]
                                                        kind: SymbolId(10)

            [Qasm.Lowerer.InconsistentTypesInArrayConcatenation

              x inconsistent types in array concatenation expression: readonly
              | array[int[8], 4, 2], readonly array[uint[8], 5, 2]
               ,-[test:3:13]
             2 |     def f(readonly array[int[8], 4, 2] a, readonly array[uint[8], 5, 2] b, mutable array[int[8], 9, 2] c) {
             3 |         c = a ++ b;
               :             ^^^^^^
             4 |     }
               `----
            ]"#]],
    );
}
