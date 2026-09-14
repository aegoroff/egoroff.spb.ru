/*
Language: Zig
Description: Zig is a general-purpose programming language and toolchain for
maintaining robust, optimal and reusable software.
Website: https://ziglang.org

Keywords are taken verbatim from `std.zig.Token.keywords` and primitive type
names from `std.zig.primitives.names` of Zig 0.16.0, except for `anytype` and
`anyframe`, which are formally keywords but are highlighted as types here to
keep them visually consistent with the rest of the `any*` family.
*/
import type { HLJSApi, Language, Mode } from "highlight.js";

export default function zig(hljs: HLJSApi): Language {
  const KEYWORDS = [
    "addrspace",
    "align",
    "allowzero",
    "and",
    "asm",
    "break",
    "callconv",
    "catch",
    "comptime",
    "const",
    "continue",
    "defer",
    "else",
    "enum",
    "errdefer",
    "error",
    "export",
    "extern",
    "fn",
    "for",
    "if",
    "inline",
    "linksection",
    "noalias",
    "noinline",
    "nosuspend",
    "opaque",
    "or",
    "orelse",
    "packed",
    "pub",
    "resume",
    "return",
    "struct",
    "suspend",
    "switch",
    "test",
    "threadlocal",
    "try",
    "union",
    "unreachable",
    "var",
    "volatile",
    "while",
  ];

  const LITERALS: Mode = {
    className: "literal",
    match: /\b(?:true|false|null|undefined)\b/,
  };

  const TYPES: Mode = {
    className: "type",
    variants: [
      {
        // Sized integers and floats
        match: /\b(?:[iu]\d+|f16|f32|f64|f80|f128)\b/,
        relevance: 2,
      },
      {
        // C ABI types
        match:
          /\b(?:c_char|c_short|c_ushort|c_int|c_uint|c_long|c_ulong|c_longlong|c_ulonglong|c_longdouble)\b/,
        relevance: 1,
      },
      {
        // Everything else primitive, plus the `any*` family
        match:
          /\b(?:isize|usize|comptime_int|comptime_float|bool|void|noreturn|type|anyerror|anyopaque|anyframe|anytype)\b/,
        relevance: 0,
      },
    ],
  };

  const BUILT_IN: Mode = {
    className: "built_in",
    variants: [
      // `@import(` is the single most Zig-specific construct there is, so it
      // carries the relevance that keeps auto-detection working.
      { match: /@(?:import|cImport)(?=\()/, relevance: 10 },
      { match: /@[A-Za-z_]\w*/, relevance: 0 },
    ],
  };

  // `@"weird name"` is an identifier, not a builtin call and not a string.
  const QUOTED_IDENTIFIER: Mode = {
    className: "title",
    match: /@"[^"]*"/,
  };

  const COMMENT_TAGS: Mode = {
    className: "doctag",
    match: /\b(?:TODO|FIXME|XXX|NOTE)\b:?/,
    relevance: 0,
  };

  // `//`, `///` and `//!` all render the same, so one mode covers them.
  const COMMENTS: Mode = hljs.COMMENT(/\/\//, /$/, {
    contains: [COMMENT_TAGS],
  });

  const ESCAPES: Mode = {
    className: "string",
    variants: [
      { match: /\\(?:[nrt'"\\]|x[0-9a-fA-F]{2}|u\{[0-9a-fA-F]+\})/ },
      // Invalid escape sequence
      { match: /\\./ },
    ],
    relevance: 0,
  };

  const STRINGS: Mode = {
    className: "string",
    variants: [
      { begin: /"/, end: /"/, contains: [ESCAPES] },
      { begin: /'/, end: /'/, contains: [ESCAPES] },
      // Multiline strings take no escape sequences at all
      { begin: /\\\\/, end: /$/ },
    ],
    relevance: 0,
  };

  const OPERATORS: Mode = {
    className: "operator",
    variants: [
      // C pointer
      { match: /\[\*c\]/ },
      // Array concatenation and repetition
      { match: /\+\+|\*\*/ },
      // Comparison
      { match: /==|!=|<=|>=/ },
      // Arithmetic, including wrapping and saturating forms
      { match: /(?:\+[%|]?|-[%|]?|\*[%|]?|\/|%)=?/ },
      // Bitwise and logical
      { match: /(?:<<[%|]?|>>|!|&|\^|\|)=?/ },
    ],
    relevance: 0,
  };

  const FUNCTION: Mode = {
    match: [/\bfn\b/, /\s+/, /[A-Za-z_]\w*/],
    scope: { 1: "keyword", 3: "title.function" },
    relevance: 0,
  };

  const NUMBERS: Mode = {
    className: "number",
    variants: [
      // Hexadecimal float
      {
        match: /\b0x[0-9a-fA-F][0-9a-fA-F_]*(?:\.[0-9a-fA-F][0-9a-fA-F_]*)?[pP][+-]?\d[\d_]*/,
      },
      // Hexadecimal integer
      { match: /\b0x[0-9a-fA-F][0-9a-fA-F_]*\b/ },
      // Octal
      { match: /\b0o[0-7][0-7_]*\b/ },
      // Binary
      { match: /\b0b[01][01_]*\b/ },
      // Decimal float
      {
        match: /\b\d[\d_]*(?:\.\d[\d_]*)?(?:[eE][+-]?\d[\d_]*)?\b/,
      },
    ],
    relevance: 0,
  };

  return {
    name: "Zig",
    aliases: ["zig"],
    keywords: { keyword: KEYWORDS },
    contains: [
      COMMENTS,
      LITERALS,
      QUOTED_IDENTIFIER,
      BUILT_IN,
      STRINGS,
      TYPES,
      FUNCTION,
      NUMBERS,
      OPERATORS,
    ],
  };
}
