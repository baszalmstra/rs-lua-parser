pub type Result<T> = std::result::Result<T, failure::Error>;

use std::path::{Path, PathBuf};
pub use teraron::{Mode, Overwrite, Verify};
use clap::{App, SubCommand};

pub const GRAMMAR: &str = "crates/lua_parser/src/grammar.ron";
pub const SYNTAX_KINDS: &str = "crates/lua_parser/src/syntax_kind/generated.rs.tera";
pub const AST: &str = "crates/lua_parser/src/ast/generated.rs.tera";

pub fn generate(mode: Mode) -> Result<()> {
    let grammar = project_root().join(GRAMMAR);
    let syntax_kinds = project_root().join(SYNTAX_KINDS);
    let ast = project_root().join(AST);
    teraron::generate(&syntax_kinds, &grammar, mode)?;
    teraron::generate(&ast, &grammar, mode)?;
    Ok(())
}

pub fn project_root() -> PathBuf {
    Path::new(&env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .to_path_buf()
}

fn main() -> Result<()> {
    let matches = App::new("tasks")
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
        .subcommand(SubCommand::with_name("gen-syntax"))
        .get_matches();
    match matches
        .subcommand_name()
        .expect("Subcommand must be specified")
        {
            "gen-syntax" => generate(Overwrite)?,
            _ => unreachable!(),
        }
    Ok(())
}
