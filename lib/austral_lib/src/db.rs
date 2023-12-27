#[derive(Default)]
#[salsa::db(crate::Jar)] // parser::Jar)] // we can combine Jars here.
pub struct RootDatabase {
    storage: salsa::Storage<RootDatabase>,
}

impl RootDatabase {
    // Code left for reference: feel free to delete.
    // pub fn compile_string(&self, source: String) -> Result<(), CompilerError> {
    //     let source = parser::ProgramSource::new(self, source);
    //     let parsed_program = crate::parse::parse_program(self, source)?;

    //     let mut code_generator = codegen::CodeGenerator::new();

    //     code_generator.generate_main(parsed_program.statements(self));

    //     let code = code_generator.emit();

    //     std::fs::write("main.o", code).unwrap();

    //     let _ = codegen::link(Path::new("main.o"), Path::new("main"));

    //     Ok(())
    // }
}

// blanket implementation for salsa::Database.
impl salsa::Database for RootDatabase {}

impl salsa::ParallelDatabase for RootDatabase {
    fn snapshot(&self) -> salsa::Snapshot<RootDatabase> {
        salsa::Snapshot::new(RootDatabase {
            storage: self.storage.snapshot(),
        })
    }
}
