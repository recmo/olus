//! Wrapper adding indentation awareness to the Logos lexer. Doing this during
//! tokenizing  allows the grammar to be context-free.
use {
    super::{Node, Span},
    logos::{Lexer as LogosLexer, Logos},
};

/// Indentation aware Lexer for the `Node` enum.
pub struct Lexer<'source> {
    logos: LogosLexer<'source, Node>,

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
            if indentation.starts_with(last) {
                self.indentation.push(indentation);

                // Create indentation node for the added part.
                newline_span.end += last.len();
                indentation_span.start += last.len();
                self.buffer.push((Node::Indent, indentation_span));
            } else {
                // Indentation does not have common prefix with last indentation.
                self.buffer
                    .push((Node::ErrorInconsistentIndentation, indentation_span));
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
                self.buffer.push((Node::Dedent, indentation_span));
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

fn indentation(whitespace: &str) -> (&str, &str) {
    let start = whitespace.rfind(is_newline).unwrap_or(0);
    whitespace.split_at(start + 1)
}

/// Newlines according to UAX31-R3a1
const fn is_newline(char: char) -> bool {
    matches!(
        char,
        '\u{000a}' | '\u{000b}' | '\u{000c}' | '\u{000d}' | '\u{0085}' | '\u{2028}' | '\u{2029}'
    )
}
