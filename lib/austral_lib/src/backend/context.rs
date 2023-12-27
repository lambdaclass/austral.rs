use melior::{
    dialect::DialectRegistry,
    ir::{Location, Module as MeliorModule},
    utility::{register_all_dialects, register_all_llvm_translations, register_all_passes},
    Context as MeliorContext,
};

use super::{error::CompilerError, module::Module, pass_manager::run_pass_manager, Program};

#[derive(Debug, Eq, PartialEq)]
pub struct Context {
    melior_context: MeliorContext,
}

unsafe impl Send for Context {}
unsafe impl Sync for Context {}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Context {
    pub fn new() -> Self {
        let melior_context = initialize_mlir();
        Self { melior_context }
    }

    /// Compiles an Austral program into MLIR and then lowers to LLVM.
    /// Returns the corresponding Module struct.
    pub fn compile(&self, program: &Program) -> Result<Module, CompilerError> {
        let mut melior_module = MeliorModule::new(Location::unknown(&self.melior_context));

        // Create the Sierra program registry
        // let registry = ProgramRegistry::<CoreType, CoreLibfunc>::new(program)?;

        super::codegen::compile(&self.melior_context, &melior_module, program)?;

        // TODO: Add proper error handling.
        run_pass_manager(&self.melior_context, &mut melior_module).unwrap();

        Ok(Module::new(melior_module))
    }
}

/// Initialize an MLIR context.
pub fn initialize_mlir() -> MeliorContext {
    let context = MeliorContext::new();
    context.append_dialect_registry(&{
        let registry = DialectRegistry::new();
        register_all_dialects(&registry);
        registry
    });
    context.load_all_available_dialects();
    register_all_passes();
    register_all_llvm_translations(&context);
    context
}
