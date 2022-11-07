# Oluś


## Compilation process

1. [`lexer`] is a [logo] based tokenizer. It recognizes the minimal set of
   tokens the language has and parses nested quoted strings.
2. [`parse`] is a [rowan] based concrete syntax tree. It recognizes indentation
   blocks, lines and matches parenthesis.
