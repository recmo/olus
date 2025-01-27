/**
 * @file The Oluś Language.
 * @author Remco Bloemen <remco@wicked.ventures>
 * @license MIT
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

module.exports = grammar({
  name: "olus",

  // External scanner is disabled. For basic higlighting we don't need indentation support.
  // externals: ($) => [$.indent, $.dedent, $.newline, $.error_sentinel],

  // Whitespace tokens you want to skip or treat as extras.
  extras: ($) => [/[\p{Pattern_White_Space}]+/u],

  conflicts: ($) => [[$.procedure, $._expression]],

  rules: {
    source_file: ($) => repeat($.statement),

    statement: ($) => seq(choice($.procedure, $.call), $.newline),
    procedure: ($) => seq(repeat($.identifier), $.colon, optional($.call)),
    call: ($) => repeat1($._expression),

    _expression: ($) => choice($.identifier, $.group, $.string, $.number),

    group: ($) => seq($.paren_open, choice($.procedure, $.call), $.paren_close),
    string: ($) => seq($.string_open, repeat(choice($._string_content, $.string)), $.string_close),

    colon: ($) => ":",
    paren_open: ($) => "(",
    paren_close: ($) => ")",
    string_open: ($) => "“",
    string_close: ($) => "”",
    _string_content: ($) => /[^“”]+/,
    number: ($) => /\d+/,
    identifier: ($) => /\p{XID_Start}\p{XID_Continue}*/u,
    newline: ($) => /[\u000a\u000b\u000c\u000d\u0085\u2028\u2029]+/u,
  },
});
