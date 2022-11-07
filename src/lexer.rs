use logos::{Lexer, Logos};
use strum::{EnumCount, FromRepr};

#[derive(Logos, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, FromRepr, EnumCount)]
#[logos(subpattern newline=r"[\u000a\u000b\u000c\u000d\u0085\u2028\u2029]")]
pub enum Token {
    // White space excluding line breaks
    // See <https://www.unicode.org/reports/tr14>
    // See <https://util.unicode.org/UnicodeJsps/list-unicodeset.jsp?a=[:Pattern_White_Space=Yes:]>
    #[regex(r"[\p{Pattern_White_Space}--(?&newline)]+")]
    Whitespace,

    // Line breaks
    #[regex(r"[\p{Pattern_White_Space}&&(?&newline)]+")]
    Newline,

    #[token(":")]
    Colon,
    #[token("(")]
    ParenOpen,
    #[token(")")]
    ParenClose,

    // Identifiers and symbols
    // See <https://www.unicode.org/reports/tr31>
    // See <https://util.unicode.org/UnicodeJsps/list-unicodeset.jsp?a=[:Pattern_Syntax=Yes:]>
    #[regex(r"\p{XID_Start}\p{XID_Continue}*|\p{Pattern_Syntax}")]
    Identifier,

    #[token("“", string)]
    String,

    #[regex(r"[0-9]+")]
    // TODO: https://github.com/maciejhirsz/logos/issues/133#issuecomment-687281059
    Number,

    #[token("”")]
    #[error]
    Error,
}

impl Default for Token {
    fn default() -> Self {
        Self::Error
    }
}

/// Matches a string literal by bumping the span.
fn string(lexer: &mut Lexer<Token>) -> bool {
    #[derive(Logos)]
    enum Token {
        #[token("“")]
        Open,
        #[token("”")]
        Close,
        #[regex(r"[^“”]+")]
        #[error]
        Other,
    }
    let mut inner = Lexer::<Token>::new(lexer.remainder());
    let mut nesting = 1;
    while let Some(token) = inner.next() {
        match token {
            Token::Open => nesting += 1,
            Token::Close => {
                nesting -= 1;
                if nesting == 0 {
                    lexer.bump(inner.span().end);
                    return true;
                }
            }
            Token::Other => {}
        }
    }
    false
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_string() {
        let source = "“Hello, “nested” world!”";
        let mut lexer = Lexer::<Token>::new(source);
        assert_eq!(lexer.next(), Some(Token::String));
        assert_eq!(lexer.span(), 0..32);
        assert_eq!(lexer.remainder(), "");
    }
}
