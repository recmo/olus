//! Extension trait for [`ResolvedToken`] to give the CST some AST like
//! properties.
use {
    super::{Element, ElementRef, Kind, Node, Token},
    cstree::util::NodeOrToken,
    std::iter::once,
};

/// Extension to the [`ResolvedToken`] to give the CST some AST like properties.
pub trait TokenExt {
    /// Check if the token is an identifier binder.
    fn is_binder(&self) -> bool;

    /// Check if the token is an identifier reference.
    fn is_reference(&self) -> bool;

    /// Resolve the reference to a binder.
    fn resolve(&self) -> Option<&Token>;
}

impl TokenExt for Token {
    fn is_binder(&self) -> bool {
        self.kind() == Kind::Identifier && self.parent().kind() == Kind::Proc
    }

    fn is_reference(&self) -> bool {
        self.kind() == Kind::Identifier && self.parent().kind() != Kind::Proc
    }

    fn resolve(&self) -> Option<&Token> {
        if !self.is_reference() {
            return None;
        }

        // Intital scope is the parent Block node or Root.
        let mut scope = self
            .ancestors()
            .find(|n| matches!(n.kind(), Kind::Block | Kind::Root))
            .expect("Every token descends from root.");

        // Find the first binder in the scope.
        let identifier = self.text();
        loop {
            // Try looking up in the current scope.
            let mut current = self;
            while let Some(prev) = previous_token(scope, current) {
                if prev.is_binder() && prev.text() == identifier {
                    return Some(prev);
                }
                current = prev;
            }

            // Try looking down in the current scope.
            let mut current = self;
            while let Some(next) = next_token(scope, current) {
                if next.is_binder() && next.text() == identifier {
                    return Some(next);
                }
                current = next;
            }

            // Go up in scope.
            scope = scope.parent()?;
        }
    }
}

/// First token skipping [`Node::Block`] subtrees.
fn first_token_skipping_block(element: ElementRef) -> Option<&Token> {
    match element {
        NodeOrToken::Token(token) => Some(token),
        NodeOrToken::Node(node) if node.kind() == Kind::Block => {
            first_token_skipping_block(node.next_sibling_or_token()?)
        }
        NodeOrToken::Node(node) => first_token_skipping_block(node.first_child_or_token()?),
    }
}

/// Last token skipping [`Node::Block`] subtrees.
fn last_token_skipping_block(element: ElementRef) -> Option<&Token> {
    match element {
        NodeOrToken::Token(token) => Some(token),
        NodeOrToken::Node(node) if node.kind() == Kind::Block => {
            last_token_skipping_block(node.prev_sibling_or_token()?)
        }
        NodeOrToken::Node(node) => last_token_skipping_block(node.last_child_or_token()?),
    }
}

/// Returns the next token after element without leaving the scope or entering
/// sub-scopes.
fn next_token<'a>(scope: &'a Node, token: &'a Token) -> Option<&'a Token> {
    once(ElementRef::from(token))
        .chain(
            token
                .ancestors()
                .take_while(|&it| it != scope)
                .map(ElementRef::from),
        )
        .find_map(|it| first_token_skipping_block(it.next_sibling_or_token()?))
}

/// Returns the previous token before element without leaving the scope.
fn previous_token<'a>(scope: &'a Node, token: &'a Token) -> Option<&'a Token> {
    once(ElementRef::from(token))
        .chain(
            token
                .ancestors()
                .take_while(|&it| it != scope)
                .map(ElementRef::from),
        )
        .find_map(|it| last_token_skipping_block(it.prev_sibling_or_token()?))
}
