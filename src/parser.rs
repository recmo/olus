use crate::lexer::Token;
use core::iter::Peekable;
use logos::Lexer;
use rowan::{GreenNode, GreenNodeBuilder};
use strum::{EnumCount, EnumDiscriminants, FromRepr};

// https://github.com/rust-analyzer/rowan/blob/master/examples/s_expressions.rs

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
pub struct Lang;
impl rowan::Language for Lang {
    type Kind = SyntaxKind;

    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        let raw = raw.0 as usize;
        if raw < Token::COUNT {
            SyntaxKind::Token(Token::from_repr(raw).unwrap())
        } else {
            SyntaxKind::from_repr(raw - Token::COUNT).unwrap_or_else(|| {
                panic!("invalid SyntaxKind: {}", raw);
            })
        }
    }

    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        kind.into()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Parse {
    pub green_node: GreenNode,
    pub errors:     Vec<String>,
}

pub type SyntaxNode = rowan::SyntaxNode<Lang>;
pub type SyntaxToken = rowan::SyntaxToken<Lang>;
pub type SyntaxElement = rowan::NodeOrToken<SyntaxNode, SyntaxToken>;

impl Parse {
    pub fn syntax(&self) -> SyntaxNode {
        SyntaxNode::new_root(self.green_node.clone())
    }
}

pub fn parse(text: &str) -> Parse {
    Parser {
        lexer:   Lexer::<Token>::new(text).peekable(),
        builder: GreenNodeBuilder::new(),
        errors:  Vec::new(),
    }
    .parse()
}

struct Parser<'source> {
    lexer:   Peekable<Lexer<'source, Token>>,
    builder: GreenNodeBuilder<'static>,
    errors:  Vec<String>,
}

impl Parser<'_> {
    fn parse(mut self) -> Parse {
        self.builder.start_node(SyntaxKind::Root.into());
        while self.bump() {}
        self.builder.finish_node();

        // Turn the builder into a GreenNode
        Parse {
            green_node: self.builder.finish(),
            errors:     self.errors,
        }
    }

    fn bump(&mut self) -> bool {
        let Some(token) = self.lexer.next() else {
            return false;
        };
        self.builder
            .token(SyntaxKind::Token(token).into(), self.lexer.slice());
        true
    }

    fn peek(&mut self) -> Option<Token> {}
}
