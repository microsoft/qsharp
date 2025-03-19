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

  const QSHARP_NUMBER_MODE: Mode = {
    scope: "number",
    begin: "\\b[\\d_]*\\.?[\\d_]\\b",
  };

  const QSHARP_FN_CALL: Mode = {
    scope: "title",
    match: /\b[a-zA-Z_][a-zA-Z0-9_]*\s*(?=\()/,
  };

  const QSHARP_PUNCTUATION = {
    match: /[;:(){},]/,
    className: "punctuation",
    relevance: 0,
  };

  const QSHARP_INTERP_STRING_MODE: Mode = {
    scope: "string",
    begin: /\$"/,
    end: '"',
    contains: [
      {
        scope: "subst",
        begin: /\{/,
        end: /\}/,
        keywords: QSHARP_KEYWORDS,
        contains: [
          /* add later */
        ],
      },
      hljs.BACKSLASH_ESCAPE,
    ],
  };

  return {
    name: "qsharp",
    case_insensitive: false,
    keywords: QSHARP_KEYWORDS,
    contains: [
      hljs.C_LINE_COMMENT_MODE,
      hljs.QUOTE_STRING_MODE,
      QSHARP_INTERP_STRING_MODE,
      QSHARP_OPERATOR_MODE,
      QSHARP_NUMBER_MODE,
      QSHARP_FN_CALL,
      QSHARP_PUNCTUATION,
    ],
  };
}
