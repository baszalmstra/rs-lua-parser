mod brackets;
mod classes;
mod comments;
mod cursor;
mod numbers;
mod strings;

use self::{
    classes::*, comments::scan_comment, cursor::Cursor, numbers::scan_number, strings::scan_string, brackets::*
};
use crate::lexer::strings::scan_long_string;
use crate::{
    SyntaxKind::{self, *},
    TextUnit,
};

/// A token of Mun source
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Token {
    /// The kind of token
    pub kind: SyntaxKind,

    /// The length of the token
    pub len: TextUnit,
}

/// Break a string up into its component tokens
pub fn tokenize(text: &str) -> Vec<Token> {
    let mut text = text;
    let mut result = Vec::new();
    while !text.is_empty() {
        let token = next_token(text);
        result.push(token);
        let len: u32 = token.len.into();
        text = &text[len as usize..];
    }
    result
}

/// Get the next token from a string
pub fn next_token(text: &str) -> Token {
    assert!(!text.is_empty());
    let mut ptr = Cursor::new(text);
    let c = ptr.bump().unwrap();
    let kind = next_token_inner(c, &mut ptr);
    let len = ptr.into_len();
    Token { kind, len }
}

fn next_token_inner(c: char, cursor: &mut Cursor) -> SyntaxKind {
    if is_whitespace(c) {
        cursor.bump_while(is_whitespace);
        return WHITESPACE;
    }

    match c {
        '-' if cursor.matches('-') => {
            cursor.bump();
            return scan_comment(cursor);
        }
        _ => (),
    }

    if c == '[' {
        if let Some(level) = match_long_bracket_tail('[', 0, cursor) {
            cursor.bump_n(level + 1);
            scan_long_string(level, cursor);
            return STRING;
        }
    }

    let ident_start = is_ident_start(c);
    if ident_start {
        return scan_identifier_or_keyword(c, cursor);
    }

    if is_dec_digit(c) {
        return scan_number(c, cursor);
    }

    if let Some(kind) = SyntaxKind::from_char(c) {
        return kind;
    }

    match c {
        '~' if cursor.matches('=') => {
            cursor.bump();
            return NEQ;
        }
        '"' | '\'' => {
            scan_string(c, cursor);
            return STRING;
        }
        _ => (),
    }
    ERROR
}

fn scan_identifier_or_keyword(c: char, cursor: &mut Cursor) -> SyntaxKind {
    cursor.bump_while(is_ident_continue);
    if let Some(kind) = SyntaxKind::from_keyword(cursor.current_token_text()) {
        return kind;
    }
    IDENT
}
