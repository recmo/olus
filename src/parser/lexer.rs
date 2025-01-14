//! Lexer based on Logos. For convenience the `Node` enum contains both tokens
//! and grammar nodes.
use {
    core::fmt::{self, Display},
    cstree::{RawSyntaxKind, Syntax},
    logos::Logos,
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
        write!(f, "{self:?}")
    }
}

/// Syntax definition for `CSTree`
impl Syntax for Node {
    fn from_raw(raw: RawSyntaxKind) -> Self {
        u8::try_from(raw.0)
            .ok()
            .and_then(|n| n.try_into().ok())
            .unwrap_or(Self::ErrorInvalidTokenKind)
    }

    fn into_raw(self) -> RawSyntaxKind {
        RawSyntaxKind(u8::from(self).into())
    }

    fn static_text(self) -> Option<&'static str> {
        match self {
            Self::Colon => ":",
            Self::ParenOpen => "(",
            Self::ParenClose => ")",
            _ => {
                return None;
            }
        }
        .into()
    }
}

impl Node {
    #[must_use]
    pub const fn is_trivia(&self) -> bool {
        matches!(self, Self::Whitespace | Self::Newline)
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
    let inner = Token::lexer(lexer.remainder());
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
