use super::{Parse, SyntaxKind, Token};
use crate::{Diagnostic, FileId, Span};
use logos::Lexer;
use rowan::GreenNodeBuilder;
use std::ops::Range;

const INDENT_SIZE: usize = 4;

pub(super) struct Parser<'source> {
    file_id: FileId,
    lexer:   Lexer<'source, Token>,
    peek:    Option<Token>,
    indent:  usize,
    builder: GreenNodeBuilder<'static>,
    errors:  Vec<Diagnostic>,
}

impl<'source> Parser<'source> {
    pub(super) fn new(file_id: FileId, text: &'source str) -> Self {
        let mut lexer = Lexer::new(text);
        let peek = lexer.next();
        Self {
            file_id,
            lexer,
            peek,
            indent: 0,
            builder: GreenNodeBuilder::new(),
            errors: Vec::new(),
        }
    }

    pub(super) fn finish(self) -> Parse {
        Parse {
            green_node: self.builder.finish(),
            errors:     self.errors,
        }
    }

    pub(super) fn parse_root(&mut self) {
        self.builder.start_node(SyntaxKind::Root.into());
        while self.peek.is_some() {
            self.parse_indentation();
            self.parse_def(false);
        }
        self.parse_indentation();
        self.builder.finish_node();
    }

    fn parse_indentation(&mut self) {
        let indent = if self.peek == Some(Token::Whitespace) {
            let slice = self.lexer.slice();
            if slice.len() % INDENT_SIZE != 0 || !slice.chars().all(|c| c == ' ') {
                self.error("Indentation must be a multiple of 4 spaces");
            }
            slice.len() / INDENT_SIZE
        } else {
            0
        };

        if indent > self.indent {
            if indent != self.indent + 1 {
                self.error("Indentation must increase by 4 spaces at a time");
            }
            while indent > self.indent {
                self.builder.start_node(SyntaxKind::Block.into());
                self.indent += 1;
            }
        }
        if self.peek == Some(Token::Whitespace) {
            self.bump();
        }
        if indent < self.indent {
            while indent < self.indent {
                self.builder.finish_node();
                self.indent -= 1;
            }
        }
    }

    fn parse_def(&mut self, in_group: bool) {
        let checkpoint = self.builder.checkpoint();

        let is_proc = self.try_bump_params(in_group);
        if !is_proc {
            self.builder
                .start_node_at(checkpoint, SyntaxKind::Call.into());
            self.bump_arguments(in_group);
            self.builder.finish_node();
            return;
        }

        self.builder
            .start_node_at(checkpoint, SyntaxKind::Def.into());

        self.builder
            .start_node_at(checkpoint, SyntaxKind::Proc.into());
        self.builder.finish_node();
        assert_eq!(self.peek, Some(Token::Colon));
        self.bump();

        // Parse arguments (if any)
        let checkpoint = self.builder.checkpoint();
        let count = self.bump_arguments(in_group);
        if count > 0 {
            self.builder
                .start_node_at(checkpoint, SyntaxKind::Call.into());
            self.builder.finish_node();
        }
        self.builder.finish_node();
    }

    fn try_bump_params(&mut self, in_group: bool) -> bool {
        loop {
            match self.peek {
                Some(Token::Identifier | Token::Whitespace) => self.bump(),
                Some(Token::Colon) => return true,
                Some(Token::String | Token::Number | Token::ParenOpen) => return false,
                Some(Token::Newline) | None if !in_group => {
                    return false;
                }
                Some(Token::ParenClose) if in_group => {
                    return false;
                }
                Some(Token::Newline) | None => {
                    self.error("unexpected newline");
                    return false;
                }
                Some(Token::ParenClose) => {
                    self.error("unexpected closing parenthesis");
                    self.bump();
                }
                Some(Token::Error) => {
                    self.error("unexpected character");
                    self.bump();
                }
            }
        }
    }

    fn bump_arguments(&mut self, in_group: bool) -> usize {
        let mut count = 0;
        loop {
            match self.peek {
                Some(Token::ParenOpen) => {
                    count += 1;
                    self.parse_group();
                }
                Some(Token::Identifier | Token::String | Token::Number | Token::Whitespace) => {
                    count += 1;
                    self.bump()
                }
                Some(Token::Newline) | None if !in_group => {
                    self.bump();
                    break;
                }
                Some(Token::ParenClose) if in_group => {
                    break;
                }
                Some(Token::Newline) | None => {
                    self.error("unexpected end of line");
                    break;
                }
                Some(Token::Colon) => {
                    self.error("Definition cannot contain more than one colon");
                    self.bump();
                }
                Some(Token::ParenClose) => {
                    self.error("unexpected closing parenthesis");
                    self.bump();
                }
                Some(Token::Error) => {
                    self.error("unexpected character");
                    self.bump();
                }
            }
        }
        count
    }

    fn parse_group(&mut self) {
        self.builder.start_node(SyntaxKind::Group.into());
        assert_eq!(self.peek, Some(Token::ParenOpen));
        self.bump();

        self.parse_def(true);

        // Note: This can also be EOF or newline if there is a parse error
        // assert_eq!(self.peek, Some(Token::ParenClose));
        self.bump();

        self.builder.finish_node();
    }

    fn bump(&mut self) {
        let Some(token) = self.peek else {
            panic!();
        };
        self.builder
            .token(SyntaxKind::Token(token).into(), self.lexer.slice());
        self.peek = self.lexer.next();
    }

    /// The span of the current token.
    fn span(&self) -> Span {
        self.file_id.span(self.lexer.span())
    }

    fn error<S: Into<String>>(&mut self, message: S) {
        self.errors.push(Diagnostic {
            message: message.into(),
            span:    self.span(),
        });
    }
}
