// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::{expect, Expect};
use indoc::indoc;

fn check(input: &str, expect: &Expect) {
    let actual = super::format_str(input);
    expect.assert_eq(&actual);
}

// Removing trailing whitespace from lines

#[test]
fn remove_trailing_spaces() {
    let extra_spaces = "    ";
    let input = format!(
        "/// Doc Comment with trailing spaces{extra_spaces}
        operation Foo() : Unit {{
            // Comment with trailing spaces{extra_spaces}
            let x = 3;   // In-line comment with trailing spaces{extra_spaces}
            let y = 4;{extra_spaces}
        }}
"
    );

    check(
        input.as_str(),
        &expect![[r#"
        /// Doc Comment with trailing spaces
        operation Foo() : Unit {
            // Comment with trailing spaces
            let x = 3;   // In-line comment with trailing spaces
            let y = 4;
        }
        "#]],
    );
}

#[test]
fn preserve_string_trailing_spaces() {
    let extra_spaces = "    ";
    let input = format!(
        "\"Hello{extra_spaces}
World\""
    );

    assert!(super::calculate_format_edits(input.as_str()).is_empty());
}

// Namespace items begin on their own lines

#[test]
fn namespace_items_begin_on_their_own_lines() {
    check(
        "operation Foo() : Unit {} operation Bar() : Unit {}",
        &expect![[r#"
            operation Foo() : Unit {}
            operation Bar() : Unit {}"#]],
    );
}

// Single spaces around most binary operators

#[test]
fn singe_space_around_arithmetic_bin_ops() {
    // Note that `-` is missing at this time due to it being unsupported for formatting.
    check(
        indoc! {"
    1+2;
    1   *   2;
    4  /2;
    3%  2;
    2  ^  3;
    "},
        &expect![[r#"
            1 + 2;
            1 * 2;
            4 / 2;
            3 % 2;
            2 ^ 3;
        "#]],
    );
}

#[test]
fn singe_space_around_bit_wise_bin_ops() {
    check(
        indoc! {"
    1&&&2;
    1   |||   2;
    4  ^^^2;
    3<<<  2;
    2  >>>  3;
    "},
        &expect![[r#"
            1 &&& 2;
            1 ||| 2;
            4 ^^^ 2;
            3 <<< 2;
            2 >>> 3;
        "#]],
    );
}

#[test]
fn singe_space_around_boolean_bin_ops() {
    check(
        indoc! {"
    true   and  false;
    true   or   false;
    "},
        &expect![[r#"
            true and false;
            true or false;
        "#]],
    );
}

#[test]
fn singe_space_around_bin_op_equals() {
    check(
        indoc! {"
    let x    +=    y;
    let x    -=y;
    let x*=    y;
    let x   /=   y;
    let x    %=    y;
    "},
        &expect![[r#"
            let x += y;
            let x -= y;
            let x *= y;
            let x /= y;
            let x %= y;
        "#]],
    );
}

#[test]
fn singe_space_around_equals() {
    check("let x   =   3;", &expect!["let x = 3;"]);
}

#[test]
fn singe_space_around_colon() {
    check("let x   :    Int = 3;", &expect!["let x : Int = 3;"]);
}

#[test]
fn singe_space_around_comp_ops() {
    // Note that `<` and `>` are missing at this time due to them being unsupported for formatting.
    check(
        indoc! {"
    x    <=y;
    x   >=   y;
    x    ==    y;
    x    !=    y;
    "},
        &expect![[r#"
            x <= y;
            x >= y;
            x == y;
            x != y;
        "#]],
    );
}

#[test]
fn singe_space_around_ternary() {
    check("x?   3|  4", &expect!["x ? 3 | 4"]);
}

#[test]
fn singe_space_around_copy() {
    check("x  w/3  <-   4", &expect!["x w/ 3 <- 4"]);
}

#[test]
fn singe_space_around_copy_and_update() {
    check("x  w/=3  <-   4", &expect!["x w/= 3 <- 4"]);
}

#[test]
fn singe_space_around_lambda_ops() {
    check(
        indoc! {"
    let x = ()   ->    ();
    let y = ()=>();
    "},
        &expect![[r#"
            let x = () -> ();
            let y = () => ();
        "#]],
    );
}

#[test]
fn singe_space_around_characteristic_expr() {
    check(
        "operation Foo() : Unit    is    Adj+Ctl {}",
        &expect!["operation Foo() : Unit is Adj + Ctl {}"],
    );
}

#[test]
fn singe_space_around_functors() {
    check(
        "Controlled     Adjoint   Foo()",
        &expect!["Controlled Adjoint Foo()"],
    );
}

#[test]
fn singe_space_around_as() {
    check(
        "open thing    as    other;",
        &expect!["open thing as other;"],
    );
}

// No space between unary operators and their operand

#[test]
fn no_space_before_unwrap() {
    check("let x = foo  !;", &expect!["let x = foo!;"]);
}

#[test]
fn no_space_after_bit_negation() {
    check("let x =   ~~~   3;", &expect!["let x = ~~~3;"]);
}

#[test]
fn single_space_around_boolean_negation() {
    check("let x =   not   3;", &expect!["let x = not 3;"]);
}

// No space after open parentheses and brackets and before close parentheses and brackets

#[test]
fn no_space_for_parentheses() {
    check("(  12, 13, 14   )", &expect!["(12, 13, 14)"]);
}

#[test]
fn no_space_for_brackets() {
    check("[  12 + 13 + 14   ]", &expect!["[12 + 13 + 14]"]);
}

// No space after open string-interpolation argument braces and before close string-interpolation argument braces

#[test]
fn no_space_for_string_interpolation_argument_braces() {
    check(
        r#"let x = $"First { 1 + 1 } Third";"#,
        &expect![[r#"let x = $"First {1 + 1} Third";"#]],
    );
}

// No space before commas or semicolons

#[test]
fn no_space_before_comma() {
    check("(12  ,  13 , 14)", &expect!["(12, 13, 14)"]);
}

#[test]
fn no_space_before_semicolons() {
    check("let x = 3  ;", &expect!["let x = 3;"]);
}

// Newline after semicolons

#[test]
fn newline_after_semicolon() {
    check(
        "let x = 3; let y = 2;",
        &expect![[r#"
        let x = 3;
        let y = 2;"#]],
    );
}

#[test]
fn preserve_eol_comment() {
    let input = indoc! {"let x = 3;    // End-of-line Comment
        let y = 2;
        "};
    assert!(super::calculate_format_edits(input).is_empty());
}

// Newline before declaration keywords

#[test]
fn newline_before_let() {
    check(
        "let x = 3; {} let y = 2;",
        &expect![[r#"
        let x = 3;
        {}
        let y = 2;"#]],
    );
}

#[test]
fn newline_before_mutable() {
    check(
        "mutable x = 3; {} mutable y = 2;",
        &expect![[r#"
        mutable x = 3;
        {}
        mutable y = 2;"#]],
    );
}

#[test]
fn newline_before_set() {
    check(
        "set x = 3; {} set y = 2;",
        &expect![[r#"
        set x = 3;
        {}
        set y = 2;"#]],
    );
}

#[test]
fn newline_before_use() {
    check(
        "use q = Qubit(); {} use w = Qubit();",
        &expect![[r#"
        use q = Qubit();
        {}
        use w = Qubit();"#]],
    );
}

#[test]
fn newline_before_borrow() {
    check(
        "borrow q = Qubit(); {} borrow w = Qubit();",
        &expect![[r#"
        borrow q = Qubit();
        {}
        borrow w = Qubit();"#]],
    );
}

// Single space before control-flow-helper keywords

#[test]
fn single_space_before_in() {
    check("for x    in 0..2 {}", &expect![[r#"for x in 0..2 {}"#]]);
}

#[test]
fn single_space_before_until() {
    check(
        "repeat {}    until x   fixup {}",
        &expect![[r#"
            repeat {} until x
            fixup {}"#]],
    );
}

#[test]
fn single_space_before_elif_and_else() {
    check(
        "if x {}    elif y {}     else {}",
        &expect!["if x {} elif y {} else {}"],
    );
}

#[test]
fn single_space_before_apply() {
    check("within {}    apply {}", &expect!["within {} apply {}"]);
}

// No space between caller expressions and argument tuple

#[test]
fn no_space_in_front_of_argument_tuple() {
    check("Foo   (1, 2, 3)", &expect!["Foo(1, 2, 3)"]);
}

#[test]
fn no_space_in_front_of_parameter_tuple() {
    check(
        "operation Foo     (x : Int, y : Int) : Unit {}",
        &expect!["operation Foo(x : Int, y : Int) : Unit {}"],
    );
}

// No space between array expressions and indexing brackets

#[test]
fn no_space_in_front_of_array_indexing() {
    check("arr   [4]", &expect!["arr[4]"]);
}

// No space around `.`, `..`, and `::` operators

#[test]
fn no_space_around_dot_operator() {
    check("let x = thing . other;", &expect!["let x = thing.other;"]);
}

#[test]
fn no_space_around_range_operator() {
    check("let x = 1 .. 4;", &expect!["let x = 1..4;"]);
}

#[test]
fn no_space_around_field_operator() {
    check("let x = thing :: other;", &expect!["let x = thing::other;"]);
}

// No space between the `…` operator and any possible operands on either side

#[test]
fn no_space_around_full_range_in_slice() {
    check("let x = y[   ...   ];", &expect!["let x = y[...];"]);
}

#[test]
fn no_space_between_open_end_range_and_operand() {
    check("let x = 15 ...;", &expect!["let x = 15...;"]);
}

#[test]
fn no_space_between_open_start_range_and_operand() {
    check("let x = ... 15;", &expect!["let x = ...15;"]);
}

// Single space before open brace and newline after, except empty blocks have no space

#[test]
fn single_space_before_open_brace_and_newline_after() {
    check(
        indoc! {r#"
        operation Foo() : Unit{ let x = 3; }
        operation Bar() : Unit
        { { let x = 3; }{ let x = 4; } }
        "#},
        &expect![[r#"
            operation Foo() : Unit {
                let x = 3;
            }
            operation Bar() : Unit
            {
                {
                    let x = 3;
                } {
                    let x = 4;
                }
            }
        "#]],
    );
}

#[test]
fn remove_spaces_between_empty_delimiters() {
    check(
        indoc! {r#"
        operation Foo() : Unit {
        }
        operation Bar() : Unit {
            operation Baz() : Unit {   }
            let x = {

            };
            let y : Int[] = [ ];
            let z = (

             );
        }
        "#},
        &expect![[r#"
            operation Foo() : Unit {}
            operation Bar() : Unit {
                operation Baz() : Unit {}
                let x = {};
                let y : Int[] = [];
                let z = ();
            }
        "#]],
    );
}

// Correct indentation, which increases by four spaces when a brace-delimited block is opened and decreases when block is closed

#[test]
fn formatting_corrects_indentation() {
    check(
        r#"
    /// First
/// Second
    /// Third
        namespace MyQuantumProgram {
        open Microsoft.Quantum.Diagnostics;

        @EntryPoint()
        operation Main() : Int {
    let x = 3;
            let y = 4;

                    // Comment
            return 5;
        }
            }
"#,
        &expect![[r#"
            /// First
            /// Second
            /// Third
            namespace MyQuantumProgram {
                open Microsoft.Quantum.Diagnostics;

                @EntryPoint()
                operation Main() : Int {
                    let x = 3;
                    let y = 4;

                    // Comment
                    return 5;
                }
            }
        "#]],
    );
}

#[test]
fn preserve_string_indentation() {
    let input = r#""Hello
    World""#;

    assert!(super::calculate_format_edits(input).is_empty());
}

// Will respect user new-lines and indentation added into expressions

#[test]
fn preserve_user_newlines_in_expressions() {
    let input = indoc! {r#"
        let x = [
            thing1,
            thing2,
            thing3,
        ];
        let y = 1 + 2 + 3 + 4 + 5 +
                6 + 7 + 8 + 9 + 10;
        "#};
    assert!(super::calculate_format_edits(input).is_empty());
}

// Extra test case for sanity

#[test]
fn sample_has_no_formatting_changes() {
    let input = indoc! {r#"
        /// # Sample
        /// Joint Measurement
        ///
        /// # Description
        /// Joint measurements, also known as Pauli measurements, are a generalization
        /// of 2-outcome measurements to multiple qubits and other bases.
        namespace Sample {
            open Microsoft.Quantum.Diagnostics;

            @EntryPoint()
            operation Main() : (Result, Result[]) {
                // Prepare an entangled state.
                use qs = Qubit[2];  // |00〉
                H(qs[0]);           // 1/sqrt(2)(|00〉 + |10〉)
                CNOT(qs[0], qs[1]); // 1/sqrt(2)(|00〉 + |11〉)

                // Show the quantum state before performing the joint measurement.
                DumpMachine();

                // The below code uses a joint measurement as a way to check the parity
                // of the first two qubits. In this case, the parity measurement result
                // will always be `Zero`.
                // Notice how the state was not collapsed by the joint measurement.
                let parityResult = Measure([PauliZ, PauliZ], qs[...1]);
                DumpMachine();

                // However, if we perform a measurement just on the first qubit, we can
                // see how the state collapses.
                let firstQubitResult = M(qs[0]);
                DumpMachine();

                // Measuring the last qubit does not change the quantum state
                // since the state of the second qubit collapsed when the first qubit
                // was measured because they were entangled.
                let secondQubitResult = M(qs[1]);
                DumpMachine();

                ResetAll(qs);
                return (parityResult, [firstQubitResult, secondQubitResult]);
            }
        }
        "#};
    assert!(super::calculate_format_edits(input).is_empty());
}
