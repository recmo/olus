use super::{Parse, SyntaxKind, Token};
use logos::Lexer;
use rowan::GreenNodeBuilder;

const INDENT_SIZE: usize = 4;

pub(super) struct Parser<'source> {
    lexer:   Lexer<'source, Token>,
    peek:    Option<Token>,
    indent:  usize,
    builder: GreenNodeBuilder<'static>,
    errors:  Vec<String>,
}

impl<'source> Parser<'source> {
    pub(super) fn new(text: &'source str) -> Self {
        let mut lexer = Lexer::new(text);
        let peek = lexer.next();
        Self {
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
            self.parse_line();
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

    fn parse_line(&mut self) {
        self.builder.start_node(SyntaxKind::Line.into());
        loop {
            match self.peek {
                Some(Token::ParenOpen) => self.parse_group(),
                Some(
                    Token::Identifier
                    | Token::Colon
                    | Token::String
                    | Token::Number
                    | Token::Whitespace,
                ) => self.bump(),
                Some(Token::Newline) => {
                    self.bump();
                    break;
                }
                None => {
                    break;
                }
                Some(Token::ParenClose) => {
                    self.error("unexpected closing parenthesis");
                    self.bump();
                    break;
                }
                Some(Token::Error) => {
                    self.error("unexpected character");
                    self.bump();
                }
            }
        }
        self.builder.finish_node();
    }

    fn parse_group(&mut self) {
        self.builder.start_node(SyntaxKind::Group.into());
        self.bump(); // ParenOpen
        loop {
            match self.peek {
                Some(Token::ParenOpen) => self.parse_group(),
                Some(Token::ParenClose) => {
                    self.bump();
                    break;
                }
                Some(
                    Token::Identifier
                    | Token::Colon
                    | Token::String
                    | Token::Number
                    | Token::Whitespace,
                ) => self.bump(),
                Some(Token::Error) => {
                    self.error("unexpected character");
                    self.bump();
                }
                Some(Token::Newline) | None => {
                    self.error("unterminated parenthesized group");
                    break;
                }
            }
        }
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

    fn error(&mut self, message: &str) {
        // TODO: Diagnostics
        println!("error: {message}");
        self.errors.push(message.to_string());
    }
}
