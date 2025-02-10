// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// Currently Highlight.js for Q# points to https://github.com/fedonman/highlightjs-qsharp/blob/main/src/languages/qsharp.js

import { HLJSApi, Mode, type Language } from "highlight.js";

export default function (hljs: HLJSApi): Language {
  const QSHARP_KEYWORDS = {
    keyword: [
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
      "Adjoint",
      "adjoint",
      "controlled",
      "Controlled",
      "self",
      "auto",
      "distribute",
      "invert",
      "intrinsic",
      "is",
    ],
    literal: [
      "true",
      "false",
      "Zero",
      "One",
      "PauliI",
      "PauliX",
      "PauliY",
      "PauliZ",
      "Ctl",
      "Adj",
    ],
    type: [
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
    ],
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

  const QSHARP_INTERP_STRING_MODE: Mode = {
    scope: "string",
    begin: /\$"/,
    end: '"',
    illegal: /\n/,
    contains: [
      {
        scope: "subst",
        begin: /\{/,
        end: /\}/,
        keywords: QSHARP_KEYWORDS,
        contains: [
          hljs.C_LINE_COMMENT_MODE,
          "self",
          hljs.QUOTE_STRING_MODE,
          QSHARP_OPERATOR_MODE,
          QSHARP_NUMBER_MODE,
        ],
      },
      hljs.BACKSLASH_ESCAPE,
    ],
  };
  return {
    name: "Q#",
    case_insensitive: true,
    keywords: QSHARP_KEYWORDS,
    contains: [
      hljs.C_LINE_COMMENT_MODE,
      QSHARP_INTERP_STRING_MODE,
      hljs.QUOTE_STRING_MODE,
      QSHARP_OPERATOR_MODE,
      QSHARP_NUMBER_MODE,
    ],
  };
}
