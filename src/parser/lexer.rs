//! Lexer based on Logos. For convenience the `Node` enum contains both tokens
//! and grammar nodes.
use {
    super::Span,
    core::fmt::{self, Display},
    cstree::{RawSyntaxKind, Syntax},
    logos::{Logos, SpannedIter},
    num_enum::{IntoPrimitive, TryFromPrimitive},
};

/// Lexical tokens and grammar nodes.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Logos, IntoPrimitive, TryFromPrimitive,
)]
#[logos(error = Option<Node>)]
#[repr(u8)]
pub enum Node {
    /// White space without line breaks
    /// Ignores newlines from UAX31-R3a1.
    /// See <https://www.unicode.org/reports/tr31/#R3a-1>
    #[regex(
        r"[\p{Pattern_White_Space}--\u000a\u000b\u000c\u000d\u0085\u2028\u2029]+",
        priority = 1
    )]
    Whitespace,

    /// White space with line breaks.
    /// Non-line breaks are caught by `Whitespace`, which has a higher priority.
    #[regex(r"[\p{Pattern_White_Space}]+", priority = 0)]
    Newline,

    /// Colon
    #[token(":")]
    Colon,
    /// Opening Parenthesis
    #[token("(")]
    ParenOpen,
    /// Closing Parenthesis
    #[token(")")]
    ParenClose,

    /// Identifiers and symbols
    /// See <https://www.unicode.org/reports/tr31>
    /// See <https://util.unicode.org/UnicodeJsps/list-unicodeset.jsp?a=[:Pattern_Syntax=Yes:]>
    #[regex(r"\p{XID_Start}\p{XID_Continue}*|\p{Pattern_Syntax}", priority = 0)]
    Identifier,

    /// Strings litteral.
    /// Strings are delimited by mirrored assymetric double qoutes: “ and ”
    #[token("“", string)]
    String,

    /// Number literal
    #[regex(r"[0-9]+")]
    Number,

    // Virtual tokens to make the grammer context free.
    /// Increased indentation.
    Indent,
    /// Decreased indentation.
    Dedent,

    // Grammar nodes
    /// The root of the syntax tree
    Root,
    /// A line defining a procedure
    Def,
    /// A fragment declaring a procedure
    Proc,
    /// A fragment specifying a call
    Call,
    /// An indented block of code
    Block,
    /// A parenthesized group
    Group,

    // Errors
    /// Unknown token.
    ErrorUnknownToken,
    /// Unterminated string literal.
    ErrorUnterminatedString,
    /// Inconsistent indentation.
    ErrorInconsistentIndentation,
    /// Invalid token kind.
    ErrorInvalidTokenKind,
}

impl Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Syntax definition for CSTree
impl Syntax for Node {
    fn from_raw(raw: RawSyntaxKind) -> Self {
        u8::try_from(raw.0)
            .ok()
            .and_then(|n| n.try_into().ok())
            .unwrap_or(Node::ErrorInvalidTokenKind)
    }

    fn into_raw(self) -> RawSyntaxKind {
        RawSyntaxKind(u8::from(self).into())
    }

    fn static_text(self) -> Option<&'static str> {
        None
    }
}

/// Indentation aware Lexer for the `Node` enum.
pub struct Lexer<'source> {
    logos: logos::Lexer<'source, Node>,

    /// Buffer of tokens that have been read ahead
    buffer: Vec<(Node, Span)>,

    /// Indentation stack
    /// The first element is always an empty string, which is the indentation of
    /// the root node. The last element is the current indentation.
    /// Every element is a prefix of the next element.
    indentation: Vec<&'source str>,
}

impl<'source> Lexer<'source> {
    pub fn new(source: &'source str) -> Self {
        Self {
            logos:       Node::lexer(source),
            indentation: vec![&source[0..0]],
            buffer:      Vec::new(),
        }
    }
}

impl Iterator for Lexer<'_> {
    type Item = (Node, Span);

    fn next(&mut self) -> Option<Self::Item> {
        // Clear buffer first
        if let Some(token) = self.buffer.pop() {
            return Some(token);
        }

        // Put errors in the token stream so the parser can recover.
        let token = self
            .logos
            .next()?
            .unwrap_or_else(|err| err.unwrap_or(Node::ErrorUnknownToken));
        let span = self.logos.span().into();

        // Pass through non indentation tokens
        if token != Node::Newline {
            return Some((token, span));
        }

        // Ceck indentation and emit any `Node::Dedent` before and any
        // `Node::Indent` after.
        let (newline, indentation) = indentation(self.logos.slice());
        let last = self.indentation.last().copied().unwrap_or("");
        let mut newline_span = Span::new(span.start, span.start + newline.len());
        let mut indentation_span = Span::new(span.start + newline.len(), span.end);

        // TODO: Keep common identation with the newline and assign Dedents an empty
        // span.

        if indentation.len() > last.len() {
            if !indentation.starts_with(last) {
                // Indentation does not have common prefix with last indentation.
                self.buffer
                    .push((Node::ErrorInconsistentIndentation, indentation_span));
            } else {
                self.indentation.push(indentation);

                // Create indentation node for the added part.
                newline_span.end += last.len();
                indentation_span.start += last.len();
                self.buffer.push((Node::Indent, indentation_span));
            }
        } else if let Some(indents) = self
            .indentation
            .iter()
            .copied()
            .rposition(|i| i == indentation)
        {
            let dedents = self.indentation.len() - indents - 1;
            self.indentation.truncate(indents + 1);
            // Dedent span is after the newline.
            newline_span.end = span.end;
            indentation_span.start = span.end;
            for _ in 0..dedents {
                self.buffer.push((Node::Dedent, indentation_span.clone()));
            }
        } else {
            // Indentation does not match any previous indentation.
            self.buffer
                .push((Node::ErrorInconsistentIndentation, indentation_span));
        }

        // Emit the newline token before the indent/dedent tokens.
        Some((Node::Newline, newline_span))
    }
}

/// Matches a string literal by bumping the span.
fn string(lexer: &mut logos::Lexer<Node>) -> Result<(), Option<Node>> {
    #[derive(Logos)]
    #[logos(skip "[^“”]+")]
    enum Token {
        #[token("“")]
        Open,
        #[token("”")]
        Close,
    }
    let mut inner = Token::lexer(lexer.remainder());
    let mut nesting = 1;
    for (token, span) in inner.spanned() {
        match token {
            Ok(Token::Open) => nesting += 1,
            Ok(Token::Close) => {
                nesting -= 1;
                if nesting == 0 {
                    lexer.bump(span.end);
                    return Ok(());
                }
            }
            Err(()) => unreachable!("String parser is infallible."),
        }
    }

    // Unclosed string literal
    Err(Some(Node::ErrorUnterminatedString))
}

fn indentation(whitespace: &str) -> (&str, &str) {
    let start = whitespace.rfind(is_newline).unwrap_or(0);
    whitespace.split_at(start + 1)
}

const fn is_newline(char: char) -> bool {
    matches!(
        char,
        '\u{000a}' | '\u{000b}' | '\u{000c}' | '\u{000d}' | '\u{0085}' | '\u{2028}' | '\u{2029}'
    )
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_string() {
        let source = "“Hello, “nested” world!”";
        let mut lexer = Lexer::<Node>::new(source);
        assert_eq!(lexer.next(), Some(Ok(Node::String)));
        assert_eq!(lexer.span(), 0..32);
        assert_eq!(lexer.remainder(), "");
    }
}
