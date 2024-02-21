// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//use std::iter::Peekable;

use super::{
    concrete::{self, ConcreteToken, ConcreteTokenKind},
    //raw::{Lexer, Single, TokenKind},
    Delim,
    TokenKind,
};
use qsc_data_structures::span::Span;

#[derive(Debug)]
pub struct Edit {
    #[allow(dead_code)] // TODO: nobody's using this yet except for tests
    pub span: Span,
    #[allow(dead_code)] // TODO: nobody's using this yet except for tests
    pub new_text: String,
}

impl Edit {
    fn new(lo: u32, hi: u32, new_text: &str) -> Self {
        Self {
            span: Span { lo, hi },
            new_text: new_text.to_string(),
        }
    }

    fn new_with_span(span: Span, new_text: &str) -> Self {
        Self {
            span,
            new_text: new_text.to_string(),
        }
    }
}

fn make_indent_string(level: usize) -> String {
    "    ".repeat(level)
}

pub fn format(code: &str) -> Vec<Edit> {
    let tokens = concrete::ConcreteTokenIterator::new(code);
    let mut edits = vec![];

    let mut indent_level = 0;

    #[allow(unused_assignments)] // there's probably a better way of doing this, but this works
    let mut one = None;
    let mut two = None;
    let mut three = None;

    for token in tokens {
        // Advance the trio of tokens
        one = two;
        two = three;
        three = Some(token);

        let mut edits_for_triple = match (&one, &two, &three) {
            (Some(one), Some(two), Some(three)) => {
                // if the token is a {, increase the indent level
                if let ConcreteTokenKind::Cooked(TokenKind::Open(Delim::Brace)) = one.kind {
                    indent_level += 1;
                }

                // if the token is a }, decrease the indent level
                if let ConcreteTokenKind::Cooked(TokenKind::Close(Delim::Brace)) = one.kind {
                    #[allow(clippy::implicit_saturating_sub)]
                    if indent_level > 0 {
                        indent_level -= 1;
                    }
                }

                if let ConcreteTokenKind::WhiteSpace = one.kind {
                    // first token is whitespace, continue scanning
                    continue;
                } else if let ConcreteTokenKind::WhiteSpace = two.kind {
                    // whitespace in the middle
                    apply_rules(
                        one,
                        get_token_contents(code, two),
                        three,
                        code,
                        indent_level,
                    )
                } else {
                    // one, two are adjacent tokens with no whitespace in the middle
                    apply_rules(one, "", two, code, indent_level)
                }
            }
            _ => {
                // not enough tokens to apply a rule
                // TODO: we'll probably need to handle end-of-file cases here
                continue;
            }
        };

        edits.append(&mut edits_for_triple);
    }

    edits
}

fn fix_whitespace(whitespace: &str, indent_level: usize) -> String {
    //
    // when you see newline, insert the indent string
    // and trim until the next newline or the end of the string
    //

    let mut count_newlines = whitespace.chars().filter(|c| *c == '\n').count();

    // There should always be at least one newline
    if count_newlines < 1 {
        count_newlines = 1;
    }
    let mut new = "\n".repeat(count_newlines);
    new.push_str(&make_indent_string(indent_level));
    new
}

fn apply_rules(
    left: &ConcreteToken,
    whitespace: &str,
    right: &ConcreteToken,
    code: &str,
    indent_level: usize,
) -> Vec<Edit> {
    let mut edits = vec![];
    // when we get here, neither left nor right should be whitespace

    // if the right is a close brace, the indent level should be one less
    let indent_level = if let ConcreteTokenKind::Cooked(TokenKind::Close(Delim::Brace)) = right.kind
    {
        if indent_level > 0 {
            indent_level - 1
        } else {
            indent_level
        }
    } else {
        indent_level
    };

    match (&left.kind, &right.kind) {
        (
            ConcreteTokenKind::Cooked(TokenKind::Open(l)),
            ConcreteTokenKind::Cooked(TokenKind::Close(r)),
        ) if l == r => {
            rule_no_space(left, whitespace, right, &mut edits);
        }
        (ConcreteTokenKind::Comment, _) => {
            rule_trim_comments(left, &mut edits, code);
            rule_indentation(left, whitespace, right, &mut edits, indent_level);
        }
        (ConcreteTokenKind::Cooked(TokenKind::Semi), _) => match &right.kind {
            ConcreteTokenKind::Comment => {
                if whitespace.contains('\n') {
                    rule_indentation(left, whitespace, right, &mut edits, indent_level)
                }
            }
            _ => {
                rule_indentation(left, whitespace, right, &mut edits, indent_level);
            }
        },
        (ConcreteTokenKind::Cooked(TokenKind::Open(Delim::Brace)), _) => {
            rule_indentation(left, whitespace, right, &mut edits, indent_level);
        }
        (ConcreteTokenKind::Cooked(TokenKind::Close(Delim::Brace)), _) => {
            rule_indentation(left, whitespace, right, &mut edits, indent_level);
        }
        (ConcreteTokenKind::Cooked(cooked_left), ConcreteTokenKind::Cooked(cooked_right)) => {
            match (cooked_left, cooked_right) {
                (TokenKind::Ident, TokenKind::Ident)
                | (TokenKind::Keyword(_), TokenKind::Ident)
                | (TokenKind::Ident, TokenKind::Colon)
                | (TokenKind::Colon, TokenKind::Ident)
                | (TokenKind::Comma, _) => {
                    rule_single_space(left, whitespace, right, &mut edits);
                }
                (TokenKind::Ident, TokenKind::Open(Delim::Paren))
                | (TokenKind::Ident, TokenKind::Open(Delim::Bracket))
                | (TokenKind::Ident, TokenKind::Comma)
                | (TokenKind::Open(_), _)
                | (_, TokenKind::Close(_)) => {
                    rule_no_space(left, whitespace, right, &mut edits);
                }
                _ => {}
            }
        }
        _ => {}
    }

    println!(
        "edits for `{}` : {edits:?}",
        &code[left.span.lo as usize..right.span.hi as usize]
    );
    edits
}

fn rule_no_space(
    left: &ConcreteToken,
    whitespace: &str,
    right: &ConcreteToken,
    edits: &mut Vec<Edit>,
) {
    if whitespace != "" {
        edits.push(Edit::new(left.span.hi, right.span.lo, ""));
    }
}

fn rule_single_space(
    left: &ConcreteToken,
    whitespace: &str,
    right: &ConcreteToken,
    edits: &mut Vec<Edit>,
) {
    if whitespace != " " {
        edits.push(Edit::new(left.span.hi, right.span.lo, " "));
    }
}

fn rule_trim_comments(left: &ConcreteToken, edits: &mut Vec<Edit>, code: &str) {
    // fix trailing spaces on the comment
    let comment_contents = get_token_contents(code, left);
    let new_comment_contents = comment_contents.trim_end();
    if comment_contents != new_comment_contents {
        edits.push(Edit::new_with_span(left.span, new_comment_contents));
    }
}

fn rule_indentation(
    left: &ConcreteToken,
    whitespace: &str,
    right: &ConcreteToken,
    edits: &mut Vec<Edit>,
    indent_level: usize,
) {
    // if the middle whitespace contains a new line, we need to
    // fix the indentation
    let new_whitespace = fix_whitespace(whitespace, indent_level);
    if whitespace != new_whitespace {
        edits.push(Edit::new(
            left.span.hi,
            right.span.lo,
            new_whitespace.as_str(),
        ));
    }
}

fn get_token_contents<'a>(code: &'a str, token: &ConcreteToken) -> &'a str {
    &code[token.span.lo as usize..token.span.hi as usize]
}

#[cfg(test)]
mod tests {
    use expect_test::expect;
    use indoc::indoc;

    #[test]
    fn test_formatting() {
        let code = "operation   Foo   ()";
        let edits = super::format(code);
        expect![[r#"
            [
                Edit {
                    span: Span {
                        lo: 9,
                        hi: 12,
                    },
                    new_text: " ",
                },
                Edit {
                    span: Span {
                        lo: 15,
                        hi: 18,
                    },
                    new_text: "",
                },
            ]
        "#]]
        .assert_debug_eq(&edits);
    }

    #[test]
    fn test_braces() {
        let code = indoc! {r#"
        operation Foo() : Unit {}
        operation Bar() : Unit {
            operation Baz() : Unit {}
        }
        "#};
        let edits = super::format(code);
        expect![[r#"
            [
                Edit {
                    span: Span {
                        lo: 24,
                        hi: 24,
                    },
                    new_text: "\n",
                },
                Edit {
                    span: Span {
                        lo: 79,
                        hi: 79,
                    },
                    new_text: "\n    ",
                },
            ]
        "#]]
        .assert_debug_eq(&edits);
    }

    #[test]
    fn test_formatting_2() {
        let code = indoc! {r#"
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
        let edits = super::format(code);
        expect![[r#"
            []
        "#]]
        .assert_debug_eq(&edits);
    }
}
