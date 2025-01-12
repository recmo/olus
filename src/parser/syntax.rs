use super::{Language, syntax_kind::SyntaxKind, token::Token};
use crate::{FileId, Span};
use rowan::{NodeOrToken, ast::AstNode};

pub type SyntaxNode = rowan::SyntaxNode<Language>;
pub type SyntaxToken = rowan::SyntaxToken<Language>;
pub type SyntaxElement = rowan::NodeOrToken<SyntaxNode, SyntaxToken>;

macro_rules! ast_node {
    ($ast:ident, $kind:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $ast(SyntaxNode);

        impl $ast {
            #[must_use]
            pub fn span(&self, file: FileId) -> Span {
                let range = self.syntax().text_range();
                file.span(range.start().into()..range.end().into())
            }
        }

        impl AstNode for $ast {
            type Language = Language;

            fn can_cast(kind: SyntaxKind) -> bool {
                kind == SyntaxKind::$kind
            }

            fn cast(node: SyntaxNode) -> Option<Self> {
                if Self::can_cast(node.kind()) {
                    Some(Self(node))
                } else {
                    None
                }
            }

            fn syntax(&self) -> &SyntaxNode {
                &self.0
            }
        }
    };
}

macro_rules! ast_token {
    ($ast:ident, $kind:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $ast(SyntaxToken);

        impl $ast {
            #[must_use]
            pub fn text(&self) -> &str {
                self.0.text()
            }

            #[must_use]
            pub fn span(&self, file: FileId) -> Span {
                let range = self.syntax().text_range();
                file.span(range.start().into()..range.end().into())
            }

            #[must_use]
            pub fn can_cast(kind: SyntaxKind) -> bool {
                kind == SyntaxKind::Token(Token::$kind)
            }

            #[must_use]
            pub fn cast(node: SyntaxToken) -> Option<Self> {
                if Self::can_cast(node.kind()) {
                    Some(Self(node))
                } else {
                    None
                }
            }

            #[must_use]
            pub const fn syntax(&self) -> &SyntaxToken {
                &self.0
            }
        }
    };
}

ast_node!(Root, Root);
ast_node!(Def, Def);
ast_node!(Proc, Proc);
ast_node!(Call, Call);
ast_node!(Block, Block);
ast_node!(Group, Group);
ast_token!(Identifier, Identifier);
ast_token!(String, String);
ast_token!(Number, Number);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Line {
    Def(Def),
    Call(Call),
}

impl AstNode for Line {
    type Language = Language;

    fn can_cast(kind: SyntaxKind) -> bool {
        Def::can_cast(kind) || Call::can_cast(kind)
    }

    fn cast(node: SyntaxNode) -> Option<Self> {
        match node.kind() {
            SyntaxKind::Def => Def::cast(node).map(Line::Def),
            SyntaxKind::Call => Call::cast(node).map(Line::Call),
            _ => None,
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        match self {
            Self::Def(def) => def.syntax(),
            Self::Call(call) => call.syntax(),
        }
    }
}

impl Line {
    #[must_use]
    pub fn def(&self) -> Option<Def> {
        match self {
            Self::Def(def) => Some(def.clone()),
            Self::Call(_) => None,
        }
    }

    #[must_use]
    pub fn call(&self) -> Option<Call> {
        match self {
            Self::Call(call) => Some(call.clone()),
            Self::Def(_) => None,
        }
    }

    #[must_use]
    pub fn block(&self) -> Option<Block> {
        match self {
            Self::Def(def) => def.block(),
            Self::Call(call) => call.block(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Argument {
    Identifier(Identifier),
    String(String),
    Number(Number),
    Group(Group),
}

impl Argument {
    fn can_cast(kind: SyntaxKind) -> bool {
        Identifier::can_cast(kind) || Group::can_cast(kind)
    }

    fn cast(node: SyntaxElement) -> Option<Self> {
        match node {
            SyntaxElement::Token(token) => Identifier::cast(token).map(Self::Identifier),
            SyntaxElement::Node(node) => Group::cast(node).map(Self::Group),
        }
    }
}

impl Root {
    pub fn lines(&self) -> impl Iterator<Item = Line> {
        self.syntax().children().filter_map(Line::cast)
    }

    pub fn defs(&self) -> impl Iterator<Item = Def> + use<> {
        self.syntax().children().filter_map(Def::cast)
    }

    pub fn identifier_at(&self, offset: usize) -> Option<Identifier> {
        self.syntax()
            .token_at_offset(offset.try_into().unwrap())
            .right_biased()
            .and_then(Identifier::cast)
    }
}

impl Block {
    pub fn line(&self) -> Line {
        self.syntax().prev_sibling().and_then(Line::cast).unwrap()
    }

    pub fn lines(&self) -> impl Iterator<Item = Line> {
        self.syntax().children().filter_map(Line::cast)
    }
}

impl Def {
    /// The block (if any) immediately following this definition.
    pub fn block(&self) -> Option<Block> {
        self.0.next_sibling().and_then(Block::cast)
    }

    /// The line containing this definition.
    pub fn line(&self) -> Line {
        self.syntax()
            .ancestors()
            .filter(|node| node.parent().map_or(true, |n| Block::can_cast(n.kind())))
            .find_map(Line::cast)
            .unwrap()
    }

    pub fn procedure(&self) -> Proc {
        self.syntax().children().find_map(Proc::cast).unwrap()
    }

    pub fn call(&self) -> Option<Call> {
        // Try same line first
        if let Some(call) = self.syntax().children().find_map(Call::cast) {
            return Some(call);
        }

        // Try next line in associated block
        if let Some(block) = self.block() {
            let next_line = block.lines().next();
            if let Some(call) = next_line.and_then(|line| line.call()) {
                return Some(call);
            }
        }

        // Try next line in containing block
        self.line().syntax().next_sibling().and_then(Call::cast)
    }
}

impl Proc {
    pub fn group(&self) -> Option<Group> {
        self.syntax().parent().and_then(Group::cast)
    }

    pub fn identifiers(&self) -> impl Iterator<Item = Identifier> {
        self.syntax()
            .children_with_tokens()
            .filter_map(SyntaxElement::into_token)
            .filter_map(Identifier::cast)
    }

    #[must_use]
    pub fn name(&self) -> Option<Identifier> {
        if self.group().is_some() {
            None
        } else {
            self.identifiers().next()
        }
    }

    pub fn parameters(&self) -> impl Iterator<Item = Identifier> {
        self.identifiers().skip(usize::from(self.group().is_some()))
    }
}

impl Call {
    pub fn block(&self) -> Option<Block> {
        self.0.next_sibling().and_then(Block::cast)
    }

    pub fn arguments(&self) -> impl Iterator<Item = Argument> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| match node {
                NodeOrToken::Node(node) => Group::cast(node).map(Argument::Group),
                NodeOrToken::Token(token) => match token.kind() {
                    SyntaxKind::Token(Token::Identifier) => {
                        Identifier::cast(token).map(Argument::Identifier)
                    }
                    SyntaxKind::Token(Token::String) => String::cast(token).map(Argument::String),
                    SyntaxKind::Token(Token::Number) => Number::cast(token).map(Argument::Number),
                    _ => None,
                },
            })
    }
}

impl Group {
    pub fn def(&self) -> Option<Def> {
        self.syntax().children().next().and_then(Def::cast)
    }

    pub fn call(&self) -> Option<Call> {
        self.syntax().children().next().and_then(Call::cast)
    }
}

impl Identifier {
    #[must_use]
    pub fn offset(&self) -> usize {
        self.0.text_range().start().into()
    }
}

impl String {
    #[must_use]
    pub fn value(&self) -> &str {
        let raw = self.syntax().text();
        // Trim enclosing quotes
        &raw['“'.len_utf8()..raw.len() - '”'.len_utf8()]
    }
}
