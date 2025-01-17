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
#[logos(error = Option<Kind>)]
#[repr(u8)]
pub enum Kind {
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
    /// Strings are delimited by mirrored assymetric double qoutes: “ and ”.
    /// Nested strings are recognized and considered a single string.
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
    /// An indented block of code
    Block,
    /// A fragment declaring a procedure
    Proc,
    /// A fragment specifying a call
    Call,

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

impl Kind {
    /// Nodes that are semantic after parsing the concrete syntax.
    #[must_use]
    pub const fn is_syntax(&self) -> bool {
        matches!(
            self,
            Self::Root
                | Self::Block
                | Self::Proc
                | Self::Call
                | Self::Identifier
                | Self::String
                | Self::Number
        )
    }
}

impl Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

/// Syntax definition for [cstree]
impl Syntax for Kind {
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
            Self::Dedent => "",
            _ => {
                return None;
            }
        }
        .into()
    }
}

/// Matches a string literal.
fn string(lexer: &mut logos::Lexer<Kind>) -> Result<(), Option<Kind>> {
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
    Err(Some(Kind::ErrorUnterminatedString))
}
