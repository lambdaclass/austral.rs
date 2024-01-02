mod codegen;
mod context;
mod error;
mod module;
pub mod pass_manager;

// TODO: Remove this when we have a proper Program (we should get it from the DB).
#[allow(dead_code)]
pub struct Program;

pub use context::Context;
pub use module::Module;
