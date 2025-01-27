/**
 * @file The Oluś Language.
 * @author Remco Bloemen <remco@wicked.ventures>
 * @license MIT
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

module.exports = grammar({
  name: "olus",

  externals: ($) => [$.indent, $.dedent, $.newline, $.error_sentinel],

  // Whitespace tokens you want to skip or treat as extras.
  extras: ($) => [/\p{White_Space}/u],

  rules: {
    source_file: ($) => repeat($.statement),

    statement: ($) =>
      seq(repeat1($._expression), optional(seq($.colon, repeat($._expression))), $.newline),
    block: ($) => seq($.indent, repeat($.statement), $.dedent),

    _expression: ($) => choice($.identifier, $.group, $.string, $.number),

    group: ($) =>
      seq(
        $.paren_open,
        repeat($._expression),
        optional(seq($.colon, repeat($._expression))),
        $.paren_close,
      ),
    string: ($) => seq($.string_open, repeat(choice($.string_content, $.string)), $.string_close),

    colon: ($) => ":",
    paren_open: ($) => "(",
    paren_close: ($) => ")",
    string_open: ($) => "“",
    string_close: ($) => "”",
    string_content: ($) => /[^“”]+/,
    number: ($) => /\d+/,
    identifier: ($) => /\p{XID_Start}\p{XID_Continue}*/u,
  },
});
