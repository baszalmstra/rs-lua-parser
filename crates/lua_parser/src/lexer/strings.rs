use super::cursor::Cursor;
use crate::lexer::brackets::scan_long_bracket;

pub(crate) fn scan_string(c: char, cursor: &mut Cursor) {
    let quote_type = c;
    while let Some(c) = cursor.current() {
        match c {
            '\\' => {
                cursor.bump();
                if cursor.matches('\\') || cursor.matches(quote_type) {
                    cursor.bump();
                }
            }
            c if c == quote_type => {
                cursor.bump();
                return;
            }
            _ => {
                cursor.bump();
            }
        }
    }
}

pub(crate) fn scan_long_string(level: u32, cursor: &mut Cursor) {
    while scan_long_bracket(']', cursor) == None {
        cursor.bump();
    }
}
