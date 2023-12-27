use super::error::CompilerError;
use melior::{ir::Module as MeliorModule, Context as MeliorContext};

#[allow(dead_code)]
pub fn compile(
    _context: &MeliorContext,
    _module: &MeliorModule,
    // TODO: This program should be replaced with the real one from the DB.
    _program: &super::Program,
    // registry: &ProgramRegistry<TType, TLibfunc>,
) -> Result<(), CompilerError> {
    todo!()
}
