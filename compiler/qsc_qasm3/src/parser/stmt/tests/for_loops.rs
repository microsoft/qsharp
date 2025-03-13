// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::parser::{stmt::parse, tests::check};
use expect_test::expect;

#[test]
fn simple_for_loop() {
    check(
        parse,
        "
    for int x in {1, 2, 3} {
        a = 0;
    }",
        &expect![[r#"
            Stmt [5-50]:
                annotations: <empty>
                kind: ForStmt [5-50]:
                    variable_type: ScalarType [9-12]: IntType [9-12]:
                        size: <none>
                    variable_name: Ident [13-14] "x"
                    iterable: DiscreteSet [18-27]:
                        values:
                            Expr [19-20]: Lit: Int(1)
                            Expr [22-23]: Lit: Int(2)
                            Expr [25-26]: Lit: Int(3)
                    body: Stmt [28-50]:
                        annotations: <empty>
                        kind: Block [28-50]:
                            Stmt [38-44]:
                                annotations: <empty>
                                kind: AssignStmt [38-44]:
                                    lhs: IndexedIdent [38-39]:
                                        name: Ident [38-39] "a"
                                        indices: <empty>
                                    rhs: Expr [42-43]: Lit: Int(0)"#]],
    );
}

#[test]
fn empty_for_loop() {
    check(
        parse,
        "for int x in {} {}",
        &expect![[r#"
            Stmt [0-18]:
                annotations: <empty>
                kind: ForStmt [0-18]:
                    variable_type: ScalarType [4-7]: IntType [4-7]:
                        size: <none>
                    variable_name: Ident [8-9] "x"
                    iterable: DiscreteSet [13-15]:
                        values: <empty>
                    body: Stmt [16-18]:
                        annotations: <empty>
                        kind: Block [16-18]: <empty>"#]],
    );
}

#[test]
fn simple_for_loop_stmt_body() {
    check(
        parse,
        "
    for int x in {1, 2, 3}
        a = 0;
    ",
        &expect![[r#"
            Stmt [5-42]:
                annotations: <empty>
                kind: ForStmt [5-42]:
                    variable_type: ScalarType [9-12]: IntType [9-12]:
                        size: <none>
                    variable_name: Ident [13-14] "x"
                    iterable: DiscreteSet [18-27]:
                        values:
                            Expr [19-20]: Lit: Int(1)
                            Expr [22-23]: Lit: Int(2)
                            Expr [25-26]: Lit: Int(3)
                    body: Stmt [36-42]:
                        annotations: <empty>
                        kind: AssignStmt [36-42]:
                            lhs: IndexedIdent [36-37]:
                                name: Ident [36-37] "a"
                                indices: <empty>
                            rhs: Expr [40-41]: Lit: Int(0)"#]],
    );
}

#[test]
fn for_loop_range() {
    check(
        parse,
        "
    for int x in [0:2:7] {
        a = 0;
    }",
        &expect![[r#"
            Stmt [5-48]:
                annotations: <empty>
                kind: ForStmt [5-48]:
                    variable_type: ScalarType [9-12]: IntType [9-12]:
                        size: <none>
                    variable_name: Ident [13-14] "x"
                    iterable: RangeDefinition [18-25]:
                        start: Expr [19-20]: Lit: Int(0)
                        step: Expr [21-22]: Lit: Int(2)
                        end: Expr [23-24]: Lit: Int(7)
                    body: Stmt [26-48]:
                        annotations: <empty>
                        kind: Block [26-48]:
                            Stmt [36-42]:
                                annotations: <empty>
                                kind: AssignStmt [36-42]:
                                    lhs: IndexedIdent [36-37]:
                                        name: Ident [36-37] "a"
                                        indices: <empty>
                                    rhs: Expr [40-41]: Lit: Int(0)"#]],
    );
}

#[test]
fn for_loop_range_no_step() {
    check(
        parse,
        "
    for int x in [0:7] {
        a = 0;
    }",
        &expect![[r#"
            Stmt [5-46]:
                annotations: <empty>
                kind: ForStmt [5-46]:
                    variable_type: ScalarType [9-12]: IntType [9-12]:
                        size: <none>
                    variable_name: Ident [13-14] "x"
                    iterable: RangeDefinition [18-23]:
                        start: Expr [19-20]: Lit: Int(0)
                        step: <none>
                        end: Expr [21-22]: Lit: Int(7)
                    body: Stmt [24-46]:
                        annotations: <empty>
                        kind: Block [24-46]:
                            Stmt [34-40]:
                                annotations: <empty>
                                kind: AssignStmt [34-40]:
                                    lhs: IndexedIdent [34-35]:
                                        name: Ident [34-35] "a"
                                        indices: <empty>
                                    rhs: Expr [38-39]: Lit: Int(0)"#]],
    );
}

#[test]
fn for_loop_expr() {
    check(
        parse,
        "
    for int x in xs {
        a = 0;
    }",
        &expect![[r#"
            Stmt [5-43]:
                annotations: <empty>
                kind: ForStmt [5-43]:
                    variable_type: ScalarType [9-12]: IntType [9-12]:
                        size: <none>
                    variable_name: Ident [13-14] "x"
                    iterable: Expr [18-20]: Ident [18-20] "xs"
                    body: Stmt [21-43]:
                        annotations: <empty>
                        kind: Block [21-43]:
                            Stmt [31-37]:
                                annotations: <empty>
                                kind: AssignStmt [31-37]:
                                    lhs: IndexedIdent [31-32]:
                                        name: Ident [31-32] "a"
                                        indices: <empty>
                                    rhs: Expr [35-36]: Lit: Int(0)"#]],
    );
}

#[test]
fn for_loop_with_continue_stmt() {
    check(
        parse,
        "
    for int x in {1, 2, 3} {
        a = 0;
        continue;
    }",
        &expect![[r#"
            Stmt [5-68]:
                annotations: <empty>
                kind: ForStmt [5-68]:
                    variable_type: ScalarType [9-12]: IntType [9-12]:
                        size: <none>
                    variable_name: Ident [13-14] "x"
                    iterable: DiscreteSet [18-27]:
                        values:
                            Expr [19-20]: Lit: Int(1)
                            Expr [22-23]: Lit: Int(2)
                            Expr [25-26]: Lit: Int(3)
                    body: Stmt [28-68]:
                        annotations: <empty>
                        kind: Block [28-68]:
                            Stmt [38-44]:
                                annotations: <empty>
                                kind: AssignStmt [38-44]:
                                    lhs: IndexedIdent [38-39]:
                                        name: Ident [38-39] "a"
                                        indices: <empty>
                                    rhs: Expr [42-43]: Lit: Int(0)
                            Stmt [53-62]:
                                annotations: <empty>
                                kind: ContinueStmt [53-62]"#]],
    );
}

#[test]
fn for_loop_with_break_stmt() {
    check(
        parse,
        "
    for int x in {1, 2, 3} {
        a = 0;
        break;
    }",
        &expect![[r#"
            Stmt [5-65]:
                annotations: <empty>
                kind: ForStmt [5-65]:
                    variable_type: ScalarType [9-12]: IntType [9-12]:
                        size: <none>
                    variable_name: Ident [13-14] "x"
                    iterable: DiscreteSet [18-27]:
                        values:
                            Expr [19-20]: Lit: Int(1)
                            Expr [22-23]: Lit: Int(2)
                            Expr [25-26]: Lit: Int(3)
                    body: Stmt [28-65]:
                        annotations: <empty>
                        kind: Block [28-65]:
                            Stmt [38-44]:
                                annotations: <empty>
                                kind: AssignStmt [38-44]:
                                    lhs: IndexedIdent [38-39]:
                                        name: Ident [38-39] "a"
                                        indices: <empty>
                                    rhs: Expr [42-43]: Lit: Int(0)
                            Stmt [53-59]:
                                annotations: <empty>
                                kind: BreakStmt [53-59]"#]],
    );
}

#[test]
fn single_stmt_for_stmt() {
    check(
        parse,
        "for int x in {} z q;",
        &expect![[r#"
            Stmt [0-20]:
                annotations: <empty>
                kind: ForStmt [0-20]:
                    variable_type: ScalarType [4-7]: IntType [4-7]:
                        size: <none>
                    variable_name: Ident [8-9] "x"
                    iterable: DiscreteSet [13-15]:
                        values: <empty>
                    body: Stmt [16-20]:
                        annotations: <empty>
                        kind: GateCall [16-20]:
                            modifiers: <empty>
                            name: Ident [16-17] "z"
                            args: <empty>
                            duration: <none>
                            qubits:
                                IndexedIdent [18-19]:
                                    name: Ident [18-19] "q"
                                    indices: <empty>"#]],
    );
}

#[test]
fn annotations_in_single_stmt_for_stmt() {
    check(
        parse,
        "
    for int x in {}
        @foo
        @bar
        x = 5;",
        &expect![[r#"
            Stmt [5-61]:
                annotations: <empty>
                kind: ForStmt [5-61]:
                    variable_type: ScalarType [9-12]: IntType [9-12]:
                        size: <none>
                    variable_name: Ident [13-14] "x"
                    iterable: DiscreteSet [18-20]:
                        values: <empty>
                    body: Stmt [29-61]:
                        annotations:
                            Annotation [29-33]:
                                identifier: "foo"
                                value: <none>
                            Annotation [42-46]:
                                identifier: "bar"
                                value: <none>
                        kind: AssignStmt [55-61]:
                            lhs: IndexedIdent [55-56]:
                                name: Ident [55-56] "x"
                                indices: <empty>
                            rhs: Expr [59-60]: Lit: Int(5)"#]],
    );
}

#[test]
fn nested_single_stmt_for_stmt() {
    check(
        parse,
        "for int x in {} for int y in {} z q;",
        &expect![[r#"
            Stmt [0-36]:
                annotations: <empty>
                kind: ForStmt [0-36]:
                    variable_type: ScalarType [4-7]: IntType [4-7]:
                        size: <none>
                    variable_name: Ident [8-9] "x"
                    iterable: DiscreteSet [13-15]:
                        values: <empty>
                    body: Stmt [16-36]:
                        annotations: <empty>
                        kind: ForStmt [16-36]:
                            variable_type: ScalarType [20-23]: IntType [20-23]:
                                size: <none>
                            variable_name: Ident [24-25] "y"
                            iterable: DiscreteSet [29-31]:
                                values: <empty>
                            body: Stmt [32-36]:
                                annotations: <empty>
                                kind: GateCall [32-36]:
                                    modifiers: <empty>
                                    name: Ident [32-33] "z"
                                    args: <empty>
                                    duration: <none>
                                    qubits:
                                        IndexedIdent [34-35]:
                                            name: Ident [34-35] "q"
                                            indices: <empty>"#]],
    );
}
