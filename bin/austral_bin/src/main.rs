use austral_lib::{ast::ModuleDef, compiler::compile_to_binary};
use chumsky::Parser;
use melior::{dialect::DialectRegistry, Context};
use std::{
    fs,
    path::Path,
    process::{Child, Command, Stdio},
};

#[derive(clap::Parser, Debug)]
#[clap(name = "austral", about = "Austral compiler")]
struct AustralCli {
    /// File to compile
    #[arg(required = true)]
    input_file: String,

    /// Emit object file
    #[arg(short = 'o', long = "output")]
    output: Option<String>,

    #[arg(long, default_value_t = false)]
    lib: bool,

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
    let args: AustralCli = clap::Parser::parse();

    let input_file = fs::read_to_string(args.input_file).unwrap();

    let tokens = austral_lib::lexer::lex(input_file.as_str())
        .map(|(token, _span)| token)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    let ast = ModuleDef::parser().parse(&tokens).into_result().unwrap();

    if args.print_ast {
        println!("{ast:#?}");
        return;
    }

    let context = Context::new();
    context.append_dialect_registry(&{
        let dialect_registry = DialectRegistry::new();
        melior::utility::register_all_dialects(&dialect_registry);
        dialect_registry
    });
    context.load_all_available_dialects();

    let mut compiled_module = austral_lib::compiler::compile(&context, &ast, &[]);

    if args.emit_mlir {
        let mlir_code = compiled_module.as_operation();
        println!("{mlir_code}");
        return;
    }

    if args.emit_llvm || args.emit_assembler {
        austral_lib::backend::pass_manager::run_pass_manager(&context, &mut compiled_module)
            .unwrap();
        let optimized_code = compiled_module.as_operation();

        let echo_mlir = echo(&optimized_code.to_string()).unwrap();

        let mlir_traslate = Command::new("/opt/homebrew/opt/llvm/bin/mlir-translate")
            .args(["--mlir-to-llvmir", "-"])
            .stdin(Stdio::from(echo_mlir.stdout.unwrap()))
            .spawn();

        let output = mlir_traslate.unwrap().wait_with_output().unwrap();
        let llvm_ir = String::from_utf8_lossy(&output.stdout);
        if args.emit_llvm {
            println!("{}", llvm_ir.to_string());
            return;
        }

        // if args.emit_assembler {
        //     let echo_llvmir = echo(&llvm_ir.to_string()).unwrap();
        //     let clang_asm = Command::new("clang")
        //         .args(["-S", "-o", "out.s", "-x", "ir", "-"])
        //         .stdin(Stdio::from(echo_llvmir.stdout.unwrap()))
        //         .spawn();

        //     let clang_output = clang_asm.unwrap().wait_with_output().unwrap();
        //     let asm = String::from_utf8_lossy(&clang_output.stdout);
        //     println!("{}", asm.to_string());
        //     return;
        // }

        return;
    }

    let output = args.output.unwrap_or(if args.lib {
        String::from("a.dylib")
    } else {
        String::from("a.out")
    });

    compile_to_binary(&input_file, args.lib, Path::new(&output)).unwrap();
}

fn echo(text: &str) -> Result<Child, std::io::Error> {
    Command::new("echo")
        .arg(&format!("{}", text))
        .stdout(Stdio::piped())
        .spawn()
}
