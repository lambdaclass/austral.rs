pub mod ast;
mod db;
pub mod error;
pub mod lexer;
pub mod r#type;
pub mod type_system;

/// The Jar combines all the features provided by the salsa database.
/// Every tracked function, interned value, query and input must be listed here.
#[salsa::jar(db = CompilerDatabase)]
pub struct Jar(
    // examples from another project:
    // == Queries ==
    // crate::parse::parse_program

    // == Interned values ==
    // crate::ast::ProgramId,
    // crate::ast::StatementId,
    // crate::ast::FunctionId,

    // == Tracked functions ==
    // crate::ast::Program,
);

pub trait CompilerDatabase: salsa::DbWithJar<Jar> {} // + salsa::DbWithJar<parser::Jar> {}

// blanket implementation for every type that implements DbWithJar<Jar>.
// This will allow the db::Database to implement ParserDatabase without a
// concrete implemetation.
impl<DB> CompilerDatabase for DB where
    DB: ?Sized + salsa::DbWithJar<Jar> // + salsa::DbWithJar<parser::Jar> // we can combine Jars here.
{
}
