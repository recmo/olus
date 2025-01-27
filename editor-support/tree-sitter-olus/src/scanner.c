// https://gitlab.com/viktomas/tree-sitter-logseq/-/blob/5ab1cac03f2ab572f586f5d59ae7818aa78219c1/src/scanner.c
#include "tree_sitter/parser.h"
#include "tree_sitter/alloc.h"
#include "tree_sitter/array.h"

enum TokenType {
  INDENT,
  DEDENT,
  NEWLINE,
  ERROR_SENTINEL // Available when the parser is error correcting.
};

void* tree_sitter_olus_external_scanner_create() {
    return ts_calloc(1, sizeof(Array(int)));
}

void tree_sitter_olus_external_scanner_destroy(void *payload) {
    ts_free(payload);
}

unsigned tree_sitter_olus_external_scanner_serialize(
  void *payload,
  char *buffer
) {
  // ...
}

void tree_sitter_olus_external_scanner_deserialize(
  void *payload,
  const char *buffer,
  unsigned length
) {
  // ...
}

bool tree_sitter_olus_external_scanner_scan(
  void *payload,
  TSLexer *lexer,
  const bool *valid_symbols
) {
    // Nope out of error recovery.
    if (valid_symbols[ERROR_SENTINEL]) {
      return false;
    }

    Array(int) *stack = payload;
    if (valid_symbols[INDENT]) {
      array_push(stack, lexer->get_column(lexer));
      lexer->result_symbol = INDENT;
      return true;
    }
    if (valid_symbols[DEDENT]) {
      array_pop(stack); // this returns the popped element by value, but we don't need it
      lexer->result_symbol = DEDENT;
      return true;
    }

    return false;
}
