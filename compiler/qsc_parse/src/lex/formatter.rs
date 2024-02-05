use std::iter::Peekable;

use super::{
    raw::{Lexer, Single, TokenKind},
    Delim,
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

#[derive(Clone, Copy)]
struct SpannedToken {
    pub kind: TokenKind,
    pub span: Span,
}

struct SpannedTokenIterator<'a> {
    code: &'a str,
    tokens: Peekable<Lexer<'a>>,
}

impl<'a> SpannedTokenIterator<'a> {
    fn new(code: &'a str) -> Self {
        Self {
            code,
            tokens: Lexer::new(code).peekable(),
        }
    }
}

impl Iterator for SpannedTokenIterator<'_> {
    type Item = SpannedToken;

    fn next(&mut self) -> Option<Self::Item> {
        let token = self.tokens.next()?;
        let next = self.tokens.peek();
        Some(SpannedToken {
            kind: token.kind,
            span: Span {
                lo: token.offset,
                hi: next
                    .map(|t| t.offset)
                    .unwrap_or_else(|| self.code.len() as u32),
            },
        })
    }
}

pub fn format(code: &str) -> Vec<Edit> {
    let tokens = SpannedTokenIterator::new(code);
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

        let mut edits_for_triple = match (one, two, three) {
            (Some(one), Some(two), Some(three)) => {
                // if the token is a {, increase the indent level
                if one.kind == TokenKind::Single(Single::Open(Delim::Brace)) {
                    indent_level += 1;
                }
                // if the token is a }, decrease the indent level
                if one.kind == TokenKind::Single(Single::Close(Delim::Brace)) {
                    indent_level -= 1;
                }

                if one.kind == TokenKind::Whitespace {
                    // first token is whitespace, continue scanning
                    continue;
                } else if two.kind == TokenKind::Whitespace {
                    // whitespace in the middle
                    apply_rule(
                        one,
                        &code[two.span.lo as usize..two.span.hi as usize],
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

    let count_newlines = whitespace.chars().filter(|c| *c == '\n').count();
    let mut new = "\n".repeat(count_newlines);
    new.push_str(&make_indent_string(indent_level));
    new
}

fn apply_rule(
    left: SpannedToken,
    whitespace: &str,
    right: SpannedToken,
    code: &str,
    indent_level: usize,
) -> Vec<Edit> {
    let mut edits = vec![];
    // when we get here, neither left nor right should be whitespace

    // some comment

    // some other comment
    // operation Foo() : Unit {}

    match (left.kind, right.kind) {
        (TokenKind::Comment(_), _) => {
            // fix indentation
            // and fix trailing spaces on the left comment
            let comment_contents = get_token_contents(code, left);
            let new_comment_contents = comment_contents.trim_end();
            if comment_contents != new_comment_contents {
                edits.push(Edit {
                    span: left.span,
                    new_text: new_comment_contents.to_string(),
                });
            }

            // if the middle whitespace contains a new line, we need to
            // fix the indentation
            let new_whitespace = fix_whitespace(whitespace, indent_level);
            if whitespace != new_whitespace {
                edits.push(Edit {
                    span: Span {
                        lo: left.span.hi,
                        hi: right.span.lo,
                    },
                    new_text: new_whitespace.to_string(),
                });
            }
        }
        (TokenKind::Ident, TokenKind::Ident)
        //| (TokenKind::Single(Single::Colon), TokenKind::Ident)
        //| (TokenKind::Ident, TokenKind::Single(_))
        => {
            // Put exactly one space in the middle
            let old_whitespace = whitespace;
            let new_whitespace = " ";
            if old_whitespace != new_whitespace {
                edits.push(Edit {
                    span: Span {
                        lo: left.span.hi,
                        hi: right.span.lo,
                    },
                    new_text: new_whitespace.to_string(),
                });
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

fn get_token_contents(code: &str, left: SpannedToken) -> &str {
    &code[left.span.lo as usize..left.span.hi as usize]
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

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
                    new_text: " ",
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
