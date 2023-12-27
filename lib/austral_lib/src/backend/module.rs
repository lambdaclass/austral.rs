use melior::ir::Module as MeliorModule;
use std::fmt::Debug;

/// A MLIR module in the context of the Austral compiler.
/// It is conformed by the MLIR module and the program registry
pub struct Module<'m> {
    pub(crate) melior_module: MeliorModule<'m>,
    // Add a proper registry after we add the missing salsa bits.
    // pub(crate) registry: ProgramRegistry<CoreType, CoreLibfunc>,
}

impl<'m> Module<'m> {
    pub fn new(
        module: MeliorModule<'m>,
        // registry: ProgramRegistry<CoreType, CoreLibfunc>,
    ) -> Self {
        Self {
            melior_module: module,
            // registry,
        }
    }

    #[allow(dead_code)]
    pub fn module(&self) -> &MeliorModule {
        &self.melior_module
    }

    // TODO: uncomment this when we have a proper Program (we should get it from the DB).
    // pub fn program_registry(&self) -> &ProgramRegistry<CoreType, CoreLibfunc> {
    //     &self.registry
    // }
}

impl Debug for Module<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.melior_module.as_operation().to_string())
    }
}
