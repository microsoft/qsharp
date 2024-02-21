// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//use std::iter::Peekable;

use super::{
    concrete::{self, ConcreteToken},
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

fn make_indent_string(level: usize) -> String {
    "    ".repeat(level)
}

// #[derive(Clone, Copy)]
// struct SpannedToken {
//     pub kind: TokenKind,
//     pub span: Span,
// }

// struct SpannedTokenIterator<'a> {
//     code: &'a str,
//     tokens: Peekable<Lexer<'a>>,
// }

// impl<'a> SpannedTokenIterator<'a> {
//     fn new(code: &'a str) -> Self {
//         Self {
//             code,
//             tokens: Lexer::new(code).peekable(),
//         }
//     }
// }

// impl Iterator for SpannedTokenIterator<'_> {
//     type Item = SpannedToken;

//     fn next(&mut self) -> Option<Self::Item> {
//         let token = self.tokens.next()?;
//         let next = self.tokens.peek();
//         Some(SpannedToken {
//             kind: token.kind,
//             span: Span {
//                 lo: token.offset,
//                 hi: next
//                     .map(|t| t.offset)
//                     .unwrap_or_else(|| self.code.len() as u32),
//             },
//         })
//     }
// }

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
                if let ConcreteToken::Cooked(cooked) = one {
                    if cooked.kind == TokenKind::Open(Delim::Brace) {
                        indent_level += 1;
                    }
                }

                // if the token is a }, decrease the indent level
                if let ConcreteToken::Cooked(cooked) = one {
                    if cooked.kind == TokenKind::Close(Delim::Brace) {
                        indent_level -= 1;
                    }
                }

                if let ConcreteToken::WhiteSpace(_) = one {
                    // first token is whitespace, continue scanning
                    continue;
                } else if let ConcreteToken::WhiteSpace(_) = two {
                    // whitespace in the middle
                    apply_rule(
                        one,
                        get_token_contents(code, two),
                        three,
                        code,
                        indent_level,
                    )
                } else {
                    // one, two are adjacent tokens with no whitespace in the middle
                    apply_rule(one, "", two, code, indent_level)
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

fn apply_rule(
    left: &ConcreteToken,
    whitespace: &str,
    right: &ConcreteToken,
    code: &str,
    indent_level: usize,
) -> Vec<Edit> {
    let mut edits = vec![];
    // when we get here, neither left nor right should be whitespace

    // if the right is a close brace, the indent level should be one less
    let indent_level = if let ConcreteToken::Cooked(cooked) = right {
        if let TokenKind::Close(Delim::Brace) = cooked.kind {
            indent_level - 1
        } else {
            indent_level
        }
    } else {
        indent_level
    };

    match (&left, &right) {
        (ConcreteToken::Comment(span), _) => {
            // fix indentation
            // and fix trailing spaces on the left comment
            let comment_contents = get_token_contents(code, left);
            let new_comment_contents = comment_contents.trim_end();
            if comment_contents != new_comment_contents {
                edits.push(Edit {
                    span: *span,
                    new_text: new_comment_contents.to_string(),
                });
            }

            // if the middle whitespace contains a new line, we need to
            // fix the indentation
            let new_whitespace = fix_whitespace(whitespace, indent_level);
            if whitespace != new_whitespace {
                edits.push(Edit {
                    span: Span {
                        lo: left.get_span().hi,
                        hi: right.get_span().lo,
                    },
                    new_text: new_whitespace.to_string(),
                });
            }
        }
        (ConcreteToken::Cooked(cooked_left), _)
            if matches!(cooked_left.kind, TokenKind::Open(Delim::Brace)) =>
        {
            let span = cooked_left.span;
            // fix indentation
            // and fix trailing spaces on the left
            let contents = get_token_contents(code, left);
            let new_contents = contents.trim_end();
            if contents != new_contents {
                edits.push(Edit {
                    span,
                    new_text: new_contents.to_string(),
                });
            }

            // if the middle whitespace contains a new line, we need to
            // fix the indentation
            let new_whitespace = fix_whitespace(whitespace, indent_level);
            if whitespace != new_whitespace {
                edits.push(Edit {
                    span: Span {
                        lo: left.get_span().hi,
                        hi: right.get_span().lo,
                    },
                    new_text: new_whitespace.to_string(),
                });
            }
        }
        (ConcreteToken::Cooked(cooked_left), _)
            if matches!(cooked_left.kind, TokenKind::Close(Delim::Brace)) =>
        {
            let span = cooked_left.span;
            // fix indentation
            // and fix trailing spaces on the left
            let contents = get_token_contents(code, left);
            let new_contents = contents.trim_end();
            if contents != new_contents {
                edits.push(Edit {
                    span,
                    new_text: new_contents.to_string(),
                });
            }

            // if the middle whitespace contains a new line, we need to
            // fix the indentation
            let new_whitespace = fix_whitespace(whitespace, indent_level);
            if whitespace != new_whitespace {
                edits.push(Edit {
                    span: Span {
                        lo: left.get_span().hi,
                        hi: right.get_span().lo,
                    },
                    new_text: new_whitespace.to_string(),
                });
            }
        }
        (ConcreteToken::Cooked(cooked_left), ConcreteToken::Cooked(cooked_right)) => {
            match (cooked_left.kind, cooked_right.kind) {
                (TokenKind::Ident, TokenKind::Ident)
                | (TokenKind::Keyword(_), TokenKind::Ident) =>
                //| (TokenKind::Single(Single::Colon), TokenKind::Ident)
                //| (TokenKind::Ident, TokenKind::Single(_))
                {
                    // Put exactly one space in the middle
                    let old_whitespace = whitespace;
                    let new_whitespace = " ";
                    if old_whitespace != new_whitespace {
                        edits.push(Edit {
                            span: Span {
                                lo: left.get_span().hi,
                                hi: right.get_span().lo,
                            },
                            new_text: new_whitespace.to_string(),
                        });
                    }
                }
                (TokenKind::Ident, TokenKind::Open(Delim::Paren))
                | (TokenKind::Ident, TokenKind::Open(Delim::Bracket)) => {
                    // Put no space in the middle
                    let old_whitespace = whitespace;
                    let new_whitespace = "";
                    if old_whitespace != new_whitespace {
                        edits.push(Edit {
                            span: Span {
                                lo: left.get_span().hi,
                                hi: right.get_span().lo,
                            },
                            new_text: new_whitespace.to_string(),
                        });
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }

    println!(
        "edits for `{}` : {edits:?}",
        &code[left.get_span().lo as usize..right.get_span().hi as usize]
    );
    edits
}

fn get_token_contents<'a>(code: &'a str, token: &ConcreteToken) -> &'a str {
    let span = match token {
        ConcreteToken::Cooked(cooked) => cooked.span,
        ConcreteToken::Error(err) => err.get_span(),
        ConcreteToken::WhiteSpace(span) | ConcreteToken::Comment(span) => *span,
    };
    &code[span.lo as usize..span.hi as usize]
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
        let code = "/// # Sample
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
        ";
        let edits = super::format(code);
        expect![[r#""#]].assert_debug_eq(&edits);
    }
}
