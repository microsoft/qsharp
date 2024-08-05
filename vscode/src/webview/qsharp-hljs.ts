/*
Language: Q#
Website: https://docs.microsoft.com/en-us/dotnet/qsharp/
Category: common
*/

export default function (hljs: any) {
  const BUILT_IN_KEYWORDS = [
    "Unit",
    "Int",
    "BigInt",
    "Double",
    "Bool",
    "String",
    "Qubit",
    "Result",
    "Pauli",
    "Range",
  ];
  const LITERAL_KEYWORDS = [
    "true",
    "false",
    "Zero",
    "One",
    "PauliI",
    "PauliX",
    "PauliY",
    "PauliZ",
    "Adj",
    "Ctl",
  ];
  const NORMAL_KEYWORDS = [
    "operation",
    "function",
    "is",
    "namespace",
    "open",
    "as",
    "newtype",
    "let",
    "mutable",
    "set",
    "body",
    "adjoint",
    "controlled",
    "self",
    "auto",
    "if",
    "elif",
    "else",
    "return",
    "fail",
    "for",
    "in",
    "new",
    "use",
    "borrow",
    "repeat",
    "until",
    "fixup",
    "while",
    "within",
    "apply",
  ];

  const KEYWORDS = {
    keyword: NORMAL_KEYWORDS,
    built_in: BUILT_IN_KEYWORDS,
    literal: LITERAL_KEYWORDS,
  };
  const TITLE_MODE = hljs.inherit(hljs.TITLE_MODE, {
    begin: "[a-zA-Z](\\.?\\w)*",
  });
  const NUMBERS = {
    className: "number",
    variants: [
      { begin: "(-)?(0b|0o|0x)?([\\d]+)(L)?" },
      { begin: "(-)?(0b|0o|0x)?([\\d]+)(.[\\d]+)?" },
      { begin: "(-)?([\\d]+(.[\\d]*)?([eE][-+]?[\\d]+)?)" },
    ],
    relevance: 0,
  };
  const SUBST: any = {
    className: "subst",
    begin: /\{/,
    end: /\}/,
    keywords: KEYWORDS,
  };
  const SUBST_NO_LF = hljs.inherit(SUBST, { illegal: /\n/ });
  const INTERPOLATED_STRING = {
    className: "string",
    begin: /\$"/,
    end: '"',
    illegal: /\n/,
    contains: [
      { begin: /\{\{/ },
      { begin: /\}\}/ },
      hljs.BACKSLASH_ESCAPE,
      SUBST_NO_LF,
    ],
  };
  SUBST.contains = [
    INTERPOLATED_STRING,
    hljs.APOS_STRING_MODE,
    hljs.QUOTE_STRING_MODE,
    NUMBERS,
    hljs.C_BLOCK_COMMENT_MODE,
  ];
  SUBST_NO_LF.contains = [
    INTERPOLATED_STRING,
    hljs.APOS_STRING_MODE,
    hljs.QUOTE_STRING_MODE,
    NUMBERS,
    hljs.inherit(hljs.C_BLOCK_COMMENT_MODE, { illegal: /\n/ }),
  ];
  const STRING = {
    variants: [
      INTERPOLATED_STRING,
      hljs.APOS_STRING_MODE,
      hljs.QUOTE_STRING_MODE,
    ],
  };

  const GENERIC_MODIFIER = {
    begin: "<",
    end: ">",
    contains: [{ beginKeywords: "in out" }, TITLE_MODE],
  };
  const TYPE_IDENT_RE =
    hljs.IDENT_RE +
    "(<" +
    hljs.IDENT_RE +
    "(\\s*,\\s*" +
    hljs.IDENT_RE +
    ")*>)?(\\[\\])?";
  const AT_IDENTIFIER = {
    // prevents expressions like `@class` from incorrect flagging
    // `class` as a keyword
    begin: "@" + hljs.IDENT_RE,
    relevance: 0,
  };

  return {
    name: "Q#",
    aliases: ["q#", "qs", "qsharp"],
    keywords: KEYWORDS,
    illegal: /::/,
    contains: [
      hljs.COMMENT("///", "$", {
        returnBegin: true,
        contains: [
          {
            className: "doctag",
            variants: [
              {
                begin: "///",
                relevance: 0,
              },
              {
                begin: "<!--|-->",
              },
              {
                begin: "</?",
                end: ">",
              },
            ],
          },
        ],
      }),
      hljs.C_LINE_COMMENT_MODE,
      hljs.C_BLOCK_COMMENT_MODE,
      {
        className: "meta",
        begin: "#",
        end: "$",
        keywords: {
          "meta-keyword":
            "if else elif endif define undef warning error line region endregion pragma checksum",
        },
      },
      STRING,
      NUMBERS,
      {
        beginKeywords: "namespace",
        relevance: 0,
        end: /[{;=]/,
        illegal: /[^\s:]/,
        contains: [
          TITLE_MODE,
          hljs.C_LINE_COMMENT_MODE,
          hljs.C_BLOCK_COMMENT_MODE,
        ],
      },
      {
        // [Attributes("")]
        className: "meta",
        begin: "^\\s*\\@",
        excludeBegin: true,
        end: "\\(\\)",
        excludeEnd: true,
        contains: [{ className: "meta-string", begin: /"/, end: /"/ }],
      },
      {
        // Expression keywords prevent 'keyword Name(...)' from being
        // recognized as a function definition
        beginKeywords: "new return throw await else",
        relevance: 0,
      },
      {
        className: "function",
        begin:
          "(" + TYPE_IDENT_RE + "\\s+)+" + hljs.IDENT_RE + "\\s*(<.+>\\s*)?\\(",
        returnBegin: true,
        end: /\s*[{;=]/,
        excludeEnd: true,
        keywords: KEYWORDS,
        contains: [
          // prevents these from being highlighted `title`
          {
            beginKeywords: "", // FUNCTION_MODIFIERS.join(" "),
            relevance: 0,
          },
          {
            begin: hljs.IDENT_RE + "\\s*(<.+>\\s*)?\\(",
            returnBegin: true,
            contains: [hljs.TITLE_MODE, GENERIC_MODIFIER],
            relevance: 0,
          },
          {
            className: "params",
            begin: /\(/,
            end: /\)/,
            excludeBegin: true,
            excludeEnd: true,
            keywords: KEYWORDS,
            relevance: 0,
            contains: [STRING, NUMBERS, hljs.C_BLOCK_COMMENT_MODE],
          },
          hljs.C_LINE_COMMENT_MODE,
          hljs.C_BLOCK_COMMENT_MODE,
        ],
      },
      AT_IDENTIFIER,
    ],
  };
}
