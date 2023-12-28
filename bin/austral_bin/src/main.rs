use clap::Parser;
use austral_lib::ast::ModuleDef;
//use chumsky::Parser;
use std::fs;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the input file
    #[arg(short, long)]
    input_file: String,

    /// Name of the output file path
    #[arg(short, long, default_value_t = String::from("output.txt"))]
    output_path: String,
}

fn main() {
    let args = Args::parse();
    let source_code = fs::read_to_string(args.input_file).unwrap();

/*     let tokens =
        austral_lib::lexer::lex(&source_code)
            .map(|(token, _span)| token)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
    let ast = ModuleDef::parser().parse(&tokens).into_result().unwrap();
    dbg!(ast); */
}
