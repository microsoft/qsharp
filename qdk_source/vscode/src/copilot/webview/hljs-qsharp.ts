// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { HLJSApi, Mode, type Language } from "highlight.js";

// NOTE: This has a bunch of differences from how we structure the language in <vscode/syntaxes/qsharp.tmLanguage.json>
// We should align with that as much as possible eventually.

const KEYWORDS_GENERAL = [
  "use",
  "borrow",
  "mutable",
  "let",
  "set",
  "if",
  "elif",
  "else",
  "repeat",
  "until",
  "fixup",
  "for",
  "in",
  "while",
  "return",
  "fail",
  "within",
  "apply",
];

const KEYWORDS_DECL = [
  "namespace",
  "open",
  "import",
  "export",
  "as",
  "internal",
  "newtype",
  "struct",
  "operation",
  "function",
  "new",
  "body",
  "adjoint",
  "Adjoint",
  "controlled",
  "Controlled",
  "self",
  "auto",
  "distribute",
  "invert",
  "intrinsic",
];

const KEYWORDS_TYPE = [
  "Int",
  "BigInt",
  "Double",
  "Bool",
  "Qubit",
  "Pauli",
  "Result",
  "Range",
  "String",
  "Unit",
  "Ctl",
  "Adj",
  "is",
];

const KEYWORDS_CONSTANTS = [
  "true",
  "false",
  "PauliI",
  "PauliX",
  "PauliY",
  "PauliZ",
  "One",
  "Zero",
];

export default function (hljs: HLJSApi): Language {
  const QSHARP_KEYWORDS = {
    keyword: KEYWORDS_GENERAL,
    declare: KEYWORDS_DECL,
    literal: KEYWORDS_CONSTANTS,
    type: KEYWORDS_TYPE,
  };

  const QSHARP_OPERATOR_MODE: Mode = {
    scope: "operator",
    match:
      "\\b(not|and|or)\\b|\\b(w/)|(=)|(!)|(<)|(>)|(\\+)|(-)|(\\*)|(\\/)|(\\^)|(%)|(\\|)|(\\&\\&\\&)|(\\~\\~\\~)|(\\.\\.\\.)|(\\.\\.)|(\\?)",
  };

  const decimalDigits = "[0-9](_?[0-9])*";
  const frac = `\\.(${decimalDigits})`;
  const decimalInteger = `0|[1-9](_?[0-9])*|0[0-7]*[89][0-9]*`;
  const NUMBER = {
    className: "number",
    variants: [
      // DecimalLiteral
      {
        begin:
          `(\\b(${decimalInteger})((${frac})|\\.)?|(${frac}))` +
          `[eE][+-]?(${decimalDigits})\\b`,
      },
      { begin: `\\b(${decimalInteger})\\b((${frac})\\b|\\.)?|(${frac})\\b` },

      // DecimalBigIntegerLiteral
      { begin: `\\b(0|[1-9](_?[0-9])*)L\\b` },

      // NonDecimalIntegerLiteral
      { begin: "\\b0[xX][0-9a-fA-F](_?[0-9a-fA-F])*L?\\b" },
      { begin: "\\b0[bB][0-1](_?[0-1])*L?\\b" },
      { begin: "\\b0[oO][0-7](_?[0-7])*L?\\b" },
    ],
  };

  const QSHARP_CALL_MODE: Mode = {
    scope: "title.function",
    match: /\b[a-zA-Z_][a-zA-Z0-9_]*\s*(?=\()/,
  };

  const QSHARP_PUNCTUATION_MODE = {
    match: /[;:(){},]/,
    className: "punctuation",
  };

  const SUBST: Mode = {
    className: "subst",
    begin: /\{/,
    end: /\}/,
    keywords: QSHARP_KEYWORDS,
    contains: [
      hljs.QUOTE_STRING_MODE,
      QSHARP_OPERATOR_MODE,
      NUMBER,
      QSHARP_CALL_MODE,
    ],
  };

  const QSHARP_INTERP_STRING_MODE: Mode = {
    scope: "string",
    begin: /\$"/,
    end: /"/,
    contains: [hljs.BACKSLASH_ESCAPE, SUBST],
  };
  SUBST.contains?.push(QSHARP_INTERP_STRING_MODE);

  const QSHARP_CONTAINS = [
    hljs.C_LINE_COMMENT_MODE,
    hljs.QUOTE_STRING_MODE,
    QSHARP_INTERP_STRING_MODE,
    QSHARP_OPERATOR_MODE,
    NUMBER,
    QSHARP_CALL_MODE,
    QSHARP_PUNCTUATION_MODE,
  ];

  return {
    name: "qsharp",
    case_insensitive: false,
    keywords: QSHARP_KEYWORDS,
    contains: QSHARP_CONTAINS,
  };
}
