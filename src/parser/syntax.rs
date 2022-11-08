use super::{syntax_kind::SyntaxKind, token::Token, Language};
use rowan::ast::AstNode;

pub type SyntaxNode = rowan::SyntaxNode<Language>;
pub type SyntaxToken = rowan::SyntaxToken<Language>;
pub type SyntaxElement = rowan::NodeOrToken<SyntaxNode, SyntaxToken>;

macro_rules! ast_node {
    ($ast:ident, $kind:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $ast(SyntaxNode);

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
            pub fn text(&self) -> &str {
                self.0.text()
            }

            pub fn can_cast(kind: SyntaxKind) -> bool {
                kind == SyntaxKind::Token(Token::$kind)
            }

            pub fn cast(node: SyntaxToken) -> Option<Self> {
                if Self::can_cast(node.kind()) {
                    Some(Self(node))
                } else {
                    None
                }
            }

            pub fn syntax(&self) -> &SyntaxToken {
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
ast_node!(IDef, IDef);
ast_node!(ICall, ICall);
ast_token!(Identifier, Identifier);

impl Root {
    pub fn defs(&self) -> impl Iterator<Item = Def> {
        self.syntax().children().filter_map(Def::cast)
    }
}

impl Block {
    pub fn def(&self) -> Def {
        self.syntax().prev_sibling().and_then(Def::cast).unwrap()
    }

    pub fn defs(&self) -> impl Iterator<Item = Def> {
        self.syntax().children().filter_map(Def::cast)
    }
}

impl Def {
    pub fn block(&self) -> Option<Block> {
        self.0.next_sibling().and_then(Block::cast)
    }
}
