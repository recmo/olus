/**
 * @file The Oluś Language.
 * @author Remco Bloemen <remco@wicked.ventures>
 * @license MIT
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

module.exports = grammar({
  name: "olus",

  // Whitespace tokens you want to skip or treat as extras.
  extras: ($) => [/\p{White_Space}/u],

  rules: {
    source_file: ($) => repeat($._expression),

    _expression: ($) => choice($.identifier, $.colon, $.group, $.string, $.number),

    identifier: ($) => /\p{XID_Start}\p{XID_Continue}*/u,
    colon: ($) => ":",
    group: ($) => seq("(", repeat($._expression), ")"),
    string: ($) => seq("“", repeat(choice(/[^“”]/, $.string)), "”"),
    number: ($) => /\d+/,
  },
});
