use {
    super::{ElementRef, Kind, Node, NodeExt, Span, Token, TokenExt},
    crate::ir::{Atom, Identifier, Procedure, Program},
    core::mem::{replace, swap},
};

enum Expression<B> {
    Atom(Atom<B>),
    Procedure {
        source:    Span,
        arguments: Vec<Identifier>,
        body:      Vec<Expression<B>>,
    },
    Call {
        source: Span,
        body:   Vec<Expression<B>>,
    },
}

struct Compiler<B, F> {
    identifiers: Vec<Identifier>,
    program:     Program<B>,
    builtins:    F,
}

#[must_use]
pub fn compile<B, F: FnMut(&str) -> Option<B>>(
    source: String,
    root: &Node,
    builtins: F,
) -> Program<B> {
    let mut compiler = Compiler {
        identifiers: Vec::new(),
        program: Program {
            source,
            procedures: Vec::new(),
        },
        builtins,
    };
    compiler.compile_node(root);
    compiler.program
}

impl<B> Expression<B> {
    #[must_use]
    const fn source(&self) -> Span {
        match self {
            Self::Atom(atom) => atom.source(),
            Self::Procedure { source, .. } | Self::Call { source, .. } => *source,
        }
    }
}

impl<B, F: FnMut(&str) -> Option<B>> Compiler<B, F> {
    fn compile_node(&mut self, node: &Node) {
        match node.kind() {
            Kind::Block => {
                for child in node.children() {
                    self.compile_node(child);
                }
            }
            Kind::Proc => {
                let source = node.span();
                let arguments = node
                    .children_with_tokens()
                    .filter_map(|n| {
                        n.as_token()
                            .filter(|t| t.is_binder())
                            .map(|t| self.parse_binder(t))
                    })
                    .collect();
                let body = node
                    .call()
                    .expect("Could not resolve procedure body.")
                    .children_with_tokens()
                    .filter_map(|e| self.parse_expression(e))
                    .collect();
                let body = self.compile_call(body);
                self.program.procedures.push(Procedure {
                    source,
                    arguments,
                    body,
                });
            }
            Kind::Call => {
                // TODO: Detect unbound calls.
            }
            _ => unreachable!(),
        }
    }

    /// Compile a call of expressions into a call of atoms.
    fn compile_call(&mut self, mut expr: Vec<Expression<B>>) -> Vec<Atom<B>> {
        // First eliminate all call groups by converting them to procedure
        // groups.
        while let Some(call) = expr
            .iter()
            .position(|e| matches!(e, Expression::Call { .. }))
        {
            // Create a new variable to name the argument.
            let source = expr[call].source();
            let (definition, reference) = self.fresh_variable(false, source);

            // Replace the call with the named argument.
            let call = replace(&mut expr[call], Expression::Atom(reference));
            let Expression::Call { mut body, .. } = call else {
                unreachable!()
            };

            // Swap the body and the expression.
            swap(&mut body, &mut expr);

            // Append a procedure node to the expression.
            expr.push(Expression::Procedure {
                source,
                arguments: vec![definition],
                body,
            });
        }

        // Then eliminate all procedure groups by giving them names.
        expr.into_iter()
            .map(|e| match e {
                Expression::Atom(atom) => atom,
                Expression::Procedure {
                    source,
                    mut arguments,
                    body,
                } => {
                    let (definition, reference) = self.fresh_variable(false, source);
                    arguments.insert(0, definition);
                    let body = self.compile_call(body);
                    self.program.procedures.push(Procedure {
                        source,
                        arguments,
                        body,
                    });
                    reference
                }
                _ => unreachable!(),
            })
            .collect()
    }

    fn parse_expression(&mut self, expr: ElementRef) -> Option<Expression<B>> {
        match expr {
            ElementRef::Token(token) => self.parse_atom(token).map(Expression::Atom),
            ElementRef::Node(node) if node.kind() == Kind::Proc => {
                let source = node.span();
                let arguments = node
                    .children_with_tokens()
                    .filter_map(|n| {
                        n.as_token()
                            .filter(|t| t.is_binder())
                            .map(|t| self.parse_binder(t))
                    })
                    .collect();
                let body = node
                    .call()
                    .expect("Could not resolve procedure body.")
                    .children_with_tokens()
                    .filter_map(|e| self.parse_expression(e))
                    .collect();
                Some(Expression::Procedure {
                    source,
                    arguments,
                    body,
                })
            }
            ElementRef::Node(node) if node.kind() == Kind::Call => {
                let source = node.span();
                let body = node
                    .children_with_tokens()
                    .filter_map(|e| self.parse_expression(e))
                    .collect();
                Some(Expression::Call { source, body })
            }
            _ => None,
        }
    }

    fn parse_binder(&mut self, identifier: &Token) -> Identifier {
        assert!(identifier.is_binder());
        self.identifiers
            .iter()
            .find(|i| i.source == identifier.span())
            .copied()
            .unwrap_or_else(|| self.fresh_variable(true, identifier.span()).0)
    }

    fn parse_atom(&mut self, atom: &Token) -> Option<Atom<B>> {
        match atom.kind() {
            Kind::String => Atom::String {
                source: atom.span(),
                value:  {
                    let text = atom.text();
                    text[1..text.len() - 1].to_string()
                },
            },
            Kind::Number => Atom::Number {
                source: atom.span(),
                value:  atom.text().parse().expect("Could not parse number."),
            },
            Kind::Identifier => {
                if let Some(binder) = atom.resolve() {
                    let binder = self.parse_binder(binder);
                    Atom::Reference {
                        source: atom.span(),
                        id:     binder.id,
                    }
                } else if let Some(builtin) = (self.builtins)(atom.text()) {
                    Atom::Builtin {
                        source: atom.span(),
                        builtin,
                    }
                } else {
                    panic!("Could not resolve identifier");
                }
            }
            _ => return None,
        }
        .into()
    }

    /// Construct a fresh name for an anonymous expression.
    fn fresh_variable(&mut self, named: bool, source: Span) -> (Identifier, Atom<B>) {
        let id = self.identifiers.len() as u32;
        let identifier = Identifier { source, named, id };
        let atom = Atom::Reference { source, id };
        self.identifiers.push(identifier);
        (identifier, atom)
    }
}
