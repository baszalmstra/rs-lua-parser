extern crate lua_parser;

use lua_parser::{AstNode, Chunk};
use std::{fmt::Write, path::PathBuf};
use test_utils::{dir_tests, project_dir};

#[test]
fn lexer_tests() {
    dir_tests(&test_data_dir(), &["lexer"], |text, _| {
        let tokens = lua_parser::tokenize(text);
        dbg!(&tokens);
        dump_tokens(&tokens, text)
    });
}

//#[test]
//fn parser_tests() {
//    dir_tests(&test_data_dir(), &["parser/ok"], |text, path| {
//        let file = Chunk::parse(text);
//        let errors = file.errors();
//        assert_eq!(
//            &*errors,
//            &[] as &[mun_syntax::SyntaxError],
//            "There should be no errors in the file {:?}",
//            path.display()
//        );
//        file.syntax().debug_dump()
//    })
//}

fn test_data_dir() -> PathBuf {
    project_dir().join("crates/lua_parser/tests/data")
}

fn dump_tokens(tokens: &[lua_parser::Token], text: &str) -> String {
    let mut acc = String::new();
    let mut offset = 0;
    for token in tokens {
        let len: u32 = token.len.into();
        let len = len as usize;
        let token_text = &text[offset..offset + len];
        offset += len;
        write!(acc, "{:?} {} {:?}\n", token.kind, token.len, token_text).unwrap()
    }
    acc
}
