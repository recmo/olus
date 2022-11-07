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
    let mut parser = Parser::new(text);
    parser.parse_root();
    parser.finish()
}

struct Parser<'source> {
    lexer:   Lexer<'source, Token>,
    peek:    Option<Token>,
    builder: GreenNodeBuilder<'static>,
    errors:  Vec<String>,
}

impl<'source> Parser<'source> {
    fn new(text: &'source str) -> Self {
        let mut lexer = Lexer::new(text);
        let peek = lexer.next();
        Self {
            lexer,
            peek,
            builder: GreenNodeBuilder::new(),
            errors: Vec::new(),
        }
    }

    fn finish(self) -> Parse {
        Parse {
            green_node: self.builder.finish(),
            errors:     self.errors,
        }
    }

    fn parse_root(&mut self) {
        self.builder.start_node(SyntaxKind::Root.into());
        while self.peek != None {
            self.parse_item();
        }
        self.builder.finish_node();
    }

    fn parse_item(&mut self) {
        match self.peek {
            Some(Token::ParenOpen) => self.parse_group(),
            None => {}
            item => {
                self.bump();
            }
        }
    }

    fn parse_group(&mut self) {
        self.builder.start_node(SyntaxKind::Group.into());
        self.bump();
        loop {
            match self.peek {
                Some(Token::ParenClose) => {
                    self.bump();
                    break;
                }
                None => {
                    self.error("unterminated group");
                    break;
                }
                _ => self.parse_item(),
            }
        }
        self.builder.finish_node();
    }

    fn bump(&mut self) -> bool {
        let Some(token) = self.peek else {
            return false;
        };
        self.builder
            .token(SyntaxKind::Token(token).into(), self.lexer.slice());
        self.peek = self.lexer.next();
        true
    }

    fn skip_white_space(&mut self) {
        while let Some(Token::Whitespace) = self.peek {
            self.bump();
        }
    }

    fn peek(&self) -> Option<Token> {
        self.peek
    }

    fn error(&mut self, message: &str) {
        // TODO: Diagnostics
        self.errors.push(message.to_string());
    }
}
