// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// Currently Highlight.js for Q# points to https://github.com/fedonman/highlightjs-qsharp/blob/main/src/languages/qsharp.js

export default function (hljs: any) {
  return {
    name: "Q#",
    case_insensitive: true,
    keywords: {
      keyword:
        "namespace open operation function body adjoint controlled self auto if elif else return fail for in new use borrow repeat until fixup",
      literal: "true false Zero One PauliI PauliX PauliY PauliZ Adj Ctl",
      built_in: "Unit Int BigInt Double Bool String Qubit Result Pauli Range",
    },
    contains: [
      hljs.C_LINE_COMMENT_MODE,
      hljs.C_BLOCK_COMMENT_MODE,
      {
        className: "string",
        begin: /\$"/,
        end: '"',
        illegal: /\n/,
        contains: [{ begin: /\{\{/ }, { begin: /\}\}/ }, hljs.BACKSLASH_ESCAPE],
      },
      {
        className: "number",
        variants: [
          { begin: "(-)?(0b|0o|0x)?([\\d]+)(L)?" },
          { begin: "(-)?(0b|0o|0x)?([\\d]+)(.[\\d]+)?" },
          { begin: "(-)?([\\d]+(.[\\d]*)?([eE][-+]?[\\d]+)?)" },
        ],
      },
    ],
  };
}
