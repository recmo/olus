use super::Token;
use strum::{EnumCount, EnumDiscriminants, FromRepr};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, EnumCount, FromRepr, EnumDiscriminants,
)]
pub enum SyntaxKind {
    // Re-use lexer tokens
    Token(Token),

    // Composite nodes
    Root,  // The root of the syntax tree
    Block, // Indented block
    Line,  // A line of code
    Group, // Parenthesized group
}

impl From<SyntaxKind> for rowan::SyntaxKind {
    fn from(kind: SyntaxKind) -> Self {
        Self(match kind {
            SyntaxKind::Token(token) => token as u16,
            other => Token::COUNT as u16 + SyntaxKindDiscriminants::from(other) as u16,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Language;

impl rowan::Language for Language {
    type Kind = SyntaxKind;

    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        let raw = raw.0 as usize;
        if raw < Token::COUNT {
            SyntaxKind::Token(Token::from_repr(raw).unwrap())
        } else {
            SyntaxKind::from_repr(raw - Token::COUNT)
                .unwrap_or_else(|| SyntaxKind::Token(Token::Error))
        }
    }

    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        kind.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rowan::{Language as _, SyntaxKind as SK};

    #[test]
    fn fits_u16() {
        assert_eq!(Token::COUNT, 9);
        assert_eq!(SyntaxKind::COUNT, 5);
        assert!(Token::COUNT + SyntaxKind::COUNT < u16::MAX as usize);
    }

    #[test]
    fn test_from() {
        assert_eq!(SK::from(SyntaxKind::Token(Token::Whitespace)), SK(0));
        assert_eq!(SK::from(SyntaxKind::Token(Token::Error)), SK(8));
        assert_eq!(SK::from(SyntaxKind::Root), SK(10));
        assert_eq!(SK::from(SyntaxKind::Group), SK(13));
    }

    #[test]
    fn test_to() {
        assert_eq!(
            Language::kind_from_raw(SK(0)),
            SyntaxKind::Token(Token::Whitespace)
        );
        assert_eq!(
            Language::kind_from_raw(SK(8)),
            SyntaxKind::Token(Token::Error)
        );
        assert_eq!(Language::kind_from_raw(SK(10)), SyntaxKind::Root);
        assert_eq!(Language::kind_from_raw(SK(13)), SyntaxKind::Group);
    }

    #[test]
    fn test_to_invalid() {
        assert_eq!(
            Language::kind_from_raw(SK(9)),
            SyntaxKind::Token(Token::Error)
        );
        assert_eq!(
            Language::kind_from_raw(SK(14)),
            SyntaxKind::Token(Token::Error)
        );
    }
}
