use austral_lib::ast::ModuleDef;
use chumsky::Parser as _;
use clap::Parser;
use std::fs;

#[derive(Parser, Debug)]
#[clap(name = "austral", about = "Austral compiler")]
struct AustralCli {
    /// File to compile
    #[arg(required = true)]
    input_file: String,

    /// Emit object file
    #[arg(short = 'o', long = "output", default_value_t = false)]
    emit_object: bool,

    /// Emit assembly
    #[arg(short = 's', long = "assembly", default_value_t = false)]
    emit_assembler: bool,

    /// Emit LLVM IR
    #[arg(short = 'l', long = "llvm", default_value_t = false)]
    emit_llvm: bool,

    /// Emit MLIR IR
    #[arg(short = 'm', long = "mlir", default_value_t = false)]
    emit_mlir: bool,

    /// Print AST
    #[arg(short = 'a', long = "ast", default_value_t = false)]
    print_ast: bool,
}

fn main() {
    let args = AustralCli::parse();

    let input_file = fs::read_to_string(args.input_file).unwrap();

    let tokens = austral_lib::lexer::lex(input_file.as_str())
        .map(|(token, _span)| token)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    let ast = ModuleDef::parser().parse(&tokens).into_result().unwrap();

    if args.print_ast {
        println!("{ast:#?}");
    }
}
