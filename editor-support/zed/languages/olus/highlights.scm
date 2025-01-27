; Atoms and nested strings
(colon) @operator
(number) @number
(paren_open) @punctuation.bracket
(paren_close) @punctuation.bracket
(string
    (string_open) @string.special.symbol
    (string_close) @string.special.symbol
) @string

; Statements
(statement (procedure .
    (identifier) @function
    (identifier)* @variable.special
))
(group (procedure (identifier) @variable.special)) @function

; Calls
(call (identifier) @variable)

; Calls to soft conventions
(statement (call
    . (identifier) @_name (#any-of? @_name "title" "section")
    . (string) @title
))
(statement (call
    . (identifier) @_name (#any-of? @_name "doc" "description")
    . (string) @comment.doc
))
(statement (call . (string) @comment))
