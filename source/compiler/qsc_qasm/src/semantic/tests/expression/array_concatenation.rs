// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod array;
mod dyn_array_ref;
mod static_array_ref;

// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::semantic::tests::check_stmt_kinds as check;
use expect_test::expect;

#[test]
fn concatenating_static_and_dynamic_array_refs_errors() {
    let source = "
    def f(readonly array[int, 3] a, readonly array[int, #dim = 1] b, mutable array[int, #dim = 1] c) {
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
                Stmt [5-129]:
                    annotations: <empty>
                    kind: DefStmt [5-129]:
                        symbol_id: 8
                        has_qubit_params: false
                        parameters:
                            9
                            10
                            11
                        return_type_span: [0-0]
                        body: Block [102-129]:
                            Stmt [112-123]:
                                annotations: <empty>
                                kind: AssignStmt [112-123]:
                                    lhs: Expr [112-113]:
                                        ty: mutable array[int, #dim = 1]
                                        kind: SymbolId(11)
                                    rhs: Expr [116-122]:
                                        ty: unknown
                                        kind: ConcatExpr [116-122]:
                                            operands:
                                                Expr [116-117]:
                                                    ty: readonly array[int, 3]
                                                    kind: SymbolId(9)
                                                Expr [121-122]:
                                                    ty: readonly array[int, #dim = 1]
                                                    kind: SymbolId(10)

        [Qasm.Lowerer.InconsistentTypesInArrayConcatenation

          x inconsistent types in array concatenation expression: readonly array[int,
          | 3], readonly array[int, #dim = 1]
           ,-[test:3:13]
         2 |     def f(readonly array[int, 3] a, readonly array[int, #dim = 1] b, mutable array[int, #dim = 1] c) {
         3 |         c = a ++ b;
           :             ^^^^^^
         4 |     }
           `----
        ]"#]],
    );
}
