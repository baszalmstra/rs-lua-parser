use super::cursor::Cursor;

use crate::SyntaxKind::{self, *};
use super::brackets::*;

pub(crate) fn scan_comment(cursor: &mut Cursor) -> SyntaxKind {
    if let Some(level) = scan_long_bracket('[', cursor) {
        loop {
            if scan_long_bracket(']', cursor) == Some(level) {
                break;
            }
            if cursor.bump().is_none() {
                break;
            }
        }
        return COMMENT;
    } else {
        bump_until_eol(cursor);
        COMMENT
    }
}

fn bump_until_eol(cursor: &mut Cursor) {
    loop {
        if cursor.matches('\n') || cursor.matches_str("\r\n") {
            return;
        }
        if cursor.bump().is_none() {
            return;
        }
    }
}
