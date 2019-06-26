use super::cursor::Cursor;

pub(crate) fn scan_long_bracket(c: char, cursor: &mut Cursor) -> Option<u32> {
    if cursor.matches(c) {
        if let Some(level) = match_long_bracket_tail(c, 1, cursor) {
            cursor.bump_n(level+2);
            Some(level)
        } else {
            None
        }
    } else {
        None
    }
}

pub(crate) fn match_long_bracket(c: char, cursor: &mut Cursor) -> Option<u32> {
    if cursor.matches(c) {
        match_long_bracket_tail(c, 1, cursor)
    } else {
        None
    }
}

pub(crate) fn match_long_bracket_tail(c: char, offset: u32, cursor: &mut Cursor) -> Option<u32> {
    let mut level = 0;
    while cursor.nth(level + offset) == Some('=') {
        level += 1;
    }

    if cursor.nth(level + offset ) == Some(c) {
        Some(level)
    } else {
        None
    }
}