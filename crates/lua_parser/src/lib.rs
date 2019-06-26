mod ast;
mod syntax_kind;
mod syntax_error;
mod syntax_node;
mod syntax_text;
mod lexer;

pub use crate::{
    ast::{AstNode, Chunk},
    syntax_kind::SyntaxKind,
    syntax_error::{SyntaxError, SyntaxErrorKind},
    syntax_node::{
        Direction, InsertPosition, SyntaxElement, SyntaxNode, SyntaxToken, SyntaxTreeBuilder,
        TreeArc, WalkEvent,
    },
    syntax_text::SyntaxText,
    lexer::{Token, tokenize},
};
pub use rowan::{SmolStr, TextRange, TextUnit};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParseError(pub String);

use rowan::GreenNode;

impl Chunk {
//    fn new(green: GreenNode, errors: Vec<SyntaxError>) -> TreeArc<Chunk> {
//        let root = SyntaxNode::new(green, errors);
//        assert_eq!(root.kind(), SyntaxKind::CHUNK);
//        TreeArc::cast(root)
//    }
//
//    pub fn parse(text: &str) -> TreeArc<Chunk> {
//        let (green, errors) = parsing::parse_text(text);
//        Chunk::new(green, errors)
//    }

    pub fn errors(&self) -> Vec<SyntaxError> {
//        let mut errors = self.syntax.root_data().to_vec();
//        errors
        Vec::new()
    }
}